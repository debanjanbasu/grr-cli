//! CLI command modules.
//!
//! Every service command lives in `generated.rs` (compiled from the
//! Discovery index) and is dispatched by `gen_dispatch.rs`; the rest are
//! the hand-written account-level commands.

pub mod api;
pub mod ask;
pub mod auth;
pub mod gen_dispatch;
pub mod generated;
pub mod mcp;
pub mod safety;
pub mod setup;
pub mod transport;
