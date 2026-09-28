//! `grr api` — the discovery-driven escape hatch.
//!
//! `grr api call <id>` reaches every method Google publishes across Gmail,
//! Calendar, Drive, People, Chat, Forms, Tasks, Docs, Sheets and Slides —
//! including anything released after grr was built, with no code change
//! and no new release. The generated service tree
//! (`src/commands/generated.rs`, dispatched by `gen_dispatch.rs`) is the
//! same engine with a friendlier shape: both funnel through
//! [`call_method`], so a dry-run of the same method and parameters is
//! byte-identical either way.
//!
//! ```text
//! grr api list                                    # every method
//! grr api list gmail --filter list                # just Gmail listing methods
//! grr api describe gmail.users.messages.list      # params + scopes
//! grr api call gmail.users.messages.list --param userId=me --param 'q=is:unread'
//! grr api call drive.files.list --params '{"pageSize":10}'
//! grr gmail users messages list --user-id me      # the same call, generated
//! ```
//!
//! Authorisation is per method: grr asks Google only for the scopes the
//! method you are calling declares. Requesting every Workspace scope up
//! front would exceed the ~25-scope ceiling on unverified apps and fail at
//! consent, so a narrow call stays narrow.

use crate::commands::safety::SafetyProfile;
use crate::core::auth::GoogleAuth;
use crate::discovery::{self, Method, Service};
use crate::output::{OutputFormat, print_output};
use anyhow::{Context, Result, bail};
use clap::{Args, Subcommand};
use serde_json::{Value, json};

#[derive(Subcommand, Debug)]
pub enum ApiCommands {
    /// List every method the Discovery index knows about
    List(ApiListArgs),
    /// Show one method's parameters, scopes and documentation
    Describe(ApiDescribeArgs),
    /// Invoke a method
    Call(ApiCallArgs),
    /// Fetch fresh Discovery documents into the local cache. The embedded
    /// index keeps working offline; this pulls it forward between releases.
    Refresh(ApiRefreshArgs),
}

#[derive(Args, Debug)]
pub struct ApiRefreshArgs {
    /// Refresh one service only (gmail, calendar, drive, people, chat,
    /// forms, tasks, docs, sheets, slides)
    #[arg(long)]
    pub service: Option<String>,

    /// Output format
    #[arg(short, long, value_enum, default_value = "json")]
    pub format: OutputFormat,
}

#[derive(Args, Debug)]
pub struct ApiListArgs {
    /// Limit to one service (gmail, calendar, drive, people, chat, forms,
    /// tasks, docs, sheets, slides)
    #[arg(long)]
    pub service: Option<String>,

    /// Case-insensitive substring filter over method ids and descriptions
    #[arg(long)]
    pub filter: Option<String>,

    /// Group output by service instead of one flat list
    #[arg(long)]
    pub grouped: bool,

    /// Output format
    #[arg(short, long, value_enum, default_value = "json")]
    pub format: OutputFormat,
}

#[derive(Args, Debug)]
pub struct ApiDescribeArgs {
    /// Method id, e.g. gmail.users.messages.list
    pub method: String,

    /// Output format
    #[arg(short, long, value_enum, default_value = "json")]
    pub format: OutputFormat,
}

#[derive(Args, Debug)]
pub struct ApiCallArgs {
    /// Method id, e.g. gmail.users.messages.list
    pub method: String,

    /// Parameters as a JSON object. Values for path placeholders are
    /// substituted into the URL; the rest become query parameters, or the
    /// request body for POST/PATCH/PUT.
    #[arg(long, value_name = "JSON")]
    pub params: Option<String>,

    /// A single parameter as key=value. Repeatable. Use for values that are
    /// awkward to quote in JSON.
    #[arg(long = "param", value_name = "KEY=VALUE")]
    pub param: Vec<String>,

    /// Read the request body from a file ("-" for stdin) instead of
    /// building one from --params. Sends the bytes verbatim.
    #[arg(long, value_name = "PATH")]
    pub body_file: Option<String>,

    /// Override the HTTP method from the Discovery document
    #[arg(long, value_name = "VERB")]
    pub method_override: Option<String>,

    /// Extra fields to merge into the query string, for endpoints whose
    /// parameters Discovery does not describe (alt=json, fields, key)
    #[arg(long = "query", value_name = "KEY=VALUE")]
    pub query: Vec<String>,

    /// Print the request that would be sent without sending it
    #[arg(long)]
    pub dry_run: bool,

    // NOTE: the safety flags (--readonly, --deny-service, --deny-verb) are
    // NOT redeclared here: they are global args on the root command
    // (cli.rs), and a second declaration with the same IDs would panic
    // clap's arg-matching. Global args propagate into this subcommand's
    // matches, so from_arg_matches still fills them.
    /// Output format
    #[arg(short, long, value_enum, default_value = "json")]
    pub format: OutputFormat,
}

pub async fn handle_api_cmd(
    auth: &GoogleAuth,
    cmd: ApiCommands,
    profile: &SafetyProfile,
) -> Result<()> {
    match cmd {
        ApiCommands::List(args) => handle_list(args),
        ApiCommands::Describe(args) => handle_describe(args),
        ApiCommands::Call(args) => handle_call(auth, args, profile).await,
        // The refresh needs no credential: Discovery documents are public.
        ApiCommands::Refresh(args) => handle_refresh(args).await,
    }
}

async fn handle_refresh(args: ApiRefreshArgs) -> Result<()> {
    let reports = crate::discovery::refresh(args.service.as_deref())
        .await
        .map_err(|message| anyhow::anyhow!("{message}"))?;

    let refreshed = reports.iter().filter(|r| r.wrote_cache).count();
    let skipped = reports.iter().filter(|r| r.skipped_older).count();
    // stdout stays machine-readable; the summary goes to stderr.
    eprintln!("{refreshed} refreshed, {skipped} already current");
    print_output(
        &serde_json::json!({
            "refreshed": refreshed,
            "alreadyCurrent": skipped,
            "services": reports.iter().map(|r| serde_json::json!({
                "service": r.service,
                "from": r.from,
                "to": r.to,
                "methods": r.methods,
                "wroteCache": r.wrote_cache,
                "skippedOlder": r.skipped_older,
            })).collect::<Vec<_>>(),
            "nextStep": "restart grr (each invocation is a fresh process) to use the refreshed index",
        }),
        args.format,
    )?;
    Ok(())
}

fn matches_filter(method: &Method, needle: &str) -> bool {
    let needle = needle.to_ascii_lowercase();
    method.id.to_ascii_lowercase().contains(&needle)
        || method.description.to_ascii_lowercase().contains(&needle)
}

fn handle_list(args: ApiListArgs) -> Result<()> {
    let all = discovery::services();
    let selected: Vec<(&&str, &Service)> = match &args.service {
        Some(name) => {
            let service = all.get_key_value(name.as_str()).ok_or_else(|| {
                let known: Vec<&str> = all.keys().copied().collect();
                anyhow::anyhow!("unknown service `{name}`; known: {}", known.join(", "))
            })?;
            vec![service]
        }
        None => all.iter().collect(),
    };

    let filtered: Vec<(&&str, &Service, Vec<&Method>)> = selected
        .into_iter()
        .map(|(name, service)| {
            let methods: Vec<&Method> = match &args.filter {
                Some(needle) => service
                    .methods
                    .iter()
                    .filter(|m| matches_filter(m, needle))
                    .collect(),
                None => service.methods.iter().collect(),
            };
            (name, service, methods)
        })
        .filter(|(_, _, methods)| !methods.is_empty())
        .collect();

    if args.grouped {
        let payload: Vec<Value> = filtered
            .iter()
            .map(|(_, service, methods)| {
                json!({
                    "service": service.name,
                    "version": service.version,
                    "revision": service.revision,
                    "count": methods.len(),
                    "methods": methods.iter().map(|m| json!({
                        "id": format!("{}.{}", service.name, m.id),
                        "http": m.http_method,
                    })).collect::<Vec<_>>(),
                })
            })
            .collect();
        print_output(&Value::Array(payload), args.format)?;
        return Ok(());
    }

    let total: usize = filtered.iter().map(|(_, _, m)| m.len()).sum();
    let payload: Vec<Value> = filtered
        .iter()
        .flat_map(|(_, service, methods)| {
            methods.iter().map(move |m| {
                json!({
                    "id": format!("{}.{}", service.name, m.id),
                    "http": m.http_method,
                    "summary": first_sentence(&m.description),
                })
            })
        })
        .collect();

    // stderr keeps stdout pure JSON so `grr api list | jq` works.
    eprintln!("{total} methods");
    print_output(&Value::Array(payload), args.format)?;
    Ok(())
}

fn first_sentence(text: &str) -> String {
    match text.find(". ") {
        Some(idx) => text[..idx + 1].to_owned(),
        None => text.chars().take(120).collect(),
    }
}

fn handle_describe(args: ApiDescribeArgs) -> Result<()> {
    let (service, method) =
        discovery::resolve(&args.method).map_err(|message| anyhow::anyhow!(message))?;
    print_output(
        &json!({
            "id": format!("{}.{}", service.name, method.id),
            "service": service.name,
            "apiVersion": service.version,
            "discoveryRevision": service.revision,
            "httpMethod": method.http_method,
            "path": method.path,
            // The raw template, not a rendered URL: placeholders are the
            // point, and the caller can see which values are required.
            "urlTemplate": format!(
                "{}{}",
                service.root_url.clone().unwrap_or_default(),
                service.base()
            ) + &method.path,
            "description": method.description,
            "scopes": method.scopes,
            "leastPrivilegeScope": method.least_privilege_scope(),
            "parameters": method.parameters.iter().map(|p| json!({
                "name": p.name,
                "type": p.kind,
                "required": p.required,
                "repeated": p.repeated,
                "description": p.description,
                "enum": p.enum_values,
            })).collect::<Vec<_>>(),
            "hint": format!(
                "grr api call {}.{} --params '{{\"{}\": \"...\"}}'",
                service.name, method.id,
                method.parameters.first().map(|p| p.name.as_str()).unwrap_or("userId")
            ),
        }),
        args.format,
    )?;
    Ok(())
}

async fn handle_call(auth: &GoogleAuth, args: ApiCallArgs, profile: &SafetyProfile) -> Result<()> {
    let (service, method) = discovery::resolve(&args.method).map_err(|message| {
        anyhow::anyhow!(
            "{message}\n\nRun `grr api list {}` to see what is available.",
            service_hint(&args.method)
        )
    })?;

    // The safety gate runs before anything is built or sent — the same
    // gate the `grr mcp` server honors (src/commands/safety.rs). The
    // profile comes from the global --readonly/--deny-service/--deny-verb
    // flags, parsed once in cli.rs.
    if let Err(message) = profile.check(&service.name, method) {
        bail!("{message}");
    }

    let params = parse_params(args.params.as_deref(), &args.param)?;
    let payload = call_method(
        auth,
        service,
        method,
        params,
        CallOptions {
            body_file: args.body_file,
            verb_override: args.method_override,
            query_extras: args.query,
            dry_run: args.dry_run,
        },
    )
    .await?;
    print_output(&payload, args.format)?;
    Ok(())
}

/// The caller-controlled request shaping beyond the parameter map. All
/// optional; `grr api call` fills everything, the generated tree leaves
/// `verb_override` at `None`, `auth status` leaves almost all of it.
#[derive(Debug, Default)]
pub(crate) struct CallOptions {
    /// Read the request body from this file ("-" = stdin) instead of
    /// synthesising one from the parameters.
    pub body_file: Option<String>,
    /// Override the HTTP method from the Discovery document
    /// (`grr api call --method-override`).
    pub verb_override: Option<String>,
    /// Extra KEY=VALUE query pairs for undocumented parameters.
    pub query_extras: Vec<String>,
    /// Print the request that would be sent without sending it.
    pub dry_run: bool,
}

/// Everything needed to issue (or dry-run) one Discovery method call.
/// The URL is fully built; the body is a parsed JSON value (or the raw
/// file/stdin bytes wrapped as a JSON string when they were not JSON).
#[derive(Debug)]
pub(crate) struct RequestPlan {
    pub verb: String,
    pub url: String,
    pub body: Option<Value>,
}

/// Validate parameters against the method and build the request. Pure:
/// no network, no credentials — which is what keeps the dry-run output
/// and the parity tests honest.
///
/// `query_extras` are appended after the Discovery-derived query pairs,
/// for endpoints whose parameters Discovery does not describe
/// (`alt=json`, `fields`, `key`).
pub(crate) fn plan_request(
    service: &Service,
    method: &Method,
    params: &serde_json::Map<String, Value>,
    body_file: Option<&str>,
    verb_override: Option<&str>,
    query_extras: &[String],
) -> Result<RequestPlan> {
    // Warn loudly rather than letting Google return a confusing 400.
    for required in &method.required {
        if !params.contains_key(required) {
            let known: Vec<String> = method
                .parameters
                .iter()
                .filter(|p| p.required)
                .map(|p| format!("{} ({})", p.name, p.kind))
                .collect();
            bail!(
                "missing required parameter `{required}` for {}.{}\nRequired: {}",
                service.name,
                method.id,
                if known.is_empty() {
                    "none".to_owned()
                } else {
                    known.join(", ")
                }
            );
        }
    }

    let verb = verb_override
        .unwrap_or(&method.http_method)
        .to_ascii_uppercase();
    let path = discovery::build_path(service, method, params).map_err(|m| anyhow::anyhow!(m))?;
    let mut query = discovery::build_query(method, params);
    for extra in query_extras {
        let Some((k, v)) = extra.split_once('=') else {
            bail!("--query expects KEY=VALUE, got `{extra}`");
        };
        if !query.is_empty() {
            query.push('&');
        }
        query.push_str(&format!(
            "{}={}",
            urlencoding::encode(k),
            urlencoding::encode(v)
        ));
    }
    let url = format!(
        "{}{}{}",
        service.root_url.clone().unwrap_or_default(),
        path,
        query
    );

    // With a body file, the caller's bytes win; otherwise synthesise a body
    // from the non-path, non-query parameters for verbs that take one.
    let body: Option<Value> = match body_file {
        Some(source) => {
            let raw = if source == "-" {
                use std::io::Read;
                let mut buf = String::new();
                std::io::stdin().read_to_string(&mut buf)?;
                buf
            } else {
                std::fs::read_to_string(source)
                    .with_context(|| format!("reading body from {source}"))?
            };
            Some(serde_json::from_str(&raw).unwrap_or(Value::String(raw)))
        }
        None => {
            let placeholders = method.path_placeholders();
            let is_write = matches!(verb.as_str(), "POST" | "PATCH" | "PUT");
            if !is_write {
                None
            } else {
                let mut rest = serde_json::Map::new();
                for (key, value) in params {
                    if placeholders.contains(&key.as_str()) {
                        continue;
                    }
                    if let Some(param) = method.parameter(key)
                        && param.location.as_deref() == Some("path")
                    {
                        continue;
                    }
                    rest.insert(key.clone(), value.clone());
                }
                (!rest.is_empty()).then_some(Value::Object(rest))
            }
        }
    };

    Ok(RequestPlan { verb, url, body })
}

/// The `--dry-run` payload for a planned request. Shared by `grr api call`
/// and the generated tree so both print byte-identical output.
pub(crate) fn dry_run_payload(method: &Method, plan: &RequestPlan) -> Value {
    json!({
        "method": plan.verb,
        "url": plan.url,
        "body": plan.body,
        "scopesRequested": method.scopes,
        "leastPrivilegeScope": method.least_privilege_scope(),
        "dryRun": true,
    })
}

/// THE shared call path: `grr api call`, the generated service tree and
/// `grr auth status` all issue Discovery methods through this function.
///
/// Returns the payload for the caller to print with its chosen format: a
/// dry-run returns its plan payload, a live call the parsed response body.
pub(crate) async fn call_method(
    auth: &GoogleAuth,
    service: &Service,
    method: &Method,
    params: serde_json::Map<String, Value>,
    options: CallOptions,
) -> Result<Value> {
    let plan = plan_request(
        service,
        method,
        &params,
        options.body_file.as_deref(),
        options.verb_override.as_deref(),
        &options.query_extras,
    )?;

    if options.dry_run {
        return Ok(dry_run_payload(method, &plan));
    }

    // Authorise what this method declares. grr's credential is consented
    // for the union in SCOPES; a method needing something outside that set
    // gets a clear message rather than a bare 403.
    let needed = method.least_privilege_scope().unwrap_or_default();
    if !needed.is_empty() && !crate::core::auth::SCOPES.contains(&needed) {
        eprintln!(
            "note: {}.{} wants `{needed}`, which is outside the scopes this build consented to. \
             Attempting anyway; if Google answers 403, re-run `grr auth login` after extending the scope set.",
            service.name, method.id
        );
    }
    let token = auth.get_access_token().await?;

    let parsed_url = url::Url::parse(&plan.url).with_context(|| {
        format!(
            "`{}` is not a valid URL (Discovery path + parameters)",
            plan.url
        )
    })?;

    let mut request = auth.http().request(
        reqwest::Method::from_bytes(plan.verb.as_bytes()).unwrap_or(reqwest::Method::GET),
        parsed_url,
    );
    if let Some(payload) = &plan.body {
        request = request.json(payload);
    }
    let response = request
        .bearer_auth(token)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("{} {} failed: {e}", plan.verb, plan.url))?;

    let status = response.status();
    let text = response.text().await.unwrap_or_default();
    let parsed: Value = serde_json::from_str(&text).unwrap_or(Value::String(text.clone()));

    if !status.is_success() {
        // 403 with insufficient_scope is the common case when a method
        // needs a scope this build did not request; name it.
        if status.as_u16() == 403
            && let Some(scope) = method.least_privilege_scope()
        {
            bail!(
                "{} {} -> HTTP 403: this method needs the `{scope}` scope, \
                 which this build was not consented for.\n\
                 Every other scope grr holds is working; re-consent with that scope added \
                 to `SCOPES` in src/core/auth/mod.rs and rebuild.",
                plan.verb,
                plan.url
            );
        }
        bail!(
            "{} {} -> HTTP {}\n{}",
            plan.verb,
            plan.url,
            status.as_u16(),
            serde_json::to_string_pretty(&parsed).unwrap_or(text)
        );
    }

    Ok(parsed)
}

fn service_hint(method_id: &str) -> &str {
    method_id.split('.').next().unwrap_or("gmail")
}

/// Merge `--params` JSON with KEY=VALUE pairs (and, in the generated tree,
/// with the typed flags). Shared with the generated tree's dispatch: the
/// later, more specific source always wins, and values that parse as JSON
/// keep their real type so `--param pageSize=10` is a number.
pub(crate) fn parse_params(
    json_blob: Option<&str>,
    pairs: &[String],
) -> Result<serde_json::Map<String, Value>> {
    let mut map = serde_json::Map::new();

    if let Some(blob) = json_blob {
        let parsed: Value = serde_json::from_str(blob)
            .with_context(|| "--params must be a JSON object, e.g. '{\"userId\":\"me\"}'")?;
        let Value::Object(object) = parsed else {
            bail!("--params must be a JSON object, got {}", type_name(&parsed));
        };
        map.extend(object);
    }

    for pair in pairs {
        let Some((key, value)) = pair.split_once('=') else {
            bail!("--param expects KEY=VALUE, got `{pair}`");
        };
        // Bare `true`/`123`/`[...]` become real JSON values, so `--param
        // pageSize=10` is a number and not the string "10" — otherwise
        // every numeric parameter would need quoting.
        let coerced = serde_json::from_str::<Value>(value)
            .unwrap_or_else(|_| Value::String(value.to_owned()));
        map.insert(key.to_owned(), coerced);
    }

    Ok(map)
}

fn type_name(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "a boolean",
        Value::Number(_) => "a number",
        Value::String(_) => "a string",
        Value::Array(_) => "an array",
        Value::Object(_) => "an object",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_params_are_merged() {
        let params = parse_params(Some(r#"{"userId":"me","q":"is:unread"}"#), &[]).unwrap();
        assert_eq!(params.get("userId").unwrap(), "me");
        assert_eq!(params.get("q").unwrap(), "is:unread");
    }

    #[test]
    fn param_flags_win_over_json_params() {
        // The later, more specific source should win so a caller can
        // override one field of a large --params blob.
        let params =
            parse_params(Some(r#"{"userId":"me"}"#), &["userId=someone-else".into()]).unwrap();
        assert_eq!(params.get("userId").unwrap(), "someone-else");
    }

    #[test]
    fn param_values_are_coerced_to_json_types() {
        let params =
            parse_params(None, &["a=10".into(), "b=true".into(), "c=text".into()]).unwrap();
        assert_eq!(params.get("a").unwrap(), 10);
        assert_eq!(params.get("b").unwrap(), true);
        assert_eq!(params.get("c").unwrap(), "text");
    }

    #[test]
    fn values_containing_equals_are_preserved() {
        // Drive queries and Gmail searches contain '='; only split once.
        let params = parse_params(None, &["q=name=a.pdf".into()]).unwrap();
        assert_eq!(params.get("q").unwrap(), "name=a.pdf");
    }

    #[test]
    fn malformed_input_is_rejected_with_guidance() {
        assert!(parse_params(Some("not json"), &[]).is_err());
        assert!(parse_params(Some("[1,2]"), &[]).is_err());
        assert!(parse_params(None, &["novalue".into()]).is_err());
    }

    fn plan(id: &str, params: &[(&str, Value)]) -> (super::RequestPlan, &'static Method) {
        let (service, method) = crate::discovery::resolve(id).unwrap();
        let map: serde_json::Map<String, Value> = params
            .iter()
            .map(|(k, v)| ((*k).to_owned(), v.clone()))
            .collect();
        (
            super::plan_request(service, method, &map, None, None, &[]).unwrap(),
            method,
        )
    }

    #[test]
    fn plan_builds_the_url_from_discovery() {
        let (plan, method) = plan(
            "gmail.users.messages.list",
            &[("userId", json!("me")), ("q", json!("is:unread"))],
        );
        assert_eq!(plan.verb, "GET");
        assert_eq!(
            plan.url,
            "https://gmail.googleapis.com/gmail/v1/users/me/messages?q=is%3Aunread"
        );
        assert_eq!(plan.body, None);
        assert_eq!(method.id, "users.messages.list");
    }

    #[test]
    fn plan_rejects_a_missing_required_parameter_by_name() {
        let (service, method) = crate::discovery::resolve("gmail.users.messages.list").unwrap();
        let err = super::plan_request(service, method, &serde_json::Map::new(), None, None, &[])
            .unwrap_err()
            .to_string();
        assert!(err.contains("missing required parameter `userId`"), "{err}");
    }

    #[test]
    fn plan_appends_query_extras_after_discovery_pairs() {
        let (service, method) = crate::discovery::resolve("gmail.users.messages.list").unwrap();
        let mut params = serde_json::Map::new();
        params.insert("userId".into(), json!("me"));
        params.insert("q".into(), json!("is:unread"));
        let plan = super::plan_request(
            service,
            method,
            &params,
            None,
            None,
            &["alt=json".to_owned()],
        )
        .unwrap();
        assert!(plan.url.ends_with("q=is%3Aunread&alt=json"), "{}", plan.url);
    }

    #[test]
    fn plan_rejects_malformed_query_extras() {
        let (service, method) = crate::discovery::resolve("gmail.users.messages.list").unwrap();
        let mut params = serde_json::Map::new();
        params.insert("userId".into(), json!("me"));
        let err = super::plan_request(
            service,
            method,
            &params,
            None,
            None,
            &["no-equals-sign".to_owned()],
        )
        .unwrap_err()
        .to_string();
        assert!(err.contains("KEY=VALUE"), "{err}");
    }

    #[test]
    fn plan_uppercases_a_verb_override() {
        let (service, method) = crate::discovery::resolve("gmail.users.messages.list").unwrap();
        let mut params = serde_json::Map::new();
        params.insert("userId".into(), json!("me"));
        let plan = super::plan_request(service, method, &params, None, Some("post"), &[]).unwrap();
        assert_eq!(plan.verb, "POST");
    }

    #[test]
    fn plan_synthesizes_a_body_for_write_verbs_only() {
        // drive.files.create is a POST: leftover (non-path) params become
        // the request body, exactly as `grr api call` always did.
        let (service, method) = crate::discovery::resolve("drive.files.create").unwrap();
        let mut params = serde_json::Map::new();
        params.insert("name".into(), json!("quarterly.pdf"));
        let plan = super::plan_request(service, method, &params, None, None, &[]).unwrap();
        assert_eq!(plan.body, Some(json!({"name": "quarterly.pdf"})));

        // A GET never gets a body, even with leftover params — they ride
        // in the query string instead.
        let (service, method) = crate::discovery::resolve("gmail.users.messages.list").unwrap();
        let mut params = serde_json::Map::new();
        params.insert("userId".into(), json!("me"));
        params.insert("q".into(), json!("is:unread"));
        let plan = super::plan_request(service, method, &params, None, None, &[]).unwrap();
        assert_eq!(plan.body, None);
        assert!(plan.url.contains("q=is%3Aunread"));
    }

    #[test]
    fn plan_reads_a_body_file_verbatim() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("body.json");
        std::fs::write(&path, r#"{"name":"uploaded.pdf"}"#).unwrap();
        let (service, method) = crate::discovery::resolve("drive.files.create").unwrap();
        let plan = super::plan_request(
            service,
            method,
            &serde_json::Map::new(),
            Some(path.to_str().unwrap()),
            None,
            &[],
        )
        .unwrap();
        assert_eq!(plan.body, Some(json!({"name": "uploaded.pdf"})));

        // Non-JSON bodies are wrapped as strings, never rejected.
        let path = dir.path().join("body.txt");
        std::fs::write(&path, "not json at all").unwrap();
        let plan = super::plan_request(
            service,
            method,
            &serde_json::Map::new(),
            Some(path.to_str().unwrap()),
            None,
            &[],
        )
        .unwrap();
        assert_eq!(plan.body, Some(json!("not json at all")));
    }

    #[test]
    fn dry_run_payload_is_a_pure_function_of_the_plan() {
        let (plan, method) = plan("gmail.users.messages.list", &[("userId", json!("me"))]);
        let payload = super::dry_run_payload(method, &plan);
        assert_eq!(payload["method"], "GET");
        assert_eq!(
            payload["url"],
            "https://gmail.googleapis.com/gmail/v1/users/me/messages"
        );
        assert_eq!(payload["dryRun"], true);
        assert!(
            payload["leastPrivilegeScope"]
                .as_str()
                .unwrap()
                .ends_with("gmail.readonly")
        );
    }

    async fn test_auth() -> GoogleAuth {
        let config = crate::core::config::OAuthConfig {
            client_id: "test-id.apps.googleusercontent.com".into(),
            client_secret: None,
        };
        let storage = crate::core::auth::TokenStorage {
            access_token: "test-token".into(),
            refresh_token: None,
            expires_at: u64::MAX,
            token_type: "Bearer".into(),
            scope: String::new(),
        };
        GoogleAuth::with_token(config, storage).await.unwrap()
    }

    #[tokio::test]
    async fn handle_call_refuses_a_write_under_readonly() {
        // The gate fires before anything is built or sent, so the dummy
        // auth handle is never used.
        let auth = test_auth().await;
        let args = ApiCallArgs {
            method: "gmail.users.messages.send".into(),
            params: None,
            param: vec![],
            body_file: None,
            method_override: None,
            query: vec![],
            dry_run: true,
            format: OutputFormat::Json,
        };
        let err = handle_call(&auth, args, &SafetyProfile::readonly())
            .await
            .unwrap_err()
            .to_string();
        assert!(err.contains("POST"), "{err}");
        assert!(err.contains("--readonly"), "{err}");
        assert!(err.contains("Rerun without --readonly"), "{err}");
    }

    #[tokio::test]
    async fn handle_call_refuses_a_denied_service_and_a_denied_verb() {
        let auth = test_auth().await;
        let args = ApiCallArgs {
            method: "gmail.users.messages.list".into(),
            params: None,
            param: vec![],
            body_file: None,
            method_override: None,
            query: vec![],
            dry_run: true,
            format: OutputFormat::Json,
        };
        let profile = SafetyProfile::new(false, vec!["gmail".to_owned()], vec![]);
        let err = handle_call(&auth, args, &profile)
            .await
            .unwrap_err()
            .to_string();
        assert!(err.contains("--deny-service gmail"), "{err}");

        let args = ApiCallArgs {
            method: "gmail.users.messages.list".into(),
            params: None,
            param: vec![],
            body_file: None,
            method_override: None,
            query: vec![],
            dry_run: true,
            format: OutputFormat::Json,
        };
        let profile = SafetyProfile::new(false, vec![], vec!["get".to_owned()]);
        let err = handle_call(&auth, args, &profile)
            .await
            .unwrap_err()
            .to_string();
        assert!(err.contains("--deny-verb GET"), "{err}");
    }

    #[tokio::test]
    async fn handle_call_runs_a_read_under_readonly() {
        let auth = test_auth().await;
        let args = ApiCallArgs {
            method: "gmail.users.messages.list".into(),
            params: None,
            param: vec!["userId=me".into()],
            body_file: None,
            method_override: None,
            query: vec![],
            dry_run: true,
            format: OutputFormat::Json,
        };
        // A GET is not a write: the dry-run returns instead of refusing.
        assert!(
            handle_call(&auth, args, &SafetyProfile::readonly())
                .await
                .is_ok()
        );
    }
}
