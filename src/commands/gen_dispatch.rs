//! Dispatch for the generated service tree.
//!
//! The tree itself (`src/commands/generated.rs`) is pure clap structure —
//! generated from the Discovery index, no runtime logic. This module is the
//! runtime half: walk the matched subcommand chain to the leaf, resolve the
//! method id against the Discovery index, turn the typed flags back into a
//! parameter map under their ORIGINAL camelCase names, and hand everything
//! to the one shared call path (`api::call_method`) that `grr api call`
//! also uses.

use crate::core::auth::GoogleAuth;
use crate::discovery::{self, Method};
use crate::output::{OutputFormat, print_output};
use anyhow::{Result, bail};
use clap::ArgMatches;
use serde_json::{Map, Value};

use crate::commands::api::{self, call_method};
use crate::commands::generated::GENERATED_AT;
use crate::commands::safety::SafetyProfile;

/// Escape-hatch arg ids present on every generated leaf (plus clap's own
/// `help`). A Discovery parameter whose kebab-case name lands here is
/// renamed `param-<kebab>` — the generator applies the same rule, and the
/// tests pin the two together.
const RESERVED_FLAG_IDS: [&str; 5] = ["params", "body-file", "query", "dry-run", "format"];

/// Discovery parameter name -> generated flag id.
///
/// camelCase becomes kebab-case (`userId` -> `--user-id`), non-alphanumeric
/// runs collapse to a dash (`requestMask.includeField` ->
/// `--request-mask-include-field`), and reserved escape-hatch names are
/// prefixed (`format` -> `--param-format`).
pub(crate) fn flag_id(name: &str) -> String {
    let kebab = kebab_case(name);
    if RESERVED_FLAG_IDS.contains(&kebab.as_str()) || kebab == "help" {
        format!("param-{kebab}")
    } else {
        kebab
    }
}

/// The kebab half of [`flag_id`]. Mirrors `kebab()` in
/// `scripts/generate-commands.mjs` exactly; a drift between the two would
/// make typed flags silently vanish, so tests pin the pairs down.
pub(crate) fn kebab_case(name: &str) -> String {
    let chars: Vec<char> = name.chars().collect();
    let mut out = String::with_capacity(name.len() + 4);
    for (i, &c) in chars.iter().enumerate() {
        if c.is_ascii_uppercase() {
            // `getProfile` -> `get-profile`, but `ACLx` stays one word:
            // only a lowercase/digit -> uppercase boundary is a split.
            if let Some(&previous) = i.checked_sub(1).and_then(|j| chars.get(j))
                && (previous.is_ascii_lowercase() || previous.is_ascii_digit())
            {
                out.push('-');
            }
            out.push(c.to_ascii_lowercase());
        } else if c.is_ascii_alphanumeric() {
            out.push(c);
        } else if !out.is_empty() && !out.ends_with('-') {
            // collapse runs of specials (`.`, `$`, `_`, …) to one dash
            out.push('-');
        }
    }
    out.trim_matches('-').to_owned()
}

/// Entry point for every generated service subcommand under the permissive
/// safety profile. `matches` is the ROOT `ArgMatches` (the one whose current
/// subcommand is the service), so the full chain is walkable from here.
///
/// Production dispatch goes through [`dispatch_with_profile`] (wired in
/// `cli.rs`), which applies the global `--readonly`/`--deny-*` flags; this
/// profile-free wrapper is what the unit tests exercise.
pub async fn dispatch(matches: &ArgMatches, auth: Option<&GoogleAuth>) -> Result<()> {
    dispatch_with_profile(matches, auth, &SafetyProfile::PERMISSIVE).await
}

/// [`dispatch`] with a safety profile — the seam for `cli.rs`. The same
/// gate the `grr api call` flags and the `grr mcp` server honor.
pub async fn dispatch_with_profile(
    matches: &ArgMatches,
    auth: Option<&GoogleAuth>,
    profile: &SafetyProfile,
) -> Result<()> {
    // Walk to the deepest subcommand. The generator guarantees a leaf's
    // canonical clap name IS its full dotted method id
    // (`gmail.users.messages.list`), so the deepest name is the id —
    // dispatch never reconstructs paths. The join below only exists for
    // a leaf reached by some future non-dotted name; since every
    // intermediate subcommand is its verbatim id segment, the join is
    // exact there too.
    let mut cursor = matches;
    let mut segments: Vec<&str> = Vec::new();
    while let Some((name, sub)) = cursor.subcommand() {
        segments.push(name);
        cursor = sub;
    }
    let Some(leaf) = segments.last().copied() else {
        bail!("no method matched; expected <service> <resource...> <method>");
    };
    let id = if leaf.contains('.') {
        leaf.to_owned()
    } else {
        segments.join(".")
    };

    let (service, method) =
        discovery::resolve(&id).map_err(|message| anyhow::anyhow!("{message}"))?;

    // The safety gate runs before anything is built or sent.
    if let Err(message) = profile.check(&service.name, method) {
        bail!("{message}");
    }

    let format = cursor
        .get_one::<OutputFormat>("format")
        .cloned()
        .unwrap_or_default();

    warn_unknown_flags(method, cursor);
    let params = collect_params(method, cursor)?;

    let payload = call_method(
        auth,
        service,
        method,
        params,
        api::CallOptions {
            body_file: cursor.get_one::<String>("body-file").cloned(),
            // The verb override is `grr api call`'s escape hatch; the tree
            // speaks the Discovery document.
            verb_override: None,
            query_extras: query_extras(cursor),
            dry_run: cursor.get_flag("dry-run"),
        },
    )
    .await?;
    print_output(&payload, format)?;
    Ok(())
}

fn query_extras(leaf: &ArgMatches) -> Vec<String> {
    leaf.get_many::<String>("query")
        .map(|values| values.cloned().collect())
        .unwrap_or_default()
}

/// Build the method's parameter map: `--params` JSON first, typed flags
/// second — the merge order of `api::parse_params`, so a typed flag always
/// wins on conflict.
fn collect_params(method: &Method, leaf: &ArgMatches) -> Result<Map<String, Value>> {
    let mut params = api::parse_params(leaf.get_one::<String>("params").map(String::as_str), &[])?;
    for (name, value) in typed_params(method, leaf) {
        params.insert(name, value);
    }
    Ok(params)
}

/// Typed flag values -> JSON values under their ORIGINAL Discovery names.
///
/// Values are read with `try_get_raw` (the pre-parser `OsString`s), never
/// the typed getters: `discovery::resolve` may return a method from a
/// refreshed cache that is newer than this build's generated tree, and a
/// typed getter on a mismatched (or unknown-to-this-build) arg panics
/// inside clap. Raw reads degrade to "flag not recognised" instead.
fn typed_params(method: &Method, leaf: &ArgMatches) -> Vec<(String, Value)> {
    let mut out = Vec::new();
    for param in &method.parameters {
        let id = flag_id(&param.name);
        // Err = this build's tree does not define the flag (cache newer
        // than the binary); Ok(None) = flag defined but not passed.
        let Ok(Some(raw)) = leaf.try_get_raw(&id) else {
            continue;
        };
        let kind = param.kind.as_str();

        // SetTrue flags carry an injected "false" default when absent, so
        // presence is not enough: only a genuinely passed flag ("true")
        // becomes a parameter — an explicit false would change the request
        // URL versus leaving the parameter off entirely.
        let texts: Vec<String> = raw.map(|os| os.to_string_lossy().into_owned()).collect();
        if kind == "boolean" && !param.repeated {
            if texts.iter().any(|t| t.eq_ignore_ascii_case("true")) {
                out.push((param.name.clone(), Value::Bool(true)));
            }
            continue;
        }

        let values: Vec<Value> = texts
            .into_iter()
            .map(|text| {
                if kind == "integer"
                    && let Ok(number) = text.parse::<i64>()
                {
                    Value::Number(number.into())
                } else if kind == "boolean" {
                    Value::Bool(text.eq_ignore_ascii_case("true"))
                } else {
                    Value::String(text)
                }
            })
            .collect();
        if values.is_empty() {
            continue;
        }
        if param.repeated {
            out.push((param.name.clone(), Value::Array(values)));
        } else {
            out.push((
                param.name.clone(),
                values.into_iter().next().unwrap_or(Value::Null),
            ));
        }
    }
    out
}

/// A flag the tree parsed but the resolved method does not know means the
/// on-disk Discovery cache drifted from this build's generated tree. Its
/// value would be silently dropped, so say so and point at --params.
fn warn_unknown_flags(method: &Method, leaf: &ArgMatches) {
    let mut known: Vec<String> = RESERVED_FLAG_IDS
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    known.push("help".to_owned());
    known.extend(method.parameters.iter().map(|p| flag_id(&p.name)));
    for id in leaf.ids() {
        if !known.iter().any(|k| k == id.as_str()) {
            eprintln!(
                "note: `--{id}` is not a parameter of {} in the current Discovery index \
                 (command tree generated at {GENERATED_AT}; a refreshed cache may be newer). \
                 Pass it through --params if the API still accepts it.",
                method.id
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn flag_ids_match_the_generator_rules() {
        // Pinned against `kebab()`/`flagId()` in scripts/generate-commands.mjs.
        assert_eq!(flag_id("userId"), "user-id");
        assert_eq!(flag_id("maxResults"), "max-results");
        assert_eq!(flag_id("labelIds"), "label-ids");
        assert_eq!(flag_id("includeSpamTrash"), "include-spam-trash");
        assert_eq!(
            flag_id("requestMask.includeField"),
            "request-mask-include-field"
        );
        assert_eq!(flag_id("useAdminAccess"), "use-admin-access");
        // Reserved escape-hatch names get the param- prefix.
        assert_eq!(flag_id("format"), "param-format");
        assert_eq!(flag_id("query"), "param-query");
        assert_eq!(flag_id("help"), "param-help");
        // Non-alphanumeric runs collapse; nothing usable is left dangling.
        assert_eq!(kebab_case("$.xgafv"), "xgafv");
        assert_eq!(kebab_case("getThreadReadState"), "get-thread-read-state");
    }

    #[test]
    fn the_tree_walk_yields_the_full_dotted_id() {
        let root = crate::cli::root_command();
        let matches = root
            .try_get_matches_from([
                "grr",
                "gmail",
                "users",
                "messages",
                "list",
                "--user-id",
                "me",
            ])
            .expect("valid invocation");
        let mut cursor = &matches;
        let mut segments = Vec::new();
        while let Some((name, sub)) = cursor.subcommand() {
            segments.push(name);
            cursor = sub;
        }
        let leaf = segments.last().copied().unwrap();
        let id = if leaf.contains('.') {
            leaf.to_owned()
        } else {
            segments.join(".")
        };
        assert_eq!(id, "gmail.users.messages.list");
        assert!(discovery::resolve(&id).is_ok());
    }

    #[test]
    fn typed_flags_round_trip_under_their_original_names() {
        let root = crate::cli::root_command();
        let matches = root
            .try_get_matches_from([
                "grr",
                "gmail",
                "users",
                "messages",
                "list",
                "--user-id",
                "me",
                "--max-results",
                "10",
                "--include-spam-trash",
                "--label-ids",
                "INBOX",
                "--label-ids",
                "UNREAD",
                "--q",
                "is:unread",
            ])
            .expect("valid invocation");
        let leaf = deepest(&matches);
        let (_, method) = discovery::resolve("gmail.users.messages.list").unwrap();
        let params = collect_params(method, leaf).unwrap();

        assert_eq!(params.get("userId"), Some(&json!("me")));
        assert_eq!(params.get("maxResults"), Some(&json!(10)));
        assert_eq!(params.get("includeSpamTrash"), Some(&json!(true)));
        assert_eq!(params.get("q"), Some(&json!("is:unread")));
        assert_eq!(params.get("labelIds"), Some(&json!(["INBOX", "UNREAD"])));
    }

    #[test]
    fn typed_flags_win_over_params_json() {
        let root = crate::cli::root_command();
        let matches = root
            .try_get_matches_from([
                "grr",
                "gmail",
                "users",
                "messages",
                "list",
                "--user-id",
                "me",
                "--params",
                r#"{"userId":"someone-else","q":"is:unread"}"#,
            ])
            .expect("valid invocation");
        let leaf = deepest(&matches);
        let (_, method) = discovery::resolve("gmail.users.messages.list").unwrap();
        let params = collect_params(method, leaf).unwrap();
        assert_eq!(params.get("userId"), Some(&json!("me")));
        assert_eq!(params.get("q"), Some(&json!("is:unread")));
    }

    #[test]
    fn a_service_with_no_curated_commands_is_reachable() {
        // Tasks had zero hand-written commands; the generated tree is its
        // entire surface.
        let root = crate::cli::root_command();
        let matches = root
            .try_get_matches_from(["grr", "tasks", "tasklists", "list", "--dry-run"])
            .expect("valid invocation");
        let leaf = deepest(&matches);
        let (_, method) = discovery::resolve("tasks.tasklists.list").unwrap();
        let params = collect_params(method, leaf).unwrap();
        assert_eq!(params.get("userId"), None);
    }

    #[test]
    fn generated_and_api_call_paths_plan_identical_dry_runs() {
        // The parity contract: the same method and parameters must produce
        // byte-identical dry-run output whether they arrive through the
        // generated tree (typed flags) or `grr api call` (--params JSON).
        let (service, method) = discovery::resolve("gmail.users.messages.list").unwrap();

        // `grr api call` route: --params JSON blob.
        let api_params = api::parse_params(
            Some(r#"{"userId":"me","q":"is:unread","labelIds":["INBOX","UNREAD"]}"#),
            &[],
        )
        .unwrap();
        let api_plan = api::plan_request(service, method, &api_params, None, None, &[]).unwrap();
        let api_payload = api::dry_run_payload(method, &api_plan);

        // Generated route: the real clap tree, flags parsed by clap.
        let root = crate::cli::root_command();
        let matches = root
            .try_get_matches_from([
                "grr",
                "gmail",
                "users",
                "messages",
                "list",
                "--user-id",
                "me",
                "--q",
                "is:unread",
                "--label-ids",
                "INBOX",
                "--label-ids",
                "UNREAD",
            ])
            .expect("valid invocation");
        let leaf = deepest(&matches);
        let gen_params = collect_params(method, leaf).unwrap();
        let gen_plan = api::plan_request(service, method, &gen_params, None, None, &[]).unwrap();
        let gen_payload = api::dry_run_payload(method, &gen_plan);

        assert_eq!(api_plan.verb, gen_plan.verb);
        assert_eq!(api_plan.url, gen_plan.url);
        assert_eq!(api_plan.body, gen_plan.body);
        assert_eq!(
            serde_json::to_string(&api_payload).unwrap(),
            serde_json::to_string(&gen_payload).unwrap()
        );
    }

    #[test]
    fn a_reserved_flag_name_is_reachable_under_its_param_alias() {
        // `format` on gmail.users.messages.get collides with the output
        // --format escape hatch, so the generator exposes it as
        // --param-format and dispatch must find it by that id.
        let root = crate::cli::root_command();
        let matches = root
            .try_get_matches_from([
                "grr",
                "gmail",
                "users",
                "messages",
                "get",
                "--user-id",
                "me",
                "--id",
                "m1",
                "--param-format",
                "raw",
            ])
            .expect("valid invocation");
        let leaf = deepest(&matches);
        let (_, method) = discovery::resolve("gmail.users.messages.get").unwrap();
        let params = collect_params(method, leaf).unwrap();
        assert_eq!(params.get("format"), Some(&json!("raw")));
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
    async fn dispatch_with_profile_refuses_a_write() {
        // The gate fires before anything is built or sent, so the dummy
        // auth handle is never used.
        let root = crate::cli::root_command();
        let matches = root
            .try_get_matches_from([
                "grr",
                "gmail",
                "users",
                "messages",
                "send",
                "--user-id",
                "me",
                "--dry-run",
            ])
            .expect("valid invocation");
        let auth = test_auth().await;
        let err = dispatch_with_profile(&matches, Some(&auth), &SafetyProfile::readonly())
            .await
            .unwrap_err()
            .to_string();
        assert!(err.contains("POST"), "{err}");
        assert!(err.contains("--readonly"), "{err}");
    }

    #[tokio::test]
    async fn dispatch_with_profile_refuses_a_denied_service_only() {
        let root = crate::cli::root_command();
        let matches = root
            .try_get_matches_from([
                "grr",
                "gmail",
                "users",
                "messages",
                "list",
                "--user-id",
                "me",
                "--dry-run",
            ])
            .expect("valid invocation");
        let auth = test_auth().await;
        let profile = SafetyProfile::new(false, ["gmail".to_owned()], []);
        let err = dispatch_with_profile(&matches, Some(&auth), &profile)
            .await
            .unwrap_err()
            .to_string();
        assert!(err.contains("--deny-service gmail"), "{err}");

        // Another service rides through untouched.
        let root = crate::cli::root_command();
        let matches = root
            .try_get_matches_from(["grr", "tasks", "tasklists", "list", "--dry-run"])
            .expect("valid invocation");
        assert!(
            dispatch_with_profile(&matches, Some(&auth), &profile)
                .await
                .is_ok()
        );
    }

    #[tokio::test]
    async fn plain_dispatch_stays_permissive() {
        // The unwired default: the same dry-run the tree always returned.
        let root = crate::cli::root_command();
        let matches = root
            .try_get_matches_from([
                "grr",
                "gmail",
                "users",
                "messages",
                "list",
                "--user-id",
                "me",
                "--dry-run",
            ])
            .expect("valid invocation");
        let auth = test_auth().await;
        assert!(dispatch(&matches, Some(&auth)).await.is_ok());
    }

    fn deepest(matches: &ArgMatches) -> &ArgMatches {
        let mut cursor = matches;
        while let Some((_, sub)) = cursor.subcommand() {
            cursor = sub;
        }
        cursor
    }
}
