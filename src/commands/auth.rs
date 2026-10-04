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
    /// Use the OAuth device flow instead of the browser loopback flow
    #[arg(long)]
    pub device: bool,

    /// Log in as a NAMED account (e.g. work), keeping a separate token per
    /// account. Omit for the default account.
    #[arg(long)]
    pub account: Option<String>,

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
            if login.device {
                return handle_device_login(&auth, login.format).await;
            }
            let storage = auth.login().await?;
            print_output(
                &json!({
                    "authenticated": true,
                    "account": login.account,
                    "token_backend": auth.token_backend(),
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
            print_output(
                &json!({
                    "authenticated": true,
                    "account": status.account,
                    "email": field(&profile, "emailAddress"),
                    "messages_total": field(&profile, "messagesTotal"),
                    "threads_total": field(&profile, "threadsTotal"),
                    "history_id": field(&profile, "historyId"),
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

async fn handle_device_login(auth: &GoogleAuth, format: OutputFormat) -> Result<()> {
    let mut challenge = auth.request_device_code().await?;
    println!(
        "Visit {} and enter code: {}",
        challenge.verification_url, challenge.user_code
    );
    loop {
        tokio::time::sleep(challenge.retry_after()).await;
        if challenge.is_expired() {
            anyhow::bail!("device code expired; rerun `grr auth login --device`");
        }
        if let Some(storage) = auth.poll_device_code(&mut challenge).await? {
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
