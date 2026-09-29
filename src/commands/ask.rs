//! `grr ask` — the natural-language entry point, powered by System One
//! models (TypeSafe's Jev by default; any provider speaking the same
//! contract works by config — endpoint, model and key are all config).
//!
//! The pattern is hierarchical route-and-fill: code selects the candidate
//! sets from the embedded Discovery index, a judgment picks the intended
//! one, then typed parameters are filled — and the request is assembled
//! through the same shared call path as `grr api`:
//!
//! ```text
//! grr ask "show my unread messages"                  # plan (the default output)
//! grr ask "show my unread messages" --run            # execute
//! grr ask "..." --service gmail                      # skip the service judgment
//! grr ask "..." --method gmail.users.messages.list   # skip both, straight to params
//! ```
//!
//! Three sequential judgments, each one small HTTP request:
//!
//! 1. **Service** — a Choice over the 14 indexed services (+ `none`),
//!    criteria = what each service covers, derived from the index.
//! 2. **Method** — a Choice over the chosen service's methods (+ `none`),
//!    criteria = each method's description. Depends on 1: the answer
//!    narrows the candidate set, so the two cannot share one request.
//! 3. **Parameters** — one Choice per required parameter, all in a single
//!    request: the questions map IS the object with one field per required
//!    param, and the aggregated answers become the values. Options are the
//!    param's enum values when it declares them, else verbatim spans found
//!    in the request (plus the universal `me` alias for user-id params) —
//!    the model can only select among code-supplied candidates, never
//!    invent a value.
//!
//! Planning is the default and the plan IS the output: `--run` executes
//! through the shared call path. A low-confidence method choice
//! (configurable, default 0.6) is flagged, not blocked: the model supplies
//! the probability, code owns the behavior.

use crate::commands::api::{CallOptions, RequestPlan, call_method, plan_request};
use crate::commands::safety::SafetyProfile;
use crate::core::auth::GoogleAuth;
use crate::core::config::GrrConfig;
use crate::discovery::{self, Method, Parameter, Service};
use crate::output::{OutputFormat, print_output};
use anyhow::{Context, Result, bail};
use clap::Args;
use serde_json::{Value, json};

/// The escape hatch on every selection: "none of the candidates fits".
const NONE: &str = "none";

/// A Choice accepts a maximum of 255 options — the documented API limit
/// the option guards keep a future index honest against.
const MAX_CHOICE_OPTIONS: usize = 255;

#[derive(Args, Debug)]
pub struct AskArgs {
    /// The request in plain words, e.g. "show my unread messages"
    pub request: String,

    /// Execute the resolved request instead of printing the plan. Planning
    /// is the default: the plan IS the output.
    #[arg(long)]
    pub run: bool,

    /// Skip the service judgment: the service is already known (gmail,
    /// calendar, drive, people, chat, forms, tasks, docs, sheets, slides,
    /// script, analyticsadmin, analyticsdata, searchconsole)
    #[arg(long, value_name = "SERVICE")]
    pub service: Option<String>,

    /// Skip both routing judgments: resolve this method id directly
    /// (e.g. gmail.users.messages.list) and go straight to parameter fill
    #[arg(long, value_name = "ID")]
    pub method: Option<String>,

    /// Print the plan without sending anything (the default); kept as an
    /// explicit flag for readability, mutually exclusive with --run
    #[arg(long, conflicts_with = "run")]
    pub dry_run: bool,

    /// Output format
    #[arg(short, long, value_enum, default_value = "json")]
    pub format: OutputFormat,
}

/// The resolved System One provider: endpoint, model and key from config
/// (config file + env, defaults filled). Its own plain HTTP client, not
/// the shared grr one: that client speaks HTTP/3 prior-knowledge, which
/// only works against Google frontends — a System One provider may be
/// HTTP/1.1-only, or local (a mock in tests).
struct SystemOne {
    client: reqwest::Client,
    endpoint: String,
    model: String,
    api_key: String,
}

fn resolve_systemone(config: &GrrConfig) -> Result<SystemOne> {
    let s1 = &config.systemone;
    let api_key = s1
        .bearer_key()
        .ok_or_else(|| anyhow::anyhow!(crate::core::config::NO_API_KEY_HELP))?;
    Ok(SystemOne {
        client: reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| anyhow::anyhow!("failed to build the System One HTTP client: {e}"))?,
        endpoint: s1.endpoint_or_default().to_owned(),
        model: s1.model_or_default().to_owned(),
        api_key: api_key.to_owned(),
    })
}

impl SystemOne {
    /// The endpoint with any query string stripped: a custom provider
    /// might carry a key in the URL, and error messages must never echo it.
    fn masked_endpoint(&self) -> String {
        match self.endpoint.split_once('?') {
            Some((before, _)) => format!("{before}?…"),
            None => self.endpoint.clone(),
        }
    }
}

/// One System One request: POST `state` + typed `questions`, back the
/// `answers` map. The API key rides in the Authorization header only —
/// never in a body, never in an error message, never in the plan output.
async fn judge(s1: &SystemOne, state: Value, questions: Value) -> Result<Value> {
    let body = json!({
        "state": state,
        "model": s1.model,
        "questions": questions,
    });
    let response = s1
        .client
        .post(&s1.endpoint)
        .bearer_auth(&s1.api_key)
        .json(&body)
        .send()
        .await
        .map_err(|e| {
            anyhow::anyhow!("System One request to {} failed: {e}", s1.masked_endpoint())
        })?;

    let status = response.status();
    let text = response.text().await.unwrap_or_default();
    if !status.is_success() {
        if status.as_u16() == 429 {
            bail!(
                "System One endpoint {} is rate limited (HTTP 429); retry shortly",
                s1.masked_endpoint()
            );
        }
        bail!(
            "System One endpoint {} returned HTTP {}.\n{}",
            s1.masked_endpoint(),
            status.as_u16(),
            text
        );
    }

    let parsed: Value = serde_json::from_str(&text).with_context(|| {
        format!(
            "System One response from {} is not JSON",
            s1.masked_endpoint()
        )
    })?;
    parsed
        .get("answers")
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("System One response has no `answers` object:\n{text}"))
}

/// Extract the Choice answer for one question id: the selected option and
/// its confidence. A provider answering a Choice question with a non-choice
/// type does not speak the contract — say so rather than misreading it.
/// A missing confidence is treated as maximally uncertain (0.0), which
/// flags the answer — the safe default for a contract violation.
fn choice_answer<'a>(answers: &'a Value, id: &str) -> Result<(&'a str, f64)> {
    let answer = answers
        .get(id)
        .ok_or_else(|| anyhow::anyhow!("System One returned no answer for `{id}`"))?;
    if let Some(kind) = answer.get("type").and_then(Value::as_str)
        && kind != "choice"
    {
        bail!(
            "System One answered question `{id}` with type `{kind}`, not `choice` — \
             the provider must speak the System One contract"
        );
    }
    let choice = answer
        .get("choice")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            anyhow::anyhow!("System One's answer for `{id}` has no `choice` value:\n{answer}")
        })?;
    let confidence = answer
        .get("confidence")
        .and_then(Value::as_f64)
        .unwrap_or(0.0);
    Ok((choice, confidence))
}

/// Unique resource-path segments across a service's method ids
/// (`users.messages.list` -> `users`, `messages`), in first-seen order,
/// capped to keep the service-Choice state small.
fn resource_segments(service: &Service) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for method in &service.methods {
        let Some(cut) = method.id.rfind('.') else {
            continue;
        };
        for segment in method.id[..cut].split('.') {
            if !out.iter().any(|s| s == segment) {
                out.push(segment.to_owned());
            }
        }
    }
    out.truncate(14);
    out
}

/// One service's "what it covers" rubric for the service Choice, derived
/// from the index: title, version, method count and the resource paths
/// its methods walk. Deterministic and compact — the criteria carries the
/// semantics, the state's catalog carries the same data as fields.
fn service_rubric(service: &Service) -> String {
    let title = if service.title.is_empty() {
        &service.name
    } else {
        &service.title
    };
    format!(
        "{} ({}): {} methods over {}",
        title,
        service.version,
        service.methods.len(),
        resource_segments(service).join(", ")
    )
}

fn service_by_name(name: &str) -> Result<&'static Service> {
    discovery::services().get(name).ok_or_else(|| {
        let known: Vec<&str> = discovery::services().keys().copied().collect();
        anyhow::anyhow!("unknown service `{name}`; known: {}", known.join(", "))
    })
}

/// Judgment 1: a Choice over the 14 indexed services (+ `none`).
async fn choose_service(s1: &SystemOne, request: &str) -> Result<(&'static Service, f64)> {
    // A Choice accepts at most 255 options; 14 services + none leaves
    // plenty of headroom, but the guard keeps a future index honest.
    if discovery::services().len() >= MAX_CHOICE_OPTIONS {
        bail!(
            "{} services exceed the {MAX_CHOICE_OPTIONS}-option Choice limit",
            discovery::services().len()
        );
    }

    let mut criteria = serde_json::Map::new();
    for (name, service) in discovery::services() {
        criteria.insert((*name).to_owned(), json!(service_rubric(service)));
    }
    criteria.insert(
        NONE.to_owned(),
        json!("None of these services covers what the request needs."),
    );

    let catalog: Vec<Value> = discovery::services()
        .values()
        .map(|service| {
            json!({
                "name": service.name,
                "methods": service.methods.len(),
                "resources": resource_segments(service),
            })
        })
        .collect();

    let state = json!({
        "request": request,
        "catalog": catalog,
    });
    let questions = json!({
        "service": {
            "type": "choice",
            "instructions": {
                "question": "Which service should handle this request?",
                "focus": "Pick the one service whose methods cover what the request needs. Choose `none` when no service does.",
            },
            "criteria": criteria,
        },
    });

    let answers = judge(s1, state, questions).await?;
    let (choice, confidence) = choice_answer(&answers, "service")?;
    if choice == NONE {
        bail!(
            "none of the indexed services covers `{request}`.\n\
             Run `grr api list` to see what is available, or rephrase the request in terms of one of them."
        );
    }
    let service = discovery::services().get(choice).ok_or_else(|| {
        anyhow::anyhow!("System One chose `{choice}`, which is not an indexed service")
    })?;
    Ok((service, confidence))
}

/// Judgment 2: a Choice over the chosen service's methods (+ `none`).
/// Sequential after judgment 1 by necessity — the answer narrows the
/// candidate set — never merged into one request.
async fn choose_method<'a>(
    s1: &SystemOne,
    request: &str,
    service: &'a Service,
) -> Result<(&'a Method, f64)> {
    // A Choice accepts at most 255 options: the biggest service in the
    // current index is well under that, but the guard keeps a future
    // index honest instead of silently truncating candidates.
    if service.methods.len() >= MAX_CHOICE_OPTIONS {
        bail!(
            "{} has {} methods, exceeding the {MAX_CHOICE_OPTIONS}-option Choice limit",
            service.name,
            service.methods.len()
        );
    }

    let mut criteria = serde_json::Map::new();
    for method in &service.methods {
        criteria.insert(
            method.id.clone(),
            json!(method_summary(&method.description)),
        );
    }
    criteria.insert(
        NONE.to_owned(),
        json!("None of these methods does what the request needs."),
    );

    let state = json!({
        "request": request,
        "service": {
            "name": service.name,
            "title": service.title,
            "version": service.version,
            "methods": service.methods.len(),
        },
        "step": "The service was already chosen. Now pick the one method that does what the request needs. Choose `none` when none does.",
    });
    let questions = json!({
        "method": {
            "type": "choice",
            "instructions": {
                "question": format!(
                    "Which method of the {} service should handle this request?",
                    service.name
                ),
                "focus": "Match the request's intent to one method's description.",
            },
            "criteria": criteria,
        },
    });

    let answers = judge(s1, state, questions).await?;
    let (choice, confidence) = choice_answer(&answers, "method")?;
    if choice == NONE {
        bail!(
            "none of {}'s {} methods covers `{request}`.\n\
             Run `grr api list {}` to see what is available, or rephrase the request.",
            service.name,
            service.methods.len(),
            service.name
        );
    }
    let method = service.method(choice).ok_or_else(|| {
        anyhow::anyhow!(
            "System One chose `{choice}`, which is not a method of `{}`",
            service.name
        )
    })?;
    Ok((method, confidence))
}

fn method_summary(description: &str) -> String {
    description.chars().take(200).collect()
}

/// Is this a user-id parameter that the universal `me` alias fills?
/// Gmail and Calendar write it `userId`, People `userKey`.
fn is_user_param(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "userid" | "user" | "userkey"
    )
}

/// Candidate parameter values found in the request text: quoted spans
/// first, then whitespace tokens carrying a digit or an `@` (ids, emails,
/// amounts). Deduped, capped — the model can only select among these, so
/// recall matters more than precision, but every option is a value the
/// code supplied.
fn value_candidates(request: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut rest = request;
    while let Some(start) = rest.find(['\'', '"']) {
        let quote = &rest[start..start + 1];
        let Some(end) = rest[start + 1..].find(quote) else {
            break;
        };
        push_candidate(&mut out, &rest[start + 1..start + 1 + end]);
        rest = &rest[start + 1 + end + 1..];
    }
    for token in request.split_whitespace() {
        let cleaned = token.trim_matches(|c: char| ".,;:!?()[]{}\"'".contains(c));
        if cleaned.chars().any(|c| c.is_ascii_digit() || c == '@') {
            push_candidate(&mut out, cleaned);
        }
    }
    out.truncate(8);
    out
}

fn push_candidate(out: &mut Vec<String>, value: &str) {
    let value = value.trim();
    if value.is_empty()
        || value.len() > 120
        || value == NONE
        || value == "me"
        || out.iter().any(|c| c == value)
    {
        return;
    }
    out.push(value.to_owned());
}

/// One per-param question: structured instructions (the method context +
/// the parameter's name/kind/description) and a criteria object mapping
/// each option to a rubric. Options are the parameter's enum values when
/// it declares them — a closed set, so the answer is always a value the
/// method accepts — else the candidate spans found in the request, plus
/// the authenticated-user alias for user-id parameters, plus `none`.
fn param_question(param: &Parameter, method_description: &str, candidates: &[String]) -> Value {
    let mut criteria = serde_json::Map::new();
    match &param.enum_values {
        Some(values) => {
            for value in values {
                criteria.insert(value.clone(), Value::Null);
            }
        }
        None => {
            for candidate in candidates {
                criteria.insert(candidate.clone(), Value::Null);
            }
            if is_user_param(&param.name) {
                criteria.insert(
                    "me".to_owned(),
                    json!("The authenticated user — the universal alias for this parameter"),
                );
            }
        }
    }
    criteria.insert(
        NONE.to_owned(),
        json!(format!(
            "No value in the request fits the `{}` parameter.",
            param.name
        )),
    );

    json!({
        "type": "choice",
        "instructions": {
            "question": format!("What value should the `{}` parameter take?", param.name),
            "method": method_description,
            "parameter": {
                "name": param.name,
                "kind": param.kind,
                "description": param.description,
                "required": true,
            },
        },
        "criteria": criteria,
    })
}

/// The judgment-3 questions map: one Choice question per required
/// parameter, all in one request. Built once per flow, testable in
/// isolation.
fn param_fill_questions(
    method_description: &str,
    method: &Method,
    candidates: &[String],
) -> serde_json::Map<String, Value> {
    let mut questions = serde_json::Map::new();
    for required in &method.required {
        let Some(param) = method.parameter(required) else {
            continue;
        };
        questions.insert(
            param.name.clone(),
            param_question(param, method_description, candidates),
        );
    }
    questions
}

/// Keep the parameter's real JSON type: a required integer/number
/// parameter whose choice parses as a number is inserted as a number, not
/// the string "10" — the same coercion philosophy as `parse_params`.
fn coerce_value(choice: &str, param: &Parameter) -> Value {
    match param.kind.as_str() {
        "integer" => choice
            .parse::<i64>()
            .map(|n| json!(n))
            .unwrap_or(json!(choice)),
        "number" => choice
            .parse::<f64>()
            .map(|n| json!(n))
            .unwrap_or(json!(choice)),
        "boolean" => choice
            .parse::<bool>()
            .map(|b| json!(b))
            .unwrap_or(json!(choice)),
        _ => json!(choice),
    }
}

/// Judgment 3: fill the chosen method's required parameters. One request,
/// one Choice question per required parameter — the questions map IS the
/// object with one field per required param, and the aggregated answers
/// become the parameter values.
///
/// The documented System One contract returns choice/noul/score answers —
/// there is no object-returning question type — so the object answer is
/// realized as parallel per-param questions aggregated in code. Every
/// value is type-safe by construction: the model can only select among
/// code-supplied candidates, never invent.
async fn fill_params(
    s1: &SystemOne,
    request: &str,
    service: &Service,
    method: &Method,
) -> Result<serde_json::Map<String, Value>> {
    let candidates = value_candidates(request);
    let questions = param_fill_questions(&method.description, method, &candidates);
    if questions.is_empty() {
        return Ok(serde_json::Map::new());
    }

    let state = json!({
        "request": request,
        "method": {
            "id": format!("{}.{}", service.name, method.id),
            "description": method.description,
            "http": method.http_method,
        },
        "step": "The method was already chosen. Now fill each required parameter from the request.",
    });

    let answers = judge(s1, state, Value::Object(questions)).await?;

    let mut params = serde_json::Map::new();
    for required in &method.required {
        let Some(param) = method.parameter(required) else {
            continue;
        };
        let (choice, _) = choice_answer(&answers, &param.name)?;
        if choice == NONE {
            let full_id = format!("{}.{}", service.name, method.id);
            bail!(
                "could not fill the required parameter `{}` for {full_id} from the request.\n\
                 Run `grr api describe {full_id}` to see its parameters, or supply them directly:\n\
                 `grr api call {full_id} --params '{{\"{}\": \"...\"}}'`",
                param.name,
                param.name
            );
        }
        params.insert(param.name.clone(), coerce_value(choice, param));
    }
    Ok(params)
}

/// Everything `grr ask` resolved: the routed method, the filled parameters
/// and the built request plan. `confidence` is the method Choice's (None
/// when --method skipped the choices); `service_confidence` the service
/// Choice's (None when --service ran).
#[derive(Debug)]
struct AskResolution {
    service: &'static Service,
    method: &'static Method,
    params: serde_json::Map<String, Value>,
    confidence: Option<f64>,
    service_confidence: Option<f64>,
    model: String,
    provider: String,
    plan: RequestPlan,
    low_confidence: bool,
}

/// The full flow: resolve the provider, run the judgments the flags don't
/// skip, gate on the safety profile, fill the parameters and plan the
/// request. Pure with respect to Google — no credential needed, no call
/// happens until `--run`.
async fn resolve_ask(
    config: &GrrConfig,
    args: &AskArgs,
    profile: &SafetyProfile,
) -> Result<AskResolution> {
    let s1 = resolve_systemone(config)?;
    let request = args.request.trim();

    let (service, method, method_confidence, service_confidence) =
        if let Some(id) = args.method.as_deref() {
            if let Some(wanted) = args.service.as_deref() {
                let actual = id.split('.').next().unwrap_or(id);
                if !actual.eq_ignore_ascii_case(wanted) {
                    bail!(
                        "--method {id} belongs to service `{actual}`, but --service {wanted} \
                         was given. Drop --service or use a matching method id."
                    );
                }
            }
            let (service, method) = discovery::resolve(id).map_err(|message| {
                anyhow::anyhow!(
                    "{message}\n\nRun `grr api list {}` to see what is available.",
                    id.split('.').next().unwrap_or("gmail")
                )
            })?;
            (service, method, None, None)
        } else if let Some(name) = args.service.as_deref() {
            let service = service_by_name(name)?;
            let (method, confidence) = choose_method(&s1, request, service).await?;
            (service, method, Some(confidence), None)
        } else {
            let (service, s_confidence) = choose_service(&s1, request).await?;
            let (method, m_confidence) = choose_method(&s1, request, service).await?;
            (service, method, Some(m_confidence), Some(s_confidence))
        };

    // The safety gate runs as soon as the method is known — before the
    // parameter-fill judgment and before anything is built or sent. The
    // same gate `grr api call` honors (src/commands/safety.rs).
    if let Err(message) = profile.check(&service.name, method) {
        bail!("{message}");
    }

    let params = if method.required.is_empty() {
        serde_json::Map::new()
    } else {
        fill_params(&s1, request, service, method).await?
    };

    let plan = plan_request(service, method, &params, None, None, &[])?;

    let threshold = config.systemone.threshold();
    let low_confidence = method_confidence.is_some_and(|c| c < threshold)
        || service_confidence.is_some_and(|c| c < threshold);

    Ok(AskResolution {
        service,
        method,
        params,
        confidence: method_confidence,
        service_confidence,
        model: s1.model,
        provider: s1.endpoint,
        plan,
        low_confidence,
    })
}

/// The plan JSON — the default output of `grr ask`. The API key never
/// appears: it rides in the Authorization header only.
fn ask_plan_payload(request: &str, resolution: &AskResolution) -> Value {
    json!({
        "request": request,
        "method": format!("{}.{}", resolution.service.name, resolution.method.id),
        "service": resolution.service.name,
        "httpMethod": resolution.plan.verb,
        "params": resolution.params,
        "url": resolution.plan.url,
        "body": resolution.plan.body,
        "scopes": resolution.method.scopes,
        "leastPrivilegeScope": resolution.method.least_privilege_scope(),
        "confidence": resolution.confidence,
        "serviceConfidence": resolution.service_confidence,
        "model": resolution.model,
        "provider": resolution.provider,
        "dryRun": true,
        "lowConfidence": resolution.low_confidence,
        "nextStep": "rerun with --run to execute this request",
    })
}

/// THE `grr ask` handler — the wire-up target for cli.rs.
///
/// Planning is the default: the resolved plan (method, params, URL,
/// scopes) is the output, nothing is sent, and no Google credential is
/// needed. `--run` executes the plan through the shared call path
/// (`call_method`) and prints the response body. Low confidence is
/// flagged on stderr, never silently swallowed and never blocking.
pub async fn handle_ask(
    config: &GrrConfig,
    auth: Option<&GoogleAuth>,
    args: AskArgs,
    profile: &SafetyProfile,
) -> Result<()> {
    let resolution = resolve_ask(config, &args, profile).await?;
    let format = args.format;
    let full_id = format!("{}.{}", resolution.service.name, resolution.method.id);

    if resolution.low_confidence {
        let confidence = resolution
            .confidence
            .or(resolution.service_confidence)
            .unwrap_or(0.0);
        eprintln!(
            "warning: low confidence ({confidence:.2} < {:.2}); verify the resolved method before trusting it",
            config.systemone.threshold()
        );
    }

    if args.dry_run || !args.run {
        eprintln!(
            "note: resolved {full_id} ({} {}); plan below — rerun with --run to execute",
            resolution.plan.verb, resolution.plan.url
        );
        print_output(&ask_plan_payload(&args.request, &resolution), format)?;
        return Ok(());
    }

    let Some(auth) = auth else {
        bail!(
            "--run needs a Google credential, but none was built. Re-run without --run to print the plan only."
        );
    };
    eprintln!(
        "note: resolved {full_id} ({} {}); running",
        resolution.plan.verb, resolution.plan.url
    );
    let payload = call_method(
        auth,
        resolution.service,
        resolution.method,
        resolution.params,
        CallOptions {
            dry_run: false,
            ..Default::default()
        },
    )
    .await?;
    print_output(&payload, format)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::auth::TokenStorage;
    use wiremock::Match;
    use wiremock::matchers::method;
    use wiremock::{Mock, MockServer, Request, ResponseTemplate};

    /// Distinguish the three judgments by the questions map they carry:
    /// the service Choice, the method Choice, the per-param fills.
    struct QuestionKey(&'static str);

    impl Match for QuestionKey {
        fn matches(&self, request: &Request) -> bool {
            serde_json::from_slice::<Value>(&request.body)
                .ok()
                .and_then(|body| {
                    body.get("questions")
                        .and_then(Value::as_object)
                        .map(|questions| questions.contains_key(self.0))
                })
                .unwrap_or(false)
        }
    }

    fn choice_response(id: &str, choice: &str, confidence: f64) -> ResponseTemplate {
        ResponseTemplate::new(200).set_body_json(json!({
            "model": "jev-1.13.0",
            "answers": {
                id: {
                    "type": "choice",
                    "choice": choice,
                    "probabilities": { choice: confidence },
                    "confidence": confidence,
                }
            },
            "usage": {},
        }))
    }

    async fn test_auth() -> GoogleAuth {
        let config = crate::core::config::OAuthConfig {
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
        GoogleAuth::with_token(config, storage).await.unwrap()
    }

    fn test_config(endpoint: &str, model: &str) -> GrrConfig {
        GrrConfig {
            oauth: crate::core::config::OAuthConfig {
                client_id: "test-id.apps.googleusercontent.com".into(),
                client_secret: None,
            },
            systemone: crate::core::config::SystemOneConfig {
                endpoint: endpoint.into(),
                model: model.into(),
                api_key: Some("test-systemone-key".into()),
                confidence_threshold: None,
            },
        }
    }

    fn plan_args(request: &str) -> AskArgs {
        AskArgs {
            request: request.into(),
            run: false,
            service: None,
            method: None,
            dry_run: false,
            format: OutputFormat::Json,
        }
    }

    #[tokio::test]
    async fn the_hierarchical_flow_resolves_the_right_method_and_params() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(QuestionKey("service"))
            .respond_with(choice_response("service", "gmail", 0.9))
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(QuestionKey("method"))
            .respond_with(choice_response("method", "users.threads.get", 0.81))
            .mount(&server)
            .await;
        // Judgment 3 carries BOTH required params in one request; the
        // answers come back keyed by the same ids.
        Mock::given(method("POST"))
            .and(QuestionKey("userId"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "model": "jev-1.13.0",
                "answers": {
                    "userId": {
                        "type": "choice",
                        "choice": "me",
                        "probabilities": { "me": 0.95 },
                        "confidence": 0.95,
                    },
                    "id": {
                        "type": "choice",
                        "choice": "abc123",
                        "probabilities": { "abc123": 0.88 },
                        "confidence": 0.88,
                    },
                },
                "usage": {},
            })))
            .mount(&server)
            .await;

        let args = plan_args("show the message thread abc123");
        let resolution = resolve_ask(
            &test_config(&server.uri(), ""),
            &args,
            &SafetyProfile::PERMISSIVE,
        )
        .await
        .unwrap();

        assert_eq!(resolution.service.name, "gmail");
        assert_eq!(resolution.method.id, "users.threads.get");
        assert_eq!(resolution.params.get("userId").unwrap(), "me");
        assert_eq!(resolution.params.get("id").unwrap(), "abc123");
        assert_eq!(resolution.confidence, Some(0.81));
        assert_eq!(resolution.service_confidence, Some(0.9));
        assert_eq!(resolution.plan.verb, "GET");
        assert_eq!(
            resolution.plan.url,
            "https://gmail.googleapis.com/gmail/v1/users/me/threads/abc123"
        );

        // Three judgments = three small HTTP requests, sequential.
        assert_eq!(server.received_requests().await.unwrap().len(), 3);
    }

    #[tokio::test]
    async fn the_no_match_outcome_is_actionable() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(QuestionKey("service"))
            .respond_with(choice_response("service", "none", 0.7))
            .mount(&server)
            .await;

        let args = plan_args("book a flight to the moon");
        let err = resolve_ask(
            &test_config(&server.uri(), ""),
            &args,
            &SafetyProfile::PERMISSIVE,
        )
        .await
        .unwrap_err()
        .to_string();
        assert!(err.contains("none of the indexed services"), "{err}");
        assert!(err.contains("grr api list"), "{err}");
    }

    #[tokio::test]
    async fn a_method_no_match_is_actionable_too() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(QuestionKey("method"))
            .respond_with(choice_response("method", "none", 0.7))
            .mount(&server)
            .await;

        let mut args = plan_args("book a flight to the moon");
        args.service = Some("gmail".into());
        let err = resolve_ask(
            &test_config(&server.uri(), ""),
            &args,
            &SafetyProfile::PERMISSIVE,
        )
        .await
        .unwrap_err()
        .to_string();
        assert!(err.contains("none of gmail's"), "{err}");
        assert!(err.contains("grr api list gmail"), "{err}");
    }

    #[tokio::test]
    async fn low_confidence_is_flagged_in_the_resolution() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(QuestionKey("method"))
            .respond_with(choice_response("method", "users.threads.get", 0.42))
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(QuestionKey("userId"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "model": "jev-1.13.0",
                "answers": {
                    "userId": {
                        "type": "choice",
                        "choice": "me",
                        "probabilities": { "me": 0.95 },
                        "confidence": 0.95,
                    },
                    "id": {
                        "type": "choice",
                        "choice": "abc123",
                        "probabilities": { "abc123": 0.9 },
                        "confidence": 0.9,
                    },
                },
                "usage": {},
            })))
            .mount(&server)
            .await;

        let mut args = plan_args("show the message thread abc123");
        args.service = Some("gmail".into());
        let resolution = resolve_ask(
            &test_config(&server.uri(), ""),
            &args,
            &SafetyProfile::PERMISSIVE,
        )
        .await
        .unwrap();
        assert!(resolution.low_confidence, "0.42 < 0.6 must flag");
        assert_eq!(resolution.confidence, Some(0.42));

        let payload = ask_plan_payload(&args.request, &resolution);
        assert_eq!(payload["lowConfidence"], true);
        assert_eq!(payload["confidence"], 0.42);
    }

    #[tokio::test]
    async fn a_custom_endpoint_and_model_from_config_are_used() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(QuestionKey("service"))
            .respond_with(choice_response("service", "gmail", 0.9))
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(QuestionKey("method"))
            .respond_with(choice_response("method", "users.getProfile", 0.9))
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(QuestionKey("userId"))
            .respond_with(choice_response("userId", "me", 0.95))
            .mount(&server)
            .await;

        let args = plan_args("show my profile");
        let resolution = resolve_ask(
            &test_config(&server.uri(), "my-jev-fork"),
            &args,
            &SafetyProfile::PERMISSIVE,
        )
        .await
        .unwrap();

        // The wiremock server WAS hit (the mocks matched), which proves the
        // configured endpoint was used instead of the TypeSafe default; the
        // request bodies carry the configured model.
        let requests = server.received_requests().await.unwrap();
        assert_eq!(requests.len(), 3);
        for request in &requests {
            let body: Value = serde_json::from_slice(&request.body).unwrap();
            assert_eq!(body["model"], "my-jev-fork");
        }
        assert_eq!(resolution.model, "my-jev-fork");
    }

    #[tokio::test]
    async fn the_api_key_never_enters_a_body_or_the_plan_output() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(QuestionKey("method"))
            .respond_with(choice_response("method", "users.getProfile", 0.9))
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(QuestionKey("userId"))
            .respond_with(choice_response("userId", "me", 0.95))
            .mount(&server)
            .await;

        let mut args = plan_args("show my profile");
        args.service = Some("gmail".into());
        let resolution = resolve_ask(
            &test_config(&server.uri(), ""),
            &args,
            &SafetyProfile::PERMISSIVE,
        )
        .await
        .unwrap();
        let payload = ask_plan_payload(&args.request, &resolution);
        let plan_json = serde_json::to_string(&payload).unwrap();
        assert!(
            !plan_json.contains("test-systemone-key"),
            "the key leaked into the plan: {plan_json}"
        );

        for request in server.received_requests().await.unwrap() {
            let body = String::from_utf8(request.body.to_vec()).unwrap();
            assert!(
                !body.contains("test-systemone-key"),
                "the key leaked into a request body"
            );
            let bearer = request
                .headers
                .get("authorization")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("");
            assert_eq!(
                bearer, "Bearer test-systemone-key",
                "the key must ride in the Authorization header"
            );
        }
    }

    #[tokio::test]
    async fn handle_ask_reports_a_missing_api_key_actionably() {
        let mut config = test_config("http://127.0.0.1:1/v1/systemone", "");
        config.systemone.api_key = None;
        let args = plan_args("show my profile");
        let err = handle_ask(&config, None, args, &SafetyProfile::PERMISSIVE)
            .await
            .unwrap_err()
            .to_string();
        assert!(err.contains("TYPESAFE_API_KEY"), "{err}");
        assert!(err.contains("[systemone]"), "{err}");
    }

    #[tokio::test]
    async fn handle_ask_honors_the_safety_gate_before_running() {
        // The gate fires as soon as the method is resolved — no System One
        // call, no Google call; the endpoint is a dead port.
        let config = test_config("http://127.0.0.1:1/v1/systemone", "");
        let mut args = plan_args("send a message");
        args.run = true;
        args.method = Some("gmail.users.messages.send".into());
        let auth = test_auth().await;
        let err = handle_ask(&config, Some(&auth), args, &SafetyProfile::readonly())
            .await
            .unwrap_err()
            .to_string();
        assert!(err.contains("POST"), "{err}");
    }

    #[tokio::test]
    async fn handle_ask_run_without_a_credential_is_actionable() {
        // --method skips the judgments and drive.files.list needs no
        // required params, so no System One call happens at all.
        let config = test_config("http://127.0.0.1:1/v1/systemone", "");
        let mut args = plan_args("list my drive files");
        args.run = true;
        args.method = Some("drive.files.list".into());
        let err = handle_ask(&config, None, args, &SafetyProfile::PERMISSIVE)
            .await
            .unwrap_err()
            .to_string();
        assert!(err.contains("--run"), "{err}");
        assert!(err.contains("credential"), "{err}");
    }

    #[test]
    fn every_service_fits_the_choice_option_limit() {
        // A Choice accepts at most 255 options: the service Choice needs
        // 14 + none, the method Choice at most one service's methods + none.
        let all = discovery::services();
        assert!(all.len() < MAX_CHOICE_OPTIONS);
        for (name, service) in all {
            assert!(
                service.methods.len() < MAX_CHOICE_OPTIONS,
                "{name} has {} methods",
                service.methods.len()
            );
        }
    }

    #[test]
    fn param_fill_builds_one_question_per_required_parameter() {
        let (_, method) = discovery::resolve("gmail.users.threads.get").unwrap();
        let questions = param_fill_questions(&method.description, method, &["abc123".to_owned()]);
        assert_eq!(questions.len(), method.required.len());
        assert!(questions.contains_key("userId"));
        assert!(questions.contains_key("id"));
        // The userId question carries the universal alias; the id question
        // carries the candidate found in the request.
        let user_question = questions.get("userId").unwrap();
        assert!(user_question["criteria"].get("me").is_some());
        let id_question = questions.get("id").unwrap();
        assert!(id_question["criteria"].get("abc123").is_some());
    }

    #[test]
    fn enum_parameters_use_the_enum_values_as_options() {
        let param = Parameter {
            name: "orderBy".into(),
            kind: "string".into(),
            required: true,
            repeated: false,
            location: Some("query".into()),
            description: "Sort order".into(),
            enum_values: Some(vec!["startTime".into(), "updated".into()]),
        };
        let question = param_question(&param, "List events.", &["startTime".into()]);
        let options = question["criteria"].as_object().unwrap();
        assert!(options.contains_key("startTime"));
        assert!(options.contains_key("updated"));
        assert!(
            !options.contains_key("me"),
            "enum options are the closed set"
        );
        assert!(options.contains_key("none"));
    }

    #[test]
    fn candidates_come_from_the_request_text_verbatim() {
        let out = value_candidates("show the file named \"a b.pdf\" with id 42");
        assert!(out.contains(&"a b.pdf".to_owned()), "{out:?}");
        assert!(out.contains(&"42".to_owned()), "{out:?}");
        assert!(out.iter().all(|c| c != "none" && c != "me"));
    }

    #[test]
    fn numeric_parameters_keep_their_json_type() {
        let param = Parameter {
            name: "pageSize".into(),
            kind: "integer".into(),
            required: true,
            repeated: false,
            location: Some("query".into()),
            description: "Page size".into(),
            enum_values: None,
        };
        assert_eq!(coerce_value("10", &param), json!(10));
        assert_eq!(coerce_value("not-a-number", &param), json!("not-a-number"));
    }

    #[test]
    fn the_service_rubric_covers_every_indexed_service() {
        for (name, service) in discovery::services() {
            let rubric = service_rubric(service);
            // The rubric names the service by its title (falling back to
            // the name when the title is empty) and always carries the
            // version and a method count.
            let title: &str = if service.title.is_empty() {
                name
            } else {
                service.title.as_str()
            };
            assert!(rubric.contains(title), "{name}: {rubric}");
            assert!(rubric.contains(&service.version), "{name}: {rubric}");
        }
    }
}
