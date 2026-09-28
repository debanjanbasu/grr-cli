#[cfg(feature = "cli")]
pub mod cli;
#[cfg(feature = "cli")]
pub mod commands;
pub mod core;
pub mod discovery;
#[cfg(feature = "cli")]
pub mod mcp;
pub mod output;
#[cfg(feature = "cli")]
pub mod schema;

#[cfg(feature = "cli")]
pub use cli::run;
