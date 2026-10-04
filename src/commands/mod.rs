//! CLI command modules.
//!
//! Every service command lives in `generated.rs` (compiled from the
//! Discovery index) and is dispatched by `gen_dispatch.rs`; the rest are
//! the hand-written account-level commands.

use crate::core::prelude::*;
use anyhow::Result;

/// Build a fresh `GoogleAuth` from the loaded config. One credential backs
/// every service (and `grr mcp`); the token store is shared, so this is a
/// cheap read.
pub(crate) async fn build_auth(config: &GrrConfig) -> Result<GoogleAuth> {
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

pub mod api;
pub mod ask;
pub mod auth;
pub mod gen_dispatch;
pub mod generated;
pub mod mcp;
pub mod safety;
pub mod setup;
pub mod skills;
pub mod transport;
