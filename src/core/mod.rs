//! grr-core — the engine behind the `grr` CLI.
//!
//! Service-agnostic: OAuth (keyring-backed), zero-config HTTP transport
//! with HTTP/3 and rate-limit-aware retry, config, and runtime probes.
//! The CLI's command surface is generated from the embedded Discovery
//! index (`crate::discovery`), not from typed per-service clients.
//!
//! - HTTP/3 with QUIC (via reqwest unstable), always compiled
//! - Auto-tuned transport: zero config, adaptive pooling and backoff
//! - OS-keyring token storage with file fallback

#![deny(unsafe_code)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]

pub mod auth;
pub mod config;
pub mod config_loader;
pub mod error;
pub mod http;
pub mod runtime;

pub use auth::{AuthConfigBuilder, DeviceAuthChallenge, GoogleAuth, TokenStorage};
pub use config::{GrrConfig, OAuthConfig};
pub use config_loader::ConfigLoader;
pub use error::{GrrError, Result};
pub use http::{
    HttpCore, TransportInfo, TransportMode, apply_transport_version, resolve_transport_mode,
};
pub use runtime::{RuntimeFeatures, detect_runtime_features};

/// Prelude for convenient imports
pub mod prelude {
    pub use crate::core::auth::{AuthConfigBuilder, DeviceAuthChallenge, GoogleAuth, TokenStorage};
    pub use crate::core::config::{GrrConfig, OAuthConfig};
    pub use crate::core::config_loader::ConfigLoader;
    pub use crate::core::error::{GrrError, Result};
    pub use crate::core::http::{
        HttpCore, TransportInfo, TransportMode, apply_transport_version, resolve_transport_mode,
    };
    pub use crate::core::runtime::{RuntimeFeatures, detect_runtime_features};
}
