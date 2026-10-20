//! `grr auth setup` — create and store an OAuth client without leaving the
//! terminal.
//!
//! This replaces the old `scripts/setup-gcp.ps1`, `setup-gcp.sh` and
//! `set-client-secret.ps1` helpers: the console links, the client-id shape
//! check and the config write all live here now, so there is one code path
//! instead of three that drift apart.
//!
//! Two audiences:
//!   - humans following the prompts (default)
//!   - agents and CI, which pass `--client-id`/`--client-secret` and
//!     `--print-only` and must never be blocked on an interactive prompt

use crate::core::config_loader::ConfigLoader;
use crate::output::{OutputFormat, print_output};
use anyhow::{Context, Result, bail};
use clap::Args;
use std::io::IsTerminal;

const CONSOLE: &str = "https://console.cloud.google.com";
const SETUP_DOCS: &str = "https://github.com/debanjanbasu/grr-cli/blob/main/docs/gcp-setup.md";

/// Each API grr can talk to: the id the embedded index uses for it
/// (`discovery::services()` keys, which is also what `grr auth login --service`
/// takes), the name `gcloud services enable` knows the API by, and the label a
/// human reads.
///
/// One `gcloud services enable` covers them all; a method's scopes decide what
/// actually authorises. This is the only list — `every_indexed_service_is_here`
/// derives the id set from the index and fails the build if the two ever
/// disagree, so a service added to the index cannot be forgotten here. The
/// order is curated anyway (Gmail first, the surfaces people set up first)
/// rather than the index's A–Z, so the recipe reads the way someone would type
/// it.
///
/// The `gcloud` name is typed rather than derived: `rootUrl` is the shared
/// `www.googleapis.com` for Calendar and Drive, and Calendar's API is the
/// legacy `calendar-json.googleapis.com` — `calendar.googleapis.com` is not an
/// API `gcloud` can enable.
const SERVICE_APIS: &[(&str, &str, &str)] = &[
    ("gmail", "gmail.googleapis.com", "Gmail"),
    ("calendar", "calendar-json.googleapis.com", "Calendar"),
    ("drive", "drive.googleapis.com", "Drive"),
    ("people", "people.googleapis.com", "Contacts"),
    ("chat", "chat.googleapis.com", "Chat"),
    ("forms", "forms.googleapis.com", "Forms"),
    ("tasks", "tasks.googleapis.com", "Tasks"),
    ("docs", "docs.googleapis.com", "Docs"),
    ("sheets", "sheets.googleapis.com", "Sheets"),
    ("slides", "slides.googleapis.com", "Slides"),
    ("script", "script.googleapis.com", "Apps Script"),
    (
        "analyticsadmin",
        "analyticsadmin.googleapis.com",
        "Analytics Admin",
    ),
    (
        "analyticsdata",
        "analyticsdata.googleapis.com",
        "Analytics Data",
    ),
    (
        "searchconsole",
        "searchconsole.googleapis.com",
        "Search Console",
    ),
];

#[derive(Args, Debug, Clone)]
pub struct SetupArgs {
    /// Google OAuth client id (…apps.googleusercontent.com). Prompted for
    /// when omitted.
    #[arg(long)]
    pub client_id: Option<String>,

    /// Google OAuth client secret. Prompted for when omitted.
    #[arg(long)]
    pub client_secret: Option<String>,

    /// Print the instructions and the resolved path, but write nothing.
    /// Useful for a first look, and for agents that only want the recipe.
    #[arg(long)]
    pub print_only: bool,

    /// Overwrite an existing ~/.grr/config.toml instead of refusing.
    #[arg(long)]
    pub force: bool,

    /// Try `gcloud services enable` for the APIs when gcloud is on PATH.
    #[arg(long)]
    pub enable_apis: bool,

    /// Output format
    #[arg(short, long, value_enum, default_value = "json")]
    pub format: OutputFormat,
}

/// Shape check: Google's client ids always end in this suffix, and a
/// truncated paste is the single most common setup mistake.
fn validate_client_id(id: &str) -> Result<()> {
    let id = id.trim();
    if id.is_empty() {
        bail!("client id is empty");
    }
    if !id.ends_with(".apps.googleusercontent.com") {
        bail!(
            "that does not look like a Google OAuth client id.\n\
             Expected something ending in .apps.googleusercontent.com, got:\n  {id}\n\
             Copy it from Google Cloud console -> Google Auth Platform -> Clients."
        );
    }
    if !id.contains('.') {
        bail!("client id looks truncated: {id}");
    }
    Ok(())
}

fn validate_client_secret(secret: &str) -> Result<()> {
    // Google desktop-app secrets are the short `GOCSPX-…` form.
    if secret.trim().is_empty() {
        bail!("client secret is empty");
    }
    if secret.trim().len() < 10 {
        bail!(
            "that client secret looks truncated ({} chars).\n\
             Copy the whole value from the OAuth client page.",
            secret.trim().len()
        );
    }
    Ok(())
}

/// Escape a value for a TOML basic string.
fn toml_escape(value: &str) -> String {
    value.replace('\\', r"\\").replace('"', "\\\"")
}

/// The config template, taken verbatim from the shipped example so the two
/// can never drift: `config.toml.example` is the single source of the full
/// shape (the `[oauth]` block, the commented `[systemone]` block, the
/// resolution-order note), and setup only fills in the client values. The
/// example is tracked and committed, so it is inside the crate package and
/// this `include_str!` always resolves.
const CONFIG_TEMPLATE: &str = include_str!("../../config.toml.example");

/// Fill the client id/secret into the example template.
///
/// This is deliberately not a hand-built TOML document: mirroring the
/// example keeps every explanatory comment and the `[systemone]` block in
/// the file the user actually reads, and a future example edit propagates.
///
/// An existing config is never rewritten in place — `handle_setup` refuses
/// without `--force`, and nothing else touches the file. That is on purpose:
/// silent rewrites are the surprise-write class of bug this codebase avoids,
/// and a config missing `[systemone]` changes nothing because that section's
/// defaults apply (see `SystemOneConfig`).
fn render_config(client_id: &str, client_secret: &str) -> String {
    CONFIG_TEMPLATE
        .replacen(
            "client_id = \"\"",
            &format!("client_id = \"{}\"", toml_escape(client_id)),
            1,
        )
        .replacen(
            "# client_secret = \"\"",
            &format!("client_secret = \"{}\"", toml_escape(client_secret)),
            1,
        )
}

fn instructions() -> String {
    let mut out = String::new();
    out.push_str("grr needs one Google OAuth client. Three steps, about five minutes:\n\n");
    out.push_str(&format!(
        "1. Create a project\n   {CONSOLE}/projectcreate\n\n"
    ));
    out.push_str(&format!(
        "2. Enable the APIs (one command if you have gcloud):\n     gcloud services enable {}\n\n",
        SERVICE_APIS
            .iter()
            .map(|(_, host, _)| *host)
            .collect::<Vec<_>>()
            .join(" ")
    ));
    out.push_str(&format!(
        "3. Create the client\n   {CONSOLE}/auth/clients/create\n   Application type: Desktop app\n   Then copy the Client ID and Client secret.\n\n"
    ));
    out.push_str("Full walkthrough: ");
    out.push_str(SETUP_DOCS);
    out.push('\n');
    out
}

pub async fn handle_setup(args: SetupArgs) -> Result<()> {
    let path = ConfigLoader::config_path()?;
    let recipe = instructions();

    if args.print_only {
        print_output(
            &serde_json::json!({
                "instructions": recipe,
                "config_path": path.display().to_string(),
                "wrote_config": false,
                "apis": SERVICE_APIS.iter().map(|(_, host, label)| serde_json::json!({"service": host, "label": label})).collect::<Vec<_>>(),
            }),
            args.format,
        )?;
        return Ok(());
    }

    // Non-interactive callers get a hard error instead of a silent hang.
    let interactive = std::io::stdin().is_terminal();

    let client_id = match args.client_id {
        Some(id) => id,
        None => {
            if !interactive {
                bail!(
                    "--client-id is required when stdin is not a terminal (CI, agents, pipes).\n\n{recipe}"
                );
            }
            prompt("Client ID (…apps.googleusercontent.com): ").await?
        }
    };
    validate_client_id(&client_id)?;

    let client_secret = match args.client_secret {
        Some(secret) => secret,
        None => {
            if !interactive {
                bail!(
                    "--client-secret is required when stdin is not a terminal (CI, agents, pipes).\n\n{recipe}"
                );
            }
            prompt("Client secret: ").await?
        }
    };
    validate_client_secret(&client_secret)?;

    if path.exists() && !args.force {
        bail!(
            "{} already exists; it will not be overwritten.\n\
             Rerun with --force to replace it.",
            path.display()
        );
    }

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating {}", parent.display()))?;
    }
    let contents = render_config(client_id.trim(), client_secret.trim());
    std::fs::write(&path, contents).with_context(|| format!("writing {}", path.display()))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600));
    }

    let mut apis_enabled = false;
    if args.enable_apis {
        match enable_apis_via_gcloud().await {
            Ok(()) => apis_enabled = true,
            Err(e) => eprintln!("warning: could not enable APIs automatically: {e}"),
        }
    }

    print_output(
        &serde_json::json!({
            "wrote_config": true,
            "config_path": path.display().to_string(),
            "client_id": client_id.trim(),
            "apis_enabled": apis_enabled,
            "next_step": "grr auth login",
        }),
        args.format,
    )?;

    Ok(())
}

/// Read one line from the terminal without pinning a runtime worker.
///
/// `handle_setup` is awaited from the async CLI path, and the read can block
/// until the user hits enter. The caller has already refused non-TTY stdin
/// (setup.rs's interactive guard), so this await cannot hang a pipe.
async fn prompt(label: &str) -> Result<String> {
    use std::io::Write;
    eprint!("{label}");
    std::io::stderr().flush().ok();
    let mut line = String::new();
    let mut stdin = tokio::io::BufReader::new(tokio::io::stdin());
    tokio::io::AsyncBufReadExt::read_line(&mut stdin, &mut line).await?;
    Ok(line.trim().to_owned())
}

/// Best-effort API enablement. The gcloud CLI is optional; everything else
/// in setup works without it.
///
/// `gcloud services enable` is a network-backed call that can take minutes;
/// spawning it through tokio keeps the runtime free while it runs.
async fn enable_apis_via_gcloud() -> Result<()> {
    let services = SERVICE_APIS
        .iter()
        .map(|(_, host, _)| *host)
        .collect::<Vec<_>>()
        .join(" ");

    let status = tokio::process::Command::new("gcloud")
        .args(["services", "enable", &services])
        .status()
        .await
        .context("gcloud is not on PATH (see https://cloud.google.com/sdk/docs/install)")?;

    if !status.success() {
        bail!("`gcloud services enable` exited with {status}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_a_real_looking_client_id() {
        assert!(validate_client_id("123-abc.apps.googleusercontent.com").is_ok());
        assert!(validate_client_id("  123-abc.apps.googleusercontent.com  ").is_ok());
    }

    #[test]
    fn rejects_a_wrong_or_truncated_client_id() {
        // The classic copy-paste failures.
        assert!(validate_client_id("123-abc").is_err());
        assert!(validate_client_id("123-abc.googleusercontent.com").is_err());
        assert!(validate_client_id("").is_err());
        assert!(validate_client_id("   ").is_err());
    }

    #[test]
    fn rejects_a_truncated_client_secret() {
        assert!(validate_client_secret("GOCSPX-abc123def456").is_ok());
        assert!(validate_client_secret("short").is_err());
        assert!(validate_client_secret("   ").is_err());
    }

    #[test]
    fn escapes_quotes_when_writing_toml() {
        assert_eq!(toml_escape(r#"a"b\c"#), r#"a\"b\\c"#);
    }

    #[test]
    fn rendered_config_mirrors_the_example_with_the_client_filled_in() {
        let rendered = render_config("123-abc.apps.googleusercontent.com", "GOCSPX-secret");
        // The full current shape, comments and all.
        assert!(rendered.contains("[oauth]"));
        assert!(rendered.contains("client_id = \"123-abc.apps.googleusercontent.com\""));
        assert!(rendered.contains("client_secret = \"GOCSPX-secret\""));
        assert!(
            rendered.contains("[systemone]"),
            "the commented systemone block must ride along"
        );
        assert!(rendered.contains("GRR_CONFIG_PATH"));
        // The empty placeholder was replaced, not duplicated.
        assert!(!rendered.contains("client_id = \"\""));
        assert!(!rendered.contains("# client_secret = \"\""));
    }

    #[test]
    fn instructions_cover_every_service() {
        let text = instructions();
        for (_, host, _) in SERVICE_APIS {
            assert!(text.contains(*host), "missing {host}");
        }
        assert!(text.contains("Desktop app"));
    }

    /// The list `grr auth setup` enables is the index's own service set: same
    /// ids, no more and no fewer. A service added to `discovery.rs` (or dropped
    /// from it) must fail here rather than silently vanish from the recipe.
    #[test]
    fn every_indexed_service_is_here() {
        let indexed: std::collections::BTreeSet<&str> =
            crate::discovery::services().keys().copied().collect();
        let listed: std::collections::BTreeSet<&str> =
            SERVICE_APIS.iter().map(|(id, _, _)| *id).collect();
        assert_eq!(
            listed, indexed,
            "SERVICE_APIS and the embedded index disagree; a service must not be \
             enabled by hand or left out of `gcloud services enable`"
        );

        for (id, host, label) in SERVICE_APIS {
            assert!(!label.is_empty(), "{id} has no human label");
            // Where the index carries a service-specific rootUrl, it must back
            // the host typed here — the two `www.googleapis.com` entries
            // (Calendar, Drive) are exactly the ones the index cannot name.
            let root = crate::discovery::services()[*id]
                .root_url
                .as_deref()
                .unwrap_or_default()
                .trim_start_matches("https://")
                .trim_end_matches('/')
                .to_owned();
            if root != "www.googleapis.com" {
                assert_eq!(*host, root, "{id}'s gcloud name disagrees with the index");
            }
        }
    }
}
