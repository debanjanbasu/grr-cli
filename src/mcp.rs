//! Model Context Protocol (MCP) server for grr.
//!
//! Hand-rolled JSON-RPC 2.0 over stdio — no SDK crate: a tool-only server
//! needs five methods. Newline-delimited requests in, one response line
//! per request out, logs to stderr ONLY (stdout is the MCP stream, the
//! same discipline as the rest of grr).
//!
//! * `initialize` — protocol-version negotiation + capabilities
//! * `notifications/initialized` — a notification; never answered
//! * `tools/list` — every method in the Discovery index as a tool
//! * `tools/call` — one method invocation through the shared call path
//! * `ping` — an empty result
//!
//! Tool names are the dotted method ids `grr api call` takes
//! (`gmail.users.messages.list`), so an agent reaches every method Google
//! publishes through the same resolution the CLI uses. Errors never kill
//! the server: a tool failure is an `isError` result (MCP's tool-error
//! contract), a protocol failure a JSON-RPC error object.
//!
//! ```text
//! {"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25"}}
//! {"jsonrpc":"2.0","method":"notifications/initialized"}
//! {"jsonrpc":"2.0","id":2,"method":"tools/list"}
//! {"jsonrpc":"2.0","id":3,"method":"tools/call",
//!  "params":{"name":"gmail.users.messages.list","arguments":{"userId":"me","dry_run":true}}}
//! ```

use crate::commands::api::{self, call_method};
use crate::commands::safety::{SafetyProfile, is_write_verb};
use crate::core::auth::GoogleAuth;
use crate::discovery::{self, Method, Parameter, Service};
use serde_json::{Map, Value, json};

/// The protocol version this server speaks. A client asking for a version
/// we understand gets it echoed; anything else gets this one — MCP's
/// negotiation rule makes the server's answer authoritative.
///
/// `2025-11-25` is the latest revision the official TypeScript SDK ships
/// (`LATEST_PROTOCOL_VERSION` in @modelcontextprotocol/sdk 1.31.0), and
/// its handshake is byte-compatible with this server's surface. The newer
/// spec revision `2026-07-28` REMOVED the `initialize` handshake in favor
/// of `server/discover` plus per-request version metadata — a different
/// handshake shape this server does not implement, so it is deliberately
/// not in the list: echoing it would promise a surface we cannot serve.
pub(crate) const PROTOCOL_VERSION: &str = "2025-11-25";

/// Versions understood well enough to echo verbatim.
const SUPPORTED_VERSIONS: [&str; 3] = ["2024-11-05", "2025-06-18", "2025-11-25"];

/// JSON-RPC 2.0 error codes.
const PARSE_ERROR: i64 = -32700;
const INVALID_REQUEST: i64 = -32600;
const METHOD_NOT_FOUND: i64 = -32601;

/// One server instance: one credential, one safety profile, every tool.
/// The stdio loop is single-tasked, so no `Arc` is needed.
pub struct McpServer {
    auth: GoogleAuth,
    profile: SafetyProfile,
}

impl McpServer {
    pub fn new(auth: GoogleAuth, profile: SafetyProfile) -> Self {
        Self { auth, profile }
    }

    /// How many tools the Discovery index exposes, for the startup banner.
    pub fn tool_count(&self) -> usize {
        discovery::services()
            .values()
            .map(|s| s.methods.len())
            .sum()
    }

    /// Handle one line of JSON-RPC. `Some(response)` for requests (and for
    /// parse errors, which answer with a null id); `None` for
    /// notifications, which get no response. Never panics and never kills
    /// the server: every failure is an error-shaped response.
    pub async fn handle_line(&self, line: &str) -> Option<Value> {
        let message: Value = match serde_json::from_str(line) {
            Ok(message) => message,
            Err(error) => {
                return Some(error_response(
                    &Value::Null,
                    PARSE_ERROR,
                    format!("parse error: {error}"),
                ));
            }
        };

        // A message without an id is a notification (`notifications/
        // initialized` included): it gets no response by definition.
        let id = message.get("id").cloned()?;

        let method = message.get("method").and_then(|m| m.as_str()).unwrap_or("");
        let params = message.get("params").cloned().unwrap_or(Value::Null);

        let response = match method {
            "initialize" => json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": initialize_result(&params),
            }),
            "ping" => json!({ "jsonrpc": "2.0", "id": id, "result": {} }),
            "tools/list" => json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": { "tools": tools_list() },
            }),
            "tools/call" => {
                let result = self.tools_call(&params).await;
                json!({ "jsonrpc": "2.0", "id": id, "result": result })
            }
            "" => error_response(&id, INVALID_REQUEST, "request has no method".to_owned()),
            other => error_response(&id, METHOD_NOT_FOUND, format!("method not found: {other}")),
        };
        Some(response)
    }

    /// `tools/call`: resolve the tool name, assemble the parameters,
    /// invoke the shared call path. EVERY error — unknown tool, refused by
    /// the safety profile, HTTP failure — is an `isError` result, never a
    /// process exit: the server must survive one bad call.
    async fn tools_call(&self, params: &Value) -> Value {
        let Some(name) = params.get("name").and_then(|n| n.as_str()) else {
            return error_result("tools/call needs a tool `name`");
        };
        let arguments = params
            .get("arguments")
            .cloned()
            .unwrap_or_else(|| Value::Object(Map::new()));
        let Value::Object(args) = arguments else {
            return error_result("tools/call `arguments` must be an object");
        };
        match self.invoke_tool(name, &args).await {
            Ok(text) => ok_result(text),
            Err(message) => error_result(message),
        }
    }

    async fn invoke_tool(&self, name: &str, args: &Map<String, Value>) -> Result<String, String> {
        let (service, method) = discovery::resolve(name)
            .map_err(|message| format!("unknown tool `{name}`: {message}"))?;

        // The same gate the CLI dispatch honors (src/commands/safety.rs).
        self.profile.check(&service.name, method)?;

        // Assemble the parameter map: the `params` escape-hatch object goes
        // in first, then the tool-call arguments — the arguments are the
        // typed params and win on conflict ("typed params win", the merge
        // order of `api::parse_params`).
        let mut merged = Map::new();
        if let Some(Value::Object(object)) = args.get("params") {
            merged.extend(object.clone());
        }

        let mut body_file: Option<String> = None;
        let mut query: Vec<String> = Vec::new();
        let mut dry_run = false;
        for (key, value) in args {
            match key.as_str() {
                "params" => {}
                // Both spellings: the input schema declares `dry_run`, but
                // agents routinely write `dryRun`. No Discovery parameter
                // uses either name, so consuming them is safe.
                "dry_run" | "dryRun" => dry_run = value.as_bool().unwrap_or(false),
                "body_file" | "bodyFile" => body_file = value.as_str().map(str::to_owned),
                "query" => {
                    if let Some(items) = value.as_array() {
                        query.extend(items.iter().filter_map(|v| v.as_str().map(str::to_owned)));
                    }
                }
                other => {
                    merged.insert(other.to_owned(), value.clone());
                }
            }
        }

        // stdin IS the MCP stream: a `-` body_file would swallow JSON-RPC
        // lines. Say so rather than corrupting the session.
        if body_file.as_deref() == Some("-") {
            return Err(
                "body_file `-` (stdin) is not available over MCP; pass a file path".to_owned(),
            );
        }

        let payload = call_method(
            Some(&self.auth),
            service,
            method,
            merged,
            api::CallOptions {
                body_file,
                // The verb override is `grr api call`'s escape hatch; MCP
                // speaks the Discovery document.
                verb_override: None,
                query_extras: query,
                dry_run,
            },
        )
        .await
        .map_err(|error| error.to_string())?;

        serde_json::to_string(&payload)
            .map_err(|error| format!("serialising the result failed: {error}"))
    }
}

fn initialize_result(params: &Value) -> Value {
    let requested = params.get("protocolVersion").and_then(|v| v.as_str());
    let version = match requested {
        Some(version) if SUPPORTED_VERSIONS.contains(&version) => (*version).to_owned(),
        _ => PROTOCOL_VERSION.to_owned(),
    };
    json!({
        "protocolVersion": version,
        "capabilities": { "tools": {} },
        "serverInfo": { "name": "grr", "version": env!("CARGO_PKG_VERSION") },
    })
}

/// Every method in the Discovery index as an MCP tool. The index grows as
/// Google publishes APIs — the list is derived, never a hardcoded count.
fn tools_list() -> Vec<Value> {
    discovery::services()
        .values()
        .flat_map(|service| service.methods.iter().map(|m| tool_descriptor(service, m)))
        .collect()
}

fn tool_descriptor(service: &Service, method: &Method) -> Value {
    json!({
        "name": format!("{}.{}", service.name, method.id),
        "description": tool_description(method),
        "inputSchema": input_schema(method),
    })
}

/// The method's Discovery description + its HTTP verb + a hint that write
/// verbs need auth scopes (and are gated by `--readonly`).
fn tool_description(method: &Method) -> String {
    let verb = method.http_method.to_ascii_uppercase();
    let mut text = format!("{} [HTTP {verb}]", method.description);
    if is_write_verb(&verb) {
        text.push_str(" — write verb: needs auth scopes; refused under --readonly safety profiles");
    } else {
        text.push_str(" — needs auth scopes");
    }
    text
}

/// The tool's input schema: a JSON-schema object built from the method's
/// parameters plus grr's escape hatches; `required` is the method's
/// required parameter names.
fn input_schema(method: &Method) -> Value {
    let mut properties = Map::new();
    for param in &method.parameters {
        properties.insert(param.name.clone(), param_schema(param));
    }
    properties.insert(
        "params".into(),
        json!({
            "type": "object",
            "additionalProperties": true,
            "description": "Parameter map merged into the request; typed parameters win.",
        }),
    );
    properties.insert("body_file".into(), json!({
        "type": "string",
        "description": "Read the request body from this file ('-' for stdin) instead of building one from the parameters.",
    }));
    properties.insert("query".into(), json!({
        "type": "array",
        "items": { "type": "string" },
        "description": "Extra KEY=VALUE query pairs for endpoints whose parameters Discovery does not describe.",
    }));
    properties.insert(
        "dry_run".into(),
        json!({
            "type": "boolean",
            "description": "Return the request that would be sent, without sending it.",
        }),
    );

    let mut schema = Map::new();
    schema.insert("type".into(), json!("object"));
    schema.insert("properties".into(), Value::Object(properties));
    if !method.required.is_empty() {
        schema.insert("required".into(), json!(method.required));
    }
    Value::Object(schema)
}

/// One parameter as a JSON-schema property. Discovery's scalar kinds map
/// to JSON-schema types; anything else (`any`, nested objects) omits the
/// type rather than lie. A repeated parameter becomes an array of its base
/// type, which is what `build_query` expands at call time.
fn param_schema(param: &Parameter) -> Value {
    let mut out = Map::new();
    match param.kind.as_str() {
        "string" => {
            out.insert("type".into(), json!("string"));
        }
        "integer" => {
            out.insert("type".into(), json!("integer"));
        }
        "number" => {
            out.insert("type".into(), json!("number"));
        }
        "boolean" => {
            out.insert("type".into(), json!("boolean"));
        }
        _ => {}
    }
    if !param.description.is_empty() {
        out.insert("description".into(), json!(param.description));
    }
    if let Some(values) = &param.enum_values {
        out.insert("enum".into(), json!(values));
    }
    let base = Value::Object(out);
    if param.repeated {
        json!({ "type": "array", "items": base })
    } else {
        base
    }
}

fn error_response(id: &Value, code: i64, message: String) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": { "code": code, "message": message },
    })
}

fn ok_result(text: String) -> Value {
    json!({ "content": [ { "type": "text", "text": text } ] })
}

fn error_result(message: impl Into<String>) -> Value {
    json!({ "content": [ { "type": "text", "text": message.into() } ], "isError": true })
}

/// The stdio loop: newline-delimited JSON-RPC in, one response line per
/// request out. Generic over the reader/writer so tests can run the whole
/// handshake in-process. EOF on stdin (the client closed the session) is
/// a clean shutdown.
pub async fn serve<R, W>(server: &McpServer, reader: R, writer: &mut W) -> anyhow::Result<()>
where
    R: tokio::io::AsyncBufRead + Unpin,
    W: tokio::io::AsyncWrite + Unpin,
{
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

    let mut lines = reader.lines();
    while let Some(line) = lines.next_line().await? {
        if line.trim().is_empty() {
            continue;
        }
        if let Some(response) = server.handle_line(&line).await {
            writer.write_all(response.to_string().as_bytes()).await?;
            writer.write_all(b"\n").await?;
            writer.flush().await?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::auth::TokenStorage;
    use crate::core::config::OAuthConfig;

    async fn test_server() -> McpServer {
        test_server_with_profile(SafetyProfile::default()).await
    }

    async fn test_server_with_profile(profile: SafetyProfile) -> McpServer {
        // `GoogleAuth::with_token` is the documented test hook: it skips
        // OAuth and persistent storage entirely. Dry-run and gate tests
        // never touch the network.
        let config = OAuthConfig {
            client_id: "test-id.apps.googleusercontent.com".into(),
            client_secret: None,
        };
        let storage = TokenStorage {
            access_token: "test-token".into(),
            refresh_token: None,
            expires_at: u64::MAX,
            token_type: "Bearer".into(),
            scope: String::new(),
        };
        let auth = GoogleAuth::with_token(config, storage).await.unwrap();
        McpServer::new(auth, profile)
    }

    fn request(id: i64, method: &str, params: &str) -> String {
        format!(r#"{{"jsonrpc":"2.0","id":{id},"method":"{method}","params":{params}}}"#)
    }

    #[tokio::test]
    async fn the_stdio_loop_pipes_the_full_handshake() {
        let server = test_server().await;
        let input = format!(
            "{}\n{}\n{}\n{}\n",
            request(
                1,
                "initialize",
                r#"{"protocolVersion":"2024-11-05","clientInfo":{"name":"t","version":"0"}}"#
            ),
            r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#,
            request(2, "tools/list", "{}"),
            request(3, "ping", "{}"),
        );
        let mut out: Vec<u8> = Vec::new();
        serve(&server, input.as_bytes(), &mut out).await.unwrap();

        // One response line per REQUEST; the notification got none.
        let lines: Vec<Value> = String::from_utf8(out)
            .unwrap()
            .lines()
            .map(|l| serde_json::from_str(l).unwrap())
            .collect();
        assert_eq!(lines.len(), 3);

        assert_eq!(lines[0]["jsonrpc"], "2.0");
        assert_eq!(lines[0]["id"], 1);
        assert!(lines[0]["result"]["protocolVersion"].is_string());
        assert!(lines[0]["result"]["capabilities"]["tools"].is_object());
        assert_eq!(lines[0]["result"]["serverInfo"]["name"], "grr");
        assert!(lines[0]["result"]["serverInfo"]["version"].is_string());

        assert_eq!(lines[1]["id"], 2);
        let tools = lines[1]["result"]["tools"].as_array().unwrap();
        assert_eq!(tools.len(), server.tool_count());

        assert_eq!(lines[2]["id"], 3);
        assert_eq!(lines[2]["result"], json!({}));
    }

    #[tokio::test]
    async fn initialize_echoes_a_supported_protocol_version() {
        let server = test_server().await;

        let line = request(1, "initialize", r#"{"protocolVersion":"2025-06-18"}"#);
        let response = server.handle_line(&line).await.unwrap();
        assert_eq!(response["result"]["protocolVersion"], "2025-06-18");

        let line = request(2, "initialize", r#"{"protocolVersion":"2025-11-25"}"#);
        let response = server.handle_line(&line).await.unwrap();
        assert_eq!(response["result"]["protocolVersion"], "2025-11-25");

        // An unknown version: the server's answer is authoritative.
        let line = request(3, "initialize", r#"{"protocolVersion":"1999-01-01"}"#);
        let response = server.handle_line(&line).await.unwrap();
        assert_eq!(response["result"]["protocolVersion"], PROTOCOL_VERSION);
    }

    #[tokio::test]
    async fn tools_list_covers_every_index_method_with_an_input_schema() {
        let server = test_server().await;
        let line = request(2, "tools/list", "{}");
        let response = server.handle_line(&line).await.unwrap();
        let tools = response["result"]["tools"].as_array().unwrap();

        // The index grows as Google publishes APIs — never a hardcoded count.
        let expected: usize = discovery::services()
            .values()
            .map(|s| s.methods.len())
            .sum();
        assert_eq!(tools.len(), expected);

        for tool in tools {
            let name = tool["name"].as_str().unwrap();
            // Tool names are the dotted method ids `grr api call` takes.
            assert!(name.contains('.'), "{name} is not a dotted id");
            assert!(discovery::resolve(name).is_ok(), "{name} does not resolve");
            assert_eq!(tool["inputSchema"]["type"], "object", "{name}");
            assert!(tool["inputSchema"]["properties"].is_object(), "{name}");
            assert!(
                tool["description"].as_str().unwrap().contains("HTTP"),
                "{name}"
            );
        }
    }

    #[tokio::test]
    async fn input_schemas_carry_the_escape_hatches_and_required_params() {
        let server = test_server().await;
        let line = request(2, "tools/list", "{}");
        let response = server.handle_line(&line).await.unwrap();
        let tools = response["result"]["tools"].as_array().unwrap();
        let schema = tools
            .iter()
            .find(|t| t["name"] == "gmail.users.messages.list")
            .map(|t| t["inputSchema"].clone())
            .unwrap();

        for prop in ["params", "body_file", "query", "dry_run"] {
            assert!(schema["properties"][prop].is_object(), "missing {prop}");
        }
        let required: Vec<&str> = schema["required"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|v| v.as_str())
            .collect();
        assert!(required.contains(&"userId"), "{required:?}");

        // Typed parameters keep their Discovery types; labelIds is repeated.
        assert_eq!(schema["properties"]["maxResults"]["type"], "integer");
        assert_eq!(schema["properties"]["labelIds"]["type"], "array");
        assert_eq!(schema["properties"]["labelIds"]["items"]["type"], "string");
    }

    #[tokio::test]
    async fn enum_values_surface_in_the_schema() {
        let server = test_server().await;
        let line = request(2, "tools/list", "{}");
        let response = server.handle_line(&line).await.unwrap();
        let tools = response["result"]["tools"].as_array().unwrap();
        let schema = tools
            .iter()
            .find(|t| t["name"] == "calendar.calendarList.list")
            .map(|t| t["inputSchema"].clone())
            .unwrap();
        let values = schema["properties"]["minAccessRole"]["enum"]
            .as_array()
            .unwrap();
        assert!(
            values.iter().any(|v| v.as_str() == Some("owner")),
            "{values:?}"
        );
    }

    #[tokio::test]
    async fn tools_call_dry_run_matches_the_api_call_payload() {
        let server = test_server().await;
        let line = request(
            3,
            "tools/call",
            r#"{"name":"gmail.users.messages.list","arguments":{"userId":"me","dry_run":true}}"#,
        );
        let response = server.handle_line(&line).await.unwrap();
        assert_eq!(response["result"]["isError"], Value::Null);
        let text = response["result"]["content"][0]["text"].as_str().unwrap();
        let payload: Value = serde_json::from_str(text).unwrap();

        // The same dry-run payload `grr api call --dry-run` prints, via
        // the shared plan path.
        let (service, method) = discovery::resolve("gmail.users.messages.list").unwrap();
        let mut params = Map::new();
        params.insert("userId".into(), json!("me"));
        let plan = api::plan_request(service, method, &params, None, None, &[]).unwrap();
        assert_eq!(payload, api::dry_run_payload(method, &plan));
    }

    #[tokio::test]
    async fn tools_call_accepts_the_camel_case_dry_run_spelling() {
        let server = test_server().await;
        let line = request(
            4,
            "tools/call",
            r#"{"name":"gmail.users.messages.list","arguments":{"userId":"me","dryRun":true}}"#,
        );
        let response = server.handle_line(&line).await.unwrap();
        let text = response["result"]["content"][0]["text"].as_str().unwrap();
        let payload: Value = serde_json::from_str(text).unwrap();
        assert_eq!(payload["dryRun"], true);
    }

    #[tokio::test]
    async fn tool_arguments_win_over_the_params_escape_hatch() {
        let server = test_server().await;
        let line = request(
            5,
            "tools/call",
            r#"{"name":"gmail.users.messages.list","arguments":
                {"params":{"userId":"someone-else","q":"is:unread"},"userId":"me","dry_run":true}}"#,
        );
        let response = server.handle_line(&line).await.unwrap();
        let text = response["result"]["content"][0]["text"].as_str().unwrap();
        let payload: Value = serde_json::from_str(text).unwrap();
        // The typed argument wins; the escape hatch's other key rides along.
        let url = payload["url"].as_str().unwrap();
        assert!(url.contains("/users/me/messages"), "{url}");
        assert!(url.contains("q=is%3Aunread"), "{url}");
        assert!(!url.contains("someone-else"), "{url}");
    }

    #[tokio::test]
    async fn an_unknown_tool_is_an_error_result_not_a_crash() {
        let server = test_server().await;
        let line = request(
            6,
            "tools/call",
            r#"{"name":"gmail.users.messages.lst","arguments":{}}"#,
        );
        let response = server.handle_line(&line).await.unwrap();
        assert_eq!(response["result"]["isError"], true);
        let text = response["result"]["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("unknown tool"), "{text}");
    }

    #[tokio::test]
    async fn readonly_refuses_writes_with_an_mcp_error_result() {
        let server = test_server_with_profile(SafetyProfile::readonly()).await;

        let line = request(
            7,
            "tools/call",
            r#"{"name":"gmail.users.messages.send","arguments":{"userId":"me","dry_run":true}}"#,
        );
        let response = server.handle_line(&line).await.unwrap();
        assert_eq!(response["result"]["isError"], true);
        let text = response["result"]["content"][0]["text"].as_str().unwrap();
        // Actionable: names the method, the verb and how to lift it.
        assert!(text.contains("gmail.users.messages.send"), "{text}");
        assert!(text.contains("POST"), "{text}");
        assert!(text.contains("Rerun without --readonly"), "{text}");

        // A GET still runs under the same profile.
        let line = request(
            8,
            "tools/call",
            r#"{"name":"gmail.users.messages.list","arguments":{"userId":"me","dry_run":true}}"#,
        );
        let response = server.handle_line(&line).await.unwrap();
        assert_eq!(response["result"]["isError"], Value::Null);
    }

    #[tokio::test]
    async fn a_missing_tool_name_is_an_error_result() {
        let server = test_server().await;
        let line = request(9, "tools/call", r#"{"arguments":{}}"#);
        let response = server.handle_line(&line).await.unwrap();
        assert_eq!(response["result"]["isError"], true);
    }

    #[tokio::test]
    async fn a_stdin_body_file_is_refused_over_mcp() {
        let server = test_server().await;
        let line = request(
            10,
            "tools/call",
            r#"{"name":"gmail.users.messages.list","arguments":{"body_file":"-"}}"#,
        );
        let response = server.handle_line(&line).await.unwrap();
        assert_eq!(response["result"]["isError"], true);
        let text = response["result"]["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("stdin"), "{text}");
    }

    #[tokio::test]
    async fn an_unknown_method_is_a_jsonrpc_error() {
        let server = test_server().await;
        let line = request(11, "resources/list", "{}");
        let response = server.handle_line(&line).await.unwrap();
        assert_eq!(response["error"]["code"], -32601);
        assert!(
            response["error"]["message"]
                .as_str()
                .unwrap()
                .contains("resources/list")
        );
        assert_eq!(response["id"], 11);
    }

    #[tokio::test]
    async fn a_parse_error_answers_with_a_null_id() {
        let server = test_server().await;
        let response = server.handle_line("not json at all").await.unwrap();
        assert_eq!(response["error"]["code"], -32700);
        assert!(response["id"].is_null());
    }

    #[tokio::test]
    async fn notifications_are_never_answered() {
        let server = test_server().await;
        assert!(
            server
                .handle_line(r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#)
                .await
                .is_none()
        );
        assert!(
            server
                .handle_line(r#"{"jsonrpc":"2.0","method":"notifications/cancelled"}"#)
                .await
                .is_none()
        );
    }
}
