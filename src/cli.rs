//! grr — Google tools from the terminal, at maximum performance.
//!
//! One binary, one login, every service. The command surface has two
//! layers:
//!
//! * a thin, hand-written set of account-level commands — `auth`, `api`,
//!   `mcp`, `transport`, `schema`, `ask`, `skills` — kept as clap derive
//!   types;
//! * the ENTIRE generated service command tree, built from the committed
//!   Discovery index into `commands/generated.rs`, because the index is
//!   the single source of truth for the CLI surface and it changes daily.
//!
//! Wiring: clap's derive cannot express a runtime-generated tree, and
//! `external_subcommand` would forfeit typed flags and per-leaf `--help`.
//! So [`root_command`] composes the derive's static commands with the
//! generated service commands onto ONE `Command`, parses once with
//! `get_matches()`, and dispatch matches the static names first —
//! everything else is a generated service and goes through
//! `gen_dispatch::dispatch_with_profile`, which shares the call path with
//! `grr api`.

use crate::core::prelude::*;
use anyhow::Result;
use clap::{CommandFactory, FromArgMatches, Parser, Subcommand};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::commands::{
    api, ask, auth, build_auth, gen_dispatch, generated, mcp, safety, skills, transport,
};
use crate::logo;
use crate::schema;
use auth::AuthCommands;

// The `--version` banner lives in `logo`: the semver, then the mascot
// rendered as well as the terminal allows, then the tagline. clap
// prefixes the whole string with `grr `, and the semver must stay on the
// FIRST line — see that module for the three consumers that parse it.

#[derive(Parser)]
#[command(
    name = "grr",
    version = logo::version_banner(),
    about = "Google tools from the terminal, at maximum performance"
)]
struct Cli {
    #[command(subcommand)]
    command: StaticCommands,
    // NOTE: no global --format flag on purpose. A global `format` collides
    // by ID with per-command `format` args of *different types*, which
    // panics clap's downcast at runtime (0xC0000409). Every subcommand
    // declares its own -f/--format.
}

#[derive(Subcommand)]
enum StaticCommands {
    /// Google account authentication (PKCE browser flow; --device for headless)
    #[command(subcommand, subcommand_required = true)]
    Auth(auth::AuthCommands),

    /// Discovery-driven access to every Google Workspace method, including
    /// the ones with no dedicated command. `grr api list` to browse.
    #[command(subcommand)]
    Api(api::ApiCommands),

    /// Start a Model Context Protocol server over stdio, exposing every
    /// grr method as a typed MCP tool. --readonly for a read-only server.
    Mcp(mcp::McpArgs),

    /// Show negotiated transport protocol and runtime features
    Transport(transport::TransportArgs),

    /// Dump the full command tree as JSON (machine-readable contract)
    Schema(schema::SchemaArgs),

    /// Natural-language entry point: a System One model (Jev by default)
    /// picks the method from the discovery catalog and fills its
    /// parameters. Prints the plan; --run executes it.
    Ask(ask::AskArgs),

    /// Install the packaged agent skills into the user-level skills
    /// directory (~/.agents/skills); list shows what is installed there
    #[command(subcommand, subcommand_required = true)]
    Skills(skills::SkillsCommands),
}

/// The complete parse tree: the seven static commands plus every generated
/// service command. `grr --help` and `grr schema` both read this, so it
/// is the single definition of the CLI surface.
///
/// The safety flags are `global(true)`: clap merges global args across
/// levels, so `--readonly` parses whether it appears before the subcommand
/// (`grr --readonly gmail …`) or after it
/// (`grr gmail users messages list --readonly`), and is readable from the
/// root matches either way.
pub(crate) fn root_command() -> clap::Command {
    Cli::command()
        .subcommands(generated::tree::commands())
        .args(safety::global_args())
}

pub async fn run() -> Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            // Default: info, but silence quinn_udp's harmless IPv6
            // network-unreachable warnings on v6-less networks (v4 wins).
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info,quinn_udp=error".into()),
        ))
        // Logs go to stderr so stdout stays pure machine-readable output
        // (`grr gmail profile | jq` must not receive log lines).
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
        .init();

    let matches = root_command().get_matches();

    // Parse dispatch: the static commands first (they are few and
    // fixed); any other matched name is a generated service, which clap
    // has already validated against the registered tree.
    let Some((name, sub)) = matches.subcommand() else {
        // clap enforces `subcommand_required`; this arm exists for the
        // compiler, not for users.
        anyhow::bail!("a subcommand is required; run `grr --help`");
    };

    // Self-migrating installs: if the skills manifest records a different grr
    // version, run the smart pass before the requested command. This is
    // LAZY — `skills::maybe_auto_migrate` reads the manifest and compares the
    // version before hashing a single file, and does nothing at all when no
    // manifest exists. Skipped for `skills` itself (it manages its own
    // install/list) and unreachable for `--help`/`--version`, which clap
    // handles and exits from inside `get_matches`. stdout stays pure: the
    // one-line summary goes to stderr.
    if name != "skills"
        && let Some(summary) = skills::maybe_auto_migrate()
    {
        eprintln!("{summary}");
    }

    match name {
        // The schema dump is pure clap introspection: it must answer with
        // zero configuration, before any client (or OAuth) exists.
        "schema" => {
            let args = schema::SchemaArgs::from_arg_matches(sub)?;
            schema::handle_schema_cmd(root_command(), args)?;
        }

        // The skills are compiled into the binary (`include_str!`), so
        // install/list need zero configuration, no OAuth client, and no
        // network — like `schema`, they answer before the config is loaded.
        "skills" => {
            skills::handle_skills_cmd(skills::SkillsCommands::from_arg_matches(sub)?)?;
        }

        // `auth setup` writes the OAuth client, so it too must run before
        // the config is loaded — otherwise a user with no client could
        // never run it.
        "auth" => {
            let cmd = AuthCommands::from_arg_matches(sub)?;
            if let AuthCommands::Setup(args) = &cmd {
                return crate::commands::setup::handle_setup(args.clone()).await;
            }
            let config = ConfigLoader::load().await?;
            let auth = build_auth(&config).await?;
            auth::handle_auth_cmd(&auth, cmd).await?;
        }

        // The transport probe needs an auth handle (it issues a real
        // authenticated request) but no typed service client.
        "transport" => {
            let config = ConfigLoader::load().await?;
            let auth = build_auth(&config).await?;
            transport::handle_transport_cmd(auth, transport::TransportArgs::from_arg_matches(sub)?)
                .await?;
        }

        "api" => {
            let cmd = api::ApiCommands::from_arg_matches(sub)?;
            // `api list` / `describe` / `refresh` are pure introspection over
            // the embedded index: they must work with zero configuration —
            // a fresh CI runner has no OAuth client, and these commands
            // would otherwise die at the client gate before doing anything.
            // `call` is credential-OPTIONAL the same way: --dry-run is a
            // plan-only operation, and a real send with no client reports
            // the actionable help from the call path.
            if matches!(cmd, api::ApiCommands::Call(_)) {
                let safety_args = safety::SafetyArgs::from_arg_matches(&matches)?;
                let config = ConfigLoader::load().await?;
                let auth = build_auth(&config).await.ok();
                api::handle_api_cmd(
                    auth.as_ref(),
                    cmd,
                    &safety::SafetyProfile::from_args(&safety_args),
                )
                .await?;
            } else {
                // Permissive profile: list/describe/refresh cannot write.
                api::handle_api_cmd(None, cmd, &safety::SafetyProfile::PERMISSIVE).await?;
            }
        }

        // The MCP server is self-contained: it loads the config, builds its
        // own auth handle, and serves the JSON-RPC stream itself.
        "mcp" => {
            let args = mcp::McpArgs::from_arg_matches(sub)?;
            mcp::run_mcp(args).await?;
        }

        // The ask flow plans by default (no credential needed); --run
        // builds the auth handle and executes through the shared call
        // path. The safety gate fires as soon as the method is resolved.
        "ask" => {
            let args = ask::AskArgs::from_arg_matches(sub)?;
            let config = ConfigLoader::load().await?;
            let auth = if args.run {
                Some(build_auth(&config).await?)
            } else {
                None
            };
            let safety_args = safety::SafetyArgs::from_arg_matches(&matches)?;
            ask::handle_ask(
                &config,
                auth.as_ref(),
                args,
                &safety::SafetyProfile::from_args(&safety_args),
            )
            .await?;
        }

        // A generated service. Dispatch walks the matched chain itself
        // (the deepest subcommand's name IS the full dotted method id),
        // so it gets the full root matches. The safety profile comes from
        // the global --readonly/--deny-service/--deny-verb flags.
        //
        // The credential is OPTIONAL: --dry-run plans never consult it
        // (call_method requires it only when actually sending), so a fresh
        // CI runner — no OAuth client, no config — can exercise the whole
        // tree offline. A real send with no client gets the actionable
        // NO_CLIENT_HELP from the call path.
        _ => {
            let safety_args = safety::SafetyArgs::from_arg_matches(&matches)?;
            let config = ConfigLoader::load().await?;
            let auth = build_auth(&config).await.ok();
            gen_dispatch::dispatch_with_profile(
                &matches,
                auth.as_ref(),
                &safety::SafetyProfile::from_args(&safety_args),
            )
            .await?;
        }
    }

    Ok(())
}
