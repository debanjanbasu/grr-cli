//! grr — Google tools from the terminal, at maximum performance.
//!
//! One binary, one login, every service. The command surface has two
//! layers:
//!
//! * a thin, hand-written set of account-level commands — `auth`, `api`,
//!   `schema`, `transport` — kept as clap derive types;
//! * the ENTIRE service command tree (`gmail`, `calendar`, `drive`,
//!   `people`, `chat`, `forms`, `tasks`, `docs`, `sheets`, `slides`),
//!   generated at build time from the committed Discovery index into
//!   `commands/generated.rs`, because the index is the single source of
//!   truth for the CLI surface and it changes daily.
//!
//! Wiring: clap's derive cannot express a runtime-generated tree, and
//! `external_subcommand` would forfeit typed flags and per-leaf `--help`.
//! So [`root_command`] composes the derive's static commands with the
//! generated service commands onto ONE `Command`, parses once with
//! `get_matches()`, and dispatch matches the four static names first —
//! everything else is a generated service and goes through
//! `gen_dispatch::dispatch`, which shares the call path with `grr api`.

use crate::core::prelude::*;
use anyhow::Result;
use clap::{CommandFactory, FromArgMatches, Parser, Subcommand};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::commands::{api, ask, auth, gen_dispatch, generated, mcp, safety, transport};
use crate::schema;
use auth::AuthCommands;

/// The `--version` banner: the bare semver first (clap prefixes it with
/// `grr `), then the site mascot as a tiny ASCII crab, then the tagline —
/// all concatenated at compile time, no runtime allocation.
///
/// The semver must stay on the FIRST line: scripts/benchmark.ts reads
/// `--version`'s first line as the measured version and the workflow's
/// version gate compares it against Cargo.toml, so a crab-first layout
/// would parse as "version missing" and fail the gate open forever. The
/// Homebrew formula only greps for the version as a substring, which any
/// line order satisfies.
const VERSION_OUTPUT: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    "\n\n",
    // The mascot's idle pose (site/scripts/generate-mascot.mjs holds the
    // pixel original): wide-set glossy eyes, small smile, chunky claws.
    "      _~^~^~^~_\n",
    "  \\) / (o) (o) \\ (/\n",
    "    '_   \\_/   _'\n",
    "    \\  '-----'  /\n",
    "Google tools from the terminal, at maximum performance",
);

#[derive(Parser)]
#[command(
    name = "grr",
    version = VERSION_OUTPUT,
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
}

/// The complete parse tree: the four static commands plus every generated
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

/// Build a fresh GoogleAuth from the loaded config. One credential backs
/// every service; the token store is shared, so this is a cheap read.
async fn build_auth(config: &GrrConfig) -> Result<GoogleAuth> {
    // Fail here rather than letting an empty client_id reach Google's
    // token endpoint and come back as an opaque 400.
    if config.oauth.client_id.trim().is_empty() {
        anyhow::bail!(crate::core::config::NO_CLIENT_HELP);
    }

    Ok(AuthConfigBuilder::new()
        .client_id(config.oauth.client_id.clone())
        .client_secret(config.oauth.client_secret.clone())
        .build()
        .await?)
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

    // Parse dispatch: the four static commands first (they are few and
    // fixed); any other matched name is a generated service, which clap
    // has already validated against the registered tree.
    let Some((name, sub)) = matches.subcommand() else {
        // clap enforces `subcommand_required`; this arm exists for the
        // compiler, not for users.
        anyhow::bail!("a subcommand is required; run `grr --help`");
    };

    match name {
        // The schema dump is pure clap introspection: it must answer with
        // zero configuration, before any client (or OAuth) exists.
        "schema" => {
            let args = schema::SchemaArgs::from_arg_matches(sub)?;
            schema::handle_schema_cmd(root_command(), args)?;
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
            // Needs an auth handle but no typed service client: every URL
            // comes from the Discovery index at call time. The safety
            // profile comes from the same global flags as the tree.
            let safety_args = safety::SafetyArgs::from_arg_matches(&matches)?;
            let config = ConfigLoader::load().await?;
            let auth = build_auth(&config).await?;
            api::handle_api_cmd(&auth, cmd, &safety::SafetyProfile::from_args(&safety_args))
                .await?;
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
        _ => {
            let safety_args = safety::SafetyArgs::from_arg_matches(&matches)?;
            let config = ConfigLoader::load().await?;
            let auth = build_auth(&config).await?;
            gen_dispatch::dispatch_with_profile(
                &matches,
                &auth,
                &safety::SafetyProfile::from_args(&safety_args),
            )
            .await?;
        }
    }

    Ok(())
}
