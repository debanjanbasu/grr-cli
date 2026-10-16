//! Auth CLI commands: account-level authentication.
//!
//! No typed service client is involved: `login` needs only the OAuth
//! handle, and `status` reaches Gmail's profile through the shared
//! Discovery call path (`gmail.users.getProfile`) — the same engine as
//! `grr api call` and the generated tree.

use crate::commands::api;
use crate::core::auth::GoogleAuth;
use crate::discovery;
use crate::output::{OutputFormat, print_output};
use anyhow::Result;
use clap::Subcommand;
use serde_json::{Value, json};

#[derive(Subcommand, Debug)]
pub enum AuthCommands {
    /// Fresh login. Default is the PKCE browser (loopback) flow; --device
    /// prints a URL + code for headless environments instead.
    Login(LoginArgs),
    /// Show the authenticated account (fails when no valid credential)
    Status(StatusArgs),
    /// Create and store an OAuth client. Release builds ship with one, so
    /// this is only needed when using your own client.
    Setup(crate::commands::setup::SetupArgs),
}

#[derive(clap::Args, Debug)]
pub struct LoginArgs {
    /// Use the OAuth device flow instead of the browser loopback flow.
    /// Needs a client of type "TVs and Limited Input devices"
    /// (GRR_OAUTH__DEVICE_CLIENT_ID); the browser flow's Desktop client is
    /// not accepted by Google's device endpoint.
    #[arg(long)]
    pub device: bool,

    /// Log in as a NAMED account (e.g. work), keeping a separate token per
    /// account. Omit for the default account.
    #[arg(long)]
    pub account: Option<String>,

    /// Ask for one service's scopes in addition to the default set (e.g.
    /// docs, chat, searchconsole). Every login merges with what the account
    /// already granted, so widening in steps keeps each consent page small
    /// enough for Google to finish.
    #[arg(long, conflicts_with = "all_scopes")]
    pub service: Option<String>,

    /// Ask for every scope the embedded index can express. One big consent
    /// page: on an unverified project Google's screen stops submitting past
    /// roughly 27 ticked boxes, so prefer `--service` unless you know the
    /// project is verified.
    #[arg(long)]
    pub all_scopes: bool,

    /// Output format
    #[arg(short, long, value_enum, default_value = "json")]
    pub format: OutputFormat,
}

#[derive(clap::Args, Debug)]
pub struct StatusArgs {
    /// Show the status of a NAMED account instead of the default one
    #[arg(long)]
    pub account: Option<String>,

    /// Output format
    #[arg(short, long, value_enum, default_value = "json")]
    pub format: OutputFormat,
}

fn token_preview(token: &str) -> String {
    format!("{}...", &token[..std::cmp::min(20, token.len())])
}

/// The scope set `auth login` should ask for, from the flags.
///
/// Default: [`crate::core::auth::login_scopes`], the small set that keeps the
/// consent page submittable. `--service X` adds that service's scopes (the
/// index decides them — the same source `grr api describe` reads), while
/// `--all-scopes` asks for the whole index. Every login sends
/// `include_granted_scopes`, so each of these merges with the account's
/// existing grants rather than replacing them.
fn requested_login_scopes(login: &LoginArgs) -> Result<Vec<String>> {
    if login.all_scopes {
        return Ok(crate::core::auth::scopes().to_vec());
    }
    let mut set = crate::core::auth::login_scopes();
    if let Some(service) = &login.service {
        let scopes = crate::core::auth::scopes_for_service(service).ok_or_else(|| {
            crate::core::error::GrrError::Config(format!(
                "unknown service `{service}`: `grr api list` shows the ids the index knows"
            ))
        })?;
        set.extend(scopes);
        set.sort_unstable();
        set.dedup();
    }
    Ok(set)
}

pub async fn handle_auth_cmd(auth: &GoogleAuth, cmd: AuthCommands) -> Result<()> {
    match cmd {
        // Handled before client construction in cli.rs; this arm is only a
        // guard so the match stays exhaustive.
        AuthCommands::Setup(_) => unreachable!("auth setup handled before client construction"),
        AuthCommands::Login(login) => {
            // A named account rebinds to that account's own token store;
            // the default account keeps today's exact entry naming, so an
            // existing login is untouched. A fresh handle is built either
            // way (a cheap store + HTTP-client construction).
            let auth = match &login.account {
                Some(name) => auth.with_account_store(name).await?,
                None => auth.with_default_account_store().await?,
            };
            let scopes = requested_login_scopes(&login)?;
            if login.device {
                return handle_device_login(&auth, &scopes, login.format).await;
            }
            let storage = auth.login(&scopes).await?;
            print_output(
                &json!({
                    "authenticated": true,
                    "account": login.account,
                    "token_backend": auth.token_backend(),
                    "scopes_requested": scopes.len(),
                    "token_preview": token_preview(&storage.access_token)
                }),
                login.format,
            )?;
            Ok(())
        }
        AuthCommands::Status(status) => {
            let auth = match &status.account {
                Some(name) => auth.with_account_store(name).await?,
                None => auth.with_default_account_store().await?,
            };
            let profile = fetch_profile(&auth).await?;
            // The credential is in memory by now — `fetch_profile` is what
            // authenticated — so reporting its scope never touches the
            // keyring again. `login` reports what was *requested*; this is
            // what the credential actually holds: Google's granted set wins
            // over the request, and the consent screen lets a user approve a
            // subset of it.
            let scopes_granted = auth.granted_scopes().await;
            print_output(
                &json!({
                    "authenticated": true,
                    "account": status.account,
                    "email": field(&profile, "emailAddress"),
                    "messages_total": field(&profile, "messagesTotal"),
                    "threads_total": field(&profile, "threadsTotal"),
                    "history_id": field(&profile, "historyId"),
                    "scopes_granted": scopes_granted,
                }),
                status.format,
            )?;
            Ok(())
        }
    }
}

/// The profile check behind `auth status`, via the shared Discovery path.
///
/// The payload keys are remapped to the long-standing `auth status`
/// contract (snake_case, `authenticated` flag), so scripts reading the
/// output did not change when the typed Gmail client left the CLI.
async fn fetch_profile(auth: &GoogleAuth) -> Result<Value> {
    let (service, method) =
        discovery::resolve("gmail.users.getProfile").map_err(anyhow::Error::msg)?;
    let mut params = serde_json::Map::new();
    params.insert("userId".into(), json!("me"));
    api::call_method(
        Some(auth),
        service,
        method,
        params,
        api::CallOptions::default(),
    )
    .await
}

fn field(profile: &Value, key: &str) -> Value {
    profile.get(key).cloned().unwrap_or(Value::Null)
}

/// The device-flow login behind `grr auth login --device`.
///
/// `requested` is the same flag-derived set the browser flow gets
/// (`--service` / `--all-scopes` included); the device flow may only send it
/// intersected with Google's allowlist, and that intersection is what both the
/// request and the stored fallback use — the flags are not silently ignored
/// here, and the credential never records the index superset.
async fn handle_device_login(
    auth: &GoogleAuth,
    requested: &[String],
    format: OutputFormat,
) -> Result<()> {
    let scopes = crate::core::auth::device_flow_scopes(requested);
    let mut challenge = auth.request_device_code(&scopes).await?;
    // stderr, not stdout: `--format json` writes its payload to stdout, and a
    // prose line ahead of it breaks `... -f json | jq`.
    eprintln!(
        "Visit {} and enter code: {}",
        challenge.verification_url, challenge.user_code
    );
    loop {
        tokio::time::sleep(challenge.retry_after()).await;
        if challenge.is_expired() {
            anyhow::bail!("device code expired; rerun `grr auth login --device`");
        }
        if let Some(storage) = auth.poll_device_code(&mut challenge, &scopes).await? {
            print_output(
                &json!({
                    "authenticated": true,
                    "token_backend": auth.token_backend(),
                    "token_preview": token_preview(&storage.access_token)
                }),
                format,
            )?;
            return Ok(());
        }
    }
}
