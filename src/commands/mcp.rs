//! `grr mcp` — the Model Context Protocol server. // wired in cli.rs
//!
//! The protocol implementation lives in [`crate::mcp`]; this module is the
//! clap surface (the `mcp` subcommand is registered in `src/cli.rs`) and
//! the startup path: one `GoogleAuth` built the same way `run()` does,
//! reused for every call, plus the `--readonly` safety profile the server
//! honors exactly as the CLI dispatch does.
//!
//! ```text
//! grr mcp                # serve every Discovery method over stdio
//! grr mcp --readonly     # writes refused with MCP error results
//! ```

use crate::commands::build_auth;
use crate::commands::safety::SafetyProfile;
use crate::core::prelude::*;
use anyhow::Result;
use clap::Args;

#[derive(Args, Debug, Clone)]
pub struct McpArgs {
    /// Refuse every write method (POST/PATCH/PUT/DELETE); only reads run
    #[arg(long)]
    pub readonly: bool,
}

pub async fn run_mcp(args: McpArgs) -> Result<()> {
    let config = ConfigLoader::load().await?;
    let auth = build_auth(&config).await?;
    let readonly = args.readonly;
    let profile = SafetyProfile {
        readonly,
        ..SafetyProfile::default()
    };
    let server = crate::mcp::McpServer::new(auth, profile);

    // stderr ONLY: stdout is the MCP JSON-RPC stream.
    eprintln!(
        "grr mcp: {} tools over stdio (JSON-RPC 2.0, newline-delimited){}",
        server.tool_count(),
        if readonly {
            "; --readonly: writes refused"
        } else {
            ""
        }
    );

    let reader = tokio::io::BufReader::new(tokio::io::stdin());
    let mut writer = tokio::io::stdout();
    crate::mcp::serve(&server, reader, &mut writer).await
}
