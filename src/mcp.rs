//! Model Context Protocol (MCP) server for grr.
//!
//! Hand-rolled JSON-RPC 2.0 over stdio — no SDK crate: a tool-only server
//! needs four RPCs. Newline-delimited requests in, one response line
//! per request out, logs to stderr ONLY (stdout is the MCP stream, the
//! same discipline as the rest of grr).
//!
//! * `server/discover` — the stateless revision's discovery RPC: revisions
//!   served, capabilities, identity, and it needs no prior call
//! * `tools/list` — every method in the Discovery index as a tool
//! * `tools/call` — one method invocation through the shared call path
//!
//! `2026-07-28` is the newest revision served, so the handshake is gone from
//! the happy path — but `initialize` is answered with a minimal legacy
//! handshake (server/discover remains the discovery RPC), because the
//! deployed client ecosystem still opens with it. `ping` and
//! `logging/setLevel` answer with a JSON-RPC error naming the revision to
//! speak instead. Notifications (`notifications/initialized`, whatever the
//! revision) carry no id, so they are never answered at all.
//!
//! Tool names are the dotted method ids `grr api call` takes
//! (`gmail.users.messages.list`), so an agent reaches every method Google
//! publishes through the same resolution the CLI uses. Errors never kill
//! the server: a tool failure is an `isError` result (MCP's tool-error
//! contract), a protocol failure a JSON-RPC error object.
//!
//! ```text
//! {"jsonrpc":"2.0","id":1,"method":"server/discover"}
//! {"jsonrpc":"2.0","id":2,"method":"tools/list"}
//! {"jsonrpc":"2.0","id":3,"method":"tools/call",
//!  "params":{"name":"gmail.users.messages.list","arguments":{"userId":"me","dry_run":true}}}
//! ```

use crate::commands::api::{self, call_method};
use crate::commands::safety::{SafetyProfile, is_write_verb};
use crate::core::auth::GoogleAuth;
use crate::discovery::{self, Method, Parameter, Service};
use serde_json::{Map, Value, json};

/// The one protocol revision this server speaks: `2026-07-28`, the current
/// MCP revision, and a *stateless* redesign — it removed the `initialize`/
/// `notifications/initialized` handshake, made `server/discover` mandatory,
/// and moved the protocol version and client capabilities into per-request
/// `_meta`.
///
/// Legacy revisions are deliberately not served. Their handshake shape
/// (`ping`, `logging/setLevel`) is gone from the spec; `initialize` alone
/// is answered with a minimal legacy handshake so standard clients
/// (mcp-proxy, Claude Desktop, Cursor, Glama's introspector) can connect —
/// they then use `tools/*`, or read this constant out of `server/discover`
/// and speak the current revision.
pub(crate) const PROTOCOL_VERSION: &str = "2026-07-28";

/// Every revision this server can serve, newest first — exactly what
/// `server/discover` reports as `supportedVersions`. One entry, on purpose:
/// this server speaks the current revision and nothing older.
const SUPPORTED_VERSIONS: [&str; 1] = ["2026-07-28"];

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
            "server/discover" => json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": discover_result(),
            }),
            "tools/list" => json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": { "tools": tools_list(), "_meta": server_meta() },
            }),
            "tools/call" => {
                let mut result = self.tools_call(&params).await;
                // The stateless revision has no handshake to carry identity,
                // so every result names the server that produced it.
                if let Some(object) = result.as_object_mut() {
                    object.insert("_meta".to_owned(), server_meta());
                }
                json!({ "jsonrpc": "2.0", "id": id, "result": result })
            }
            // `initialize` compat: the stateless revision removed the
            // handshake, but the deployed client ecosystem (mcp-proxy,
            // Claude Desktop, Cursor, Glama's introspector) still opens
            // with it — and several of its members REJECT an answer that
            // names a newer revision than they asked for. Negotiate:
            // echo the client's requested revision back when it looks
            // like an MCP revision (dated string), otherwise answer with
            // the newest one we speak. Either way the client then uses
            // `tools/*`, which is revision-agnostic here.
            // `ping` stays refused: nothing in the legacy flow needs it
            // to connect.
            "initialize" => {
                let requested = params
                    .get("protocolVersion")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let looks_like_revision =
                    |v: &str| v.len() == 10 && v.as_bytes()[4] == b'-' && v.starts_with("20");
                let version = if looks_like_revision(requested) {
                    requested
                } else {
                    PROTOCOL_VERSION
                };
                json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "protocolVersion": version,
                        "capabilities": { "tools": {} },
                        "serverInfo": server_info(),
                        "instructions": discover_result()["instructions"].clone(),
                    },
                })
            }
            "logging/setLevel" => error_response(
                &id,
                METHOD_NOT_FOUND,
                format!(
                    "`{method}` is not part of MCP {PROTOCOL_VERSION} — the handshake is gone. \
                     Call `server/discover` for the revisions served, then `tools/*` inline."
                ),
            ),
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

/// The server's identity, for `_meta`. With no handshake to carry it once,
/// `io.modelcontextprotocol/serverInfo` on EVERY result is the only place a
/// client can read it.
fn server_info() -> Value {
    json!({ "name": "grr", "version": env!("CARGO_PKG_VERSION") })
}

fn server_meta() -> Value {
    json!({ "io.modelcontextprotocol/serverInfo": server_info() })
}

/// `server/discover` — the mandatory RPC of the stateless revision. No body
/// parameters, no prior call, no state: it reports the revisions served, the
/// capabilities, and the identity, so a client can pick a revision (or just
/// call `tools/*` inline) before doing anything else.
fn discover_result() -> Value {
    json!({
        "resultType": "complete",
        "supportedVersions": SUPPORTED_VERSIONS,
        "capabilities": { "tools": {} },
        "_meta": server_meta(),
        "instructions": "Google Workspace and Google Cloud APIs from the embedded Discovery \
                         index. Tool names are dotted method ids (gmail.users.messages.list); \
                         --readonly, --deny-service and --deny-verb gate every write.",
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
    // outputSchema: the pruned response shape for this method's $ref, when
    // the upstream document named one (~87% of methods). GET/list tools
    // declare their payload shape; a write tool's outputSchema describes
    // the post-write resource state.
    let output_schema = method
        .response_ref
        .as_deref()
        .and_then(|name| service.schemas.get(name))
        .map(|schema| serde_json::to_value(schema).expect("Schema serializes"));
    let mut descriptor = json!({
        "name": format!("{}.{}", service.name, method.id),
        "description": tool_description(service, method),
        "inputSchema": input_schema(service, method),
    });
    if let Some(schema) = output_schema {
        descriptor
            .as_object_mut()
            .expect("descriptor is an object")
            .insert("outputSchema".into(), schema);
    }
    descriptor
}

/// Curated descriptions for the few Discovery methods that ship none —
/// high-impact tools an agent must not have to guess at. Keyed by the
/// method's index id (resource path + verb, WITHOUT the service prefix —
/// the service name joins at tool-name assembly, not in `Method.id`);
/// anything not listed falls through to the verb-derived text.
const DESCRIPTION_OVERRIDES: &[(&str, &str)] = &[(
    "scripts.run",
    "Executes a function in an Apps Script project (Apps Script API script.run). The script \
         runs headlessly with the scopes its manifest declares — review them before calling: \
         arbitrary script code executes with your authorization. Returns the script function's \
         response (OPS_ERROR / ops for failures); script-side failures come back as tool errors, \
         not dead servers. Prefer `script processes list` to observe running executions and \
         `script projects get` to read a project's source before running it.",
)];

/// The resources this method's id sits among, as sibling guidance: the
/// other verbs over the same resource path, so an agent sees the CRUD
/// neighbourhood without leaving the tool list.
fn sibling_summary(service: &Service, method: &Method) -> String {
    // The method id is `resource.path.segments.verb`; siblings share every
    // segment but the last.
    let verb = method.id.rsplit('.').next().unwrap_or("");
    let resource = &method.id[..method.id.len().saturating_sub(verb.len()).saturating_sub(1)];
    if resource.is_empty() {
        return String::new();
    }
    let mut siblings: Vec<&str> = service
        .methods
        .iter()
        .filter(|m| {
            m.id != method.id
                && m.id
                    .strip_suffix(&format!(".{}", m.id.rsplit('.').next().unwrap_or("")))
                    == Some(resource)
        })
        .map(|m| m.id.rsplit('.').next().unwrap_or(""))
        .collect();
    siblings.sort_unstable();
    siblings.dedup();
    if siblings.is_empty() {
        String::new()
    } else {
        format!(" Siblings over the same resource: {}.", siblings.join(", "))
    }
}

/// What the call returns, derived from the verb — the shape the shared
/// call path actually delivers (JSON on stdout, envelope on MCP).
fn returns_summary(method: &Method) -> &'static str {
    match method.http_method.to_ascii_uppercase().as_str() {
        "GET" => "Returns the requested resource as JSON",
        "DELETE" => "Returns the post-delete resource state (or empty) as JSON",
        _ => "Returns the API's response resource as JSON",
    }
}

/// The method's Discovery description, enriched with what an agent needs
/// to pick and use the tool correctly: the resource it acts on, what comes
/// back, how the request body is built, the auth/safety story, and the
/// sibling methods over the same resource. Everything here is derived from
/// the Discovery index — no hand-written per-method prose to rot.
fn tool_description(service: &Service, method: &Method) -> String {
    let verb = method.http_method.to_ascii_uppercase();
    let base = DESCRIPTION_OVERRIDES
        .iter()
        .find(|(id, _)| *id == method.id)
        .map(|(_, text)| text.to_string())
        .unwrap_or_else(|| {
            if method.description.is_empty() {
                format!(
                    "{}s {} via the {} API",
                    verb.to_ascii_lowercase(),
                    method.id.rsplit('.').next().unwrap_or("resource"),
                    service.name
                )
            } else {
                method.description.clone()
            }
        });

    // Discovery descriptions commonly end in a period; normalize so we never
    // ship "text.. [HTTP".
    let base = base.trim_end_matches('.');
    let mut text = format!("{base} [HTTP {verb}]");

    // Behavior: what the call does to the world, before the auth line.
    match verb.as_str() {
        "POST" => text.push_str(" — creates or triggers a change"),
        "DELETE" => text.push_str(
            " — removes the resource; irreversible unless the API documents a trash/undo flow",
        ),
        "PATCH" | "PUT" => text.push_str(" — updates the resource"),
        _ => {}
    }
    if is_write_verb(&verb) {
        text.push_str("; refused under --readonly safety profiles");
    }

    // Completeness: the two things the schema cannot say.
    // Discovery text can carry its own trailing period mid-sentence; don't
    // let our appended clause double it. Trailing-period truncation (300
    // chars) at distill time also leaves a bare period to normalize.
    if text.ends_with(' ') {
        text.pop();
    }
    match &method.request_ref {
        Some(body_type) => {
            let fields = request_body_fields(service, body_type);
            text.push_str(&format!(
                ". {} — request body is a JSON {body_type} object{fields}; pass it via `params` or whole via `body_file`",
                returns_summary(method)
            ));
        }
        None => text.push_str(&format!(
            ". {} — request bodies are built from the typed parameters (typed flags win over the `params` map), or read whole from `body_file`",
            returns_summary(method)
        )),
    }

    text.push_str(&sibling_summary(service, method));
    text
}

/// The request schema's top-level field names, for the description — an
/// agent choosing parameters sees what the body accepts without opening
/// the schema. Long lists truncate at eight names.
fn request_body_fields(service: &Service, schema_name: &str) -> String {
    let Some(schema) = service.schemas.get(schema_name) else {
        return String::new();
    };
    let Some(properties) = &schema.properties else {
        return String::new();
    };
    let mut names: Vec<&String> = properties.keys().collect();
    names.sort_unstable();
    if names.is_empty() {
        return String::new();
    }
    if names.len() > 8 {
        format!(
            " accepting {} and {} more fields",
            names[..8]
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(", "),
            names.len() - 8
        )
    } else {
        format!(
            " accepting {}",
            names
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

/// The tool's input schema: a JSON-schema object built from the method's
/// parameters plus grr's escape hatches; `required` is the method's
/// required parameter names.
fn input_schema(service: &Service, method: &Method) -> Value {
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
    let body_file_desc = match &method.request_ref {
        Some(name) => format!(
            "Read the request body from this file ('-' for stdin) instead of building one from the parameters. The body is a JSON {name} object."
        ),
        None => "Read the request body from this file ('-' for stdin) instead of building one from the parameters.".to_owned(),
    };
    properties.insert(
        "body_file".into(),
        json!({
            "type": "string",
            "description": body_file_desc,
        }),
    );
    properties.insert("query".into(), json!({
        "type": "array",
        "items": { "type": "string" },
        "description": "Extra KEY=VALUE query pairs for endpoints whose parameters Discovery does not describe.",
    }));
    // The request-body shape, when the upstream method names one: agents
    // see the field names to build (params map keys or body_file content).
    if let Some(name) = &method.request_ref
        && let Some(schema) = service.schemas.get(name)
    {
        let mut body_schema = serde_json::to_value(schema).expect("Schema serializes");
        if let Some(obj) = body_schema.as_object_mut() {
            obj.insert(
                "description".into(),
                json!(format!("Request body of type {name}: pass it via `params` (merged into the built body) or whole via `body_file`.")),
            );
        }
        properties.insert("request_body".into(), body_schema);
    }
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
    if let Some(format) = &param.format {
        // Google's format names (int32, uint32, google-fieldmask,
        // google-datetime) pass through verbatim — close enough for a
        // consumer and honest about their origin.
        out.insert("format".into(), json!(format));
    }
    if let Some(minimum) = param.minimum {
        out.insert("minimum".into(), json!(minimum));
    }
    if let Some(maximum) = param.maximum {
        out.insert("maximum".into(), json!(maximum));
    }
    if let Some(pattern) = &param.pattern {
        out.insert("pattern".into(), json!(pattern));
    }
    if let Some(default) = &param.default_value {
        out.insert("default".into(), json!(default));
    }
    let mut description = String::new();
    if let Some(location) = &param.location {
        let hint = match location.as_str() {
            "path" => "URL path segment",
            "query" => "query parameter",
            other => other,
        };
        description.push_str(&format!("[{hint}] "));
    }
    if !param.description.is_empty() {
        description.push_str(&param.description);
    }
    if let (Some(values), Some(texts)) = (&param.enum_values, &param.enum_descriptions)
        && values.len() == texts.len()
        && !texts.is_empty()
    {
        let pairs: Vec<String> = values
            .iter()
            .zip(texts)
            .map(|(v, t)| format!("`{v}` = {t}"))
            .collect();
        if !description.is_empty() {
            description.push(' ');
        }
        description.push_str(&format!("Values: {}.", pairs.join("; ")));
    }
    if !description.is_empty() {
        out.insert("description".into(), json!(description));
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
            ..Default::default()
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
    async fn the_stdio_loop_pipes_the_stateless_requests() {
        let server = test_server().await;
        let input = format!(
            "{}\n{}\n{}\n",
            request(1, "server/discover", "{}"),
            r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#,
            request(2, "tools/list", "{}"),
        );
        let mut out: Vec<u8> = Vec::new();
        serve(&server, input.as_bytes(), &mut out).await.unwrap();

        // One response line per REQUEST; the notification got none.
        let lines: Vec<Value> = String::from_utf8(out)
            .unwrap()
            .lines()
            .map(|l| serde_json::from_str(l).unwrap())
            .collect();
        assert_eq!(lines.len(), 2);

        assert_eq!(lines[0]["jsonrpc"], "2.0");
        assert_eq!(lines[0]["id"], 1);
        assert_eq!(lines[0]["result"]["resultType"], "complete");
        assert_eq!(
            lines[0]["result"]["supportedVersions"],
            json!(["2026-07-28"])
        );
        assert!(lines[0]["result"]["capabilities"]["tools"].is_object());
        assert_eq!(
            lines[0]["result"]["_meta"]["io.modelcontextprotocol/serverInfo"]["name"],
            "grr"
        );

        assert_eq!(lines[1]["id"], 2);
        let tools = lines[1]["result"]["tools"].as_array().unwrap();
        assert_eq!(tools.len(), server.tool_count());
    }

    /// The handshake revisions are not served at all — except `initialize`,
    /// which gets a minimal legacy handshake so standard clients connect.
    /// `ping` and `logging/setLevel` get an error naming the revision to
    /// speak and the RPC that replaced the handshake.
    #[tokio::test]
    async fn legacy_handshake_methods_are_refused_with_the_revision_to_speak() {
        let server = test_server().await;

        for method in ["ping", "logging/setLevel"] {
            let line = request(1, method, "{}");
            let response = server.handle_line(&line).await.unwrap();

            assert_eq!(response["error"]["code"], METHOD_NOT_FOUND, "{method}");
        }
    }

    /// `initialize` compat: a standard client opens with the legacy
    /// handshake and must get a valid result — server identity, tool
    /// capability, and the revision it asked for echoed back (several
    /// clients reject a newer one) — instead of an error.
    #[tokio::test]
    async fn initialize_answers_a_minimal_legacy_handshake() {
        let server = test_server().await;

        let line = request(
            1,
            "initialize",
            r#"{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"probe","version":"0"}}"#,
        );
        let response = server.handle_line(&line).await.unwrap();
        let result = &response["result"];
        assert_eq!(result["protocolVersion"], "2025-06-18");
        assert_eq!(result["serverInfo"]["name"], "grr");
        assert!(result["capabilities"]["tools"].is_object());
        // The handshake completes; tools/* then works as usual.
        let line = request(2, "tools/list", "{}");
        let response = server.handle_line(&line).await.unwrap();
        assert_eq!(
            response["result"]["tools"].as_array().unwrap().len(),
            server.tool_count()
        );
    }

    /// The stateless revision has no handshake, so `server/discover` may be
    /// a client's first call — and `tools/list` works with no prior call at
    /// all. The server holds no handshake state to satisfy.
    #[tokio::test]
    async fn server_discover_answers_without_a_handshake() {
        let server = test_server().await;

        let line = request(
            1,
            "server/discover",
            r#"{"_meta":{"io.modelcontextprotocol/protocolVersion":"2026-07-28","io.modelcontextprotocol/clientInfo":{"name":"probe","version":"0"}}}"#,
        );
        let response = server.handle_line(&line).await.unwrap();
        let result = &response["result"];

        assert_eq!(result["resultType"], "complete");
        assert_eq!(
            result["supportedVersions"],
            json!(["2026-07-28"]),
            "the current revision, and only it"
        );
        assert!(result["capabilities"]["tools"].is_object());
        assert_eq!(
            result["_meta"]["io.modelcontextprotocol/serverInfo"]["name"],
            "grr"
        );

        // A 2026-07-28 client may also skip discovery entirely.
        let inline = request(2, "tools/list", "{}");
        let response = server.handle_line(&inline).await.unwrap();
        assert!(response["result"]["tools"].is_array());
    }

    /// No handshake means no `serverInfo` to carry identity, so every result
    /// names the server — the spec asks for `_meta` on each one.
    #[tokio::test]
    async fn results_identify_the_server_per_request() {
        let server = test_server().await;

        for method in ["tools/list", "server/discover"] {
            let line = request(1, method, "{}");
            let response = server.handle_line(&line).await.unwrap();
            assert_eq!(
                response["result"]["_meta"]["io.modelcontextprotocol/serverInfo"]["name"], "grr",
                "{method} did not identify the server"
            );
        }
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

    /// Tools whose upstream method documents a response schema declare an
    /// `outputSchema`; its shape mirrors the pruned Discovery schema (field
    /// names, types, array items). Tools without one omit the key rather
    /// than fake a shape.
    #[tokio::test]
    async fn output_schema_mirrors_the_response_ref() {
        let server = test_server().await;
        let line = request(2, "tools/list", "{}");
        let response = server.handle_line(&line).await.unwrap();
        let tools = response["result"]["tools"].as_array().unwrap();

        let find = |name: &str| {
            tools
                .iter()
                .find(|t| t["name"] == name)
                .unwrap_or_else(|| panic!("{name} missing"))
        };

        // gmail.users.threads.list responds with ListThreadsResponse.
        let list = find("gmail.users.threads.list");
        let output = &list["outputSchema"];
        assert!(
            output.is_object(),
            "threads.list must carry an outputSchema"
        );
        assert_eq!(output["type"], "object");
        assert!(
            output["properties"]["nextPageToken"]["type"] == "string",
            "list envelope fields survive distillation"
        );
        assert!(
            output["properties"]["threads"]["items"]["properties"]["id"]["type"] == "string",
            "nested array items are inlined one $ref deep"
        );

        // Methods whose upstream document names no response omit the key.
        let empty = find("gmail.users.stop");
        assert!(
            empty.get("outputSchema").is_none(),
            "no fabricated output schema"
        );

        // Schemas are pruned: no description keys anywhere in the output.
        let raw = serde_json::to_string(&response).unwrap();
        assert!(
            !raw.contains("\"outputSchema\":{\"description"),
            "pruned schemas carry no descriptions"
        );
    }

    /// Methods whose upstream document names a request body get its shape
    /// as a `request_body` schema (field names an agent can build) and a
    /// description clause naming the body type; tools without one keep the
    /// generic body_file wording.
    #[tokio::test]
    async fn request_body_schemas_carry_the_body_shape_and_type_name() {
        let server = test_server().await;
        let line = request(2, "tools/list", "{}");
        let response = server.handle_line(&line).await.unwrap();
        let tools = response["result"]["tools"].as_array().unwrap();
        let find = |name: &str| {
            tools
                .iter()
                .find(|t| t["name"] == name)
                .unwrap_or_else(|| panic!("{name} missing"))
        };

        let query = find("searchconsole.searchanalytics.query");
        let body = &query["inputSchema"]["properties"]["request_body"];
        assert!(body.is_object(), "query must expose its request body shape");
        assert_eq!(body["type"], "object");
        assert!(
            body["properties"]["startDate"].is_object(),
            "body field names survive distillation"
        );
        let description = query["description"].as_str().unwrap();
        assert!(
            description.contains("SearchAnalyticsQueryRequest"),
            "description names the body type: {description}"
        );
        assert!(
            description.contains("startDate"),
            "description lists top-level body fields: {description}"
        );

        // A method without a request ref: no request_body property, generic
        // body_file wording.
        let get = find("gmail.users.threads.list");
        assert!(
            get["inputSchema"]["properties"]
                .get("request_body")
                .is_none(),
            "no fabricated request body"
        );
        let body_file = get["inputSchema"]["properties"]["body_file"]["description"]
            .as_str()
            .unwrap();
        assert!(
            !body_file.contains("JSON"),
            "generic wording when no body type exists: {body_file}"
        );
    }

    /// Parameter schemas surface what Discovery documents beyond the type:
    /// location (path segment vs query), enum-value prose, format, default,
    /// and bounds.
    #[tokio::test]
    async fn parameter_schemas_carry_location_enums_format_and_bounds() {
        let server = test_server().await;
        let line = request(2, "tools/list", "{}");
        let response = server.handle_line(&line).await.unwrap();
        let tools = response["result"]["tools"].as_array().unwrap();
        let find = |name: &str| {
            tools
                .iter()
                .find(|t| t["name"] == name)
                .unwrap_or_else(|| panic!("{name} missing"))
        };

        let user_id =
            find("gmail.users.threads.list")["inputSchema"]["properties"]["userId"]["description"]
                .as_str()
                .unwrap();
        assert!(
            user_id.starts_with("[URL path segment]"),
            "path parameters announce their location: {user_id}"
        );

        // calendar.events.list maxResults: format int32, default 250, min 1.
        let max_results = find("calendar.events.list")["inputSchema"]["properties"]["maxResults"]
            .as_object()
            .expect("maxResults present");
        assert_eq!(max_results["format"], "int32");
        assert_eq!(max_results["minimum"], 1.0);
        assert_eq!(max_results["default"], "250");
    }

    /// Descriptions carry what an agent needs to pick a tool: what the call
    /// returns, how the request body is built, and — where the resource has
    /// siblings — the CRUD neighbourhood. Also covers the curated overrides:
    /// Discovery methods that ship no description still get real prose.
    #[tokio::test]
    async fn descriptions_carry_returns_body_and_siblings() {
        let server = test_server().await;
        let line = request(2, "tools/list", "{}");
        let response = server.handle_line(&line).await.unwrap();
        let tools = response["result"]["tools"].as_array().unwrap();
        let by_name = |name: &str| {
            tools
                .iter()
                .find(|t| t["name"] == name)
                .unwrap_or_else(|| panic!("{name} missing"))["description"]
                .as_str()
                .unwrap()
                .to_owned()
        };

        let delete = by_name("gmail.users.messages.delete");
        assert!(
            delete.contains("irreversible"),
            "delete should warn about irreversibility: {delete}"
        );
        assert!(
            delete.contains("refused under --readonly"),
            "write verbs name the safety gate: {delete}"
        );

        let list = by_name("calendar.events.list");
        assert!(
            list.contains("Siblings over the same resource"),
            "list tools name their CRUD neighbours: {list}"
        );
        assert!(list.contains("as JSON"), "returns are described: {list}");
        assert!(
            list.contains("body_file"),
            "body building is described: {list}"
        );

        let run = by_name("script.scripts.run");
        assert!(
            run.starts_with("Executes a function"),
            "Discovery shipped no description for scripts.run; the curated override must supply one: {run}"
        );

        // Our appended clauses never start with a period directly after the
        // upstream text's own: the verb tag always follows cleanly.
        for tool in tools {
            let d = tool["description"].as_str().unwrap();
            assert!(
                !d.contains(". [HTTP"),
                "{}: doubled period before the verb tag in {d}",
                tool["name"]
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
        let server = test_server_with_profile(SafetyProfile::new(true, [], [])).await;

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
