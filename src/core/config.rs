//! Configuration types for grr-core.
//!
//! Zero-config by design: the only thing a user must supply is OAuth
//! credentials. Every performance knob (concurrency, pooling, timeouts,
//! compression, transport) is a compile-time constant — see
//! [`crate::core::http`].

use serde::{Deserialize, Serialize};

/// Main configuration for the client
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct GrrConfig {
    #[serde(default)]
    pub oauth: OAuthConfig,
}

/// OAuth2 configuration.
///
/// Accepts both kebab-case (Rust-native) and snake_case (TypeScript-era
/// `config.toml` migration) key spellings. Redirect URI and scopes are
/// compile-time constants — there is nothing to tune here on purpose.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[derive(Default)]
pub struct OAuthConfig {
    #[serde(alias = "client_id")]
    pub client_id: String,

    /// Optional client secret. Sent at the token endpoint only when
    /// present and non-empty. PKCE-only providers (no secret issued)
    /// work with this absent; providers that mandate a secret
    /// (currently including Google, even for Desktop clients) need it.
    #[serde(default, alias = "client_secret")]
    pub client_secret: Option<String>,
}

/// OAuth client compiled into the binary at build time, if any.
///
/// `build.rs` reads `GRR_CLIENT_ID` / `GRR_CLIENT_SECRET` from the build
/// environment (or a repo-root `.env`) and re-exports them via
/// `cargo:rustc-env`, which is what `option_env!` reads here. Release
/// binaries are built in CI with both set, so end users need no OAuth
/// client of their own; source builds fall back to `~/.grr/config.toml`.
///
/// A half-populated pair is treated as absent: Google rejects a token
/// exchange that carries an id without its secret, so falling through to
/// the actionable "no client configured" error beats a confusing 400
/// from the token endpoint.
pub(crate) fn embedded_oauth_client() -> Option<(&'static str, &'static str)> {
    let id = option_env!("GRR_CLIENT_ID")?.trim();
    let secret = option_env!("GRR_CLIENT_SECRET")?.trim();
    if id.is_empty() || secret.is_empty() {
        return None;
    }
    Some((id, secret))
}

/// Explain how to supply an OAuth client, for when none could be resolved.
///
/// Release binaries carry a compiled-in client, so this only fires for
/// source builds (`cargo install`, or a local `cargo build` with no
/// `.env`). It is written for someone who has never used Google Cloud
/// before: name all three routes and let them pick.
pub const NO_CLIENT_HELP: &str = "\
no OAuth client is configured.

Pick whichever fits:

  1. Download a release build — it ships with a client already compiled in,
     so `grr auth login` just works:
       https://grr-cli.pages.dev/install/

  2. Building from source? Put your client in a .env file next to
     Cargo.toml (see .env.example), then rebuild.

  3. Or create your own OAuth client and run `grr auth setup`, which walks
     you through it and writes ~/.grr/config.toml for you. Takes about
     five minutes: https://grr-cli.pages.dev/install/";

#[cfg(test)]
mod config_compat_tests {
    use super::*;
    use figment::{
        Figment,
        providers::{Format, Toml},
    };

    #[test]
    fn oauth_accepts_snake_case_keys_from_ts_era_configs() {
        let toml = r#"
[oauth]
client_id = "id-123"
client_secret = "secret-456"
"#;
        let config = Figment::new()
            .merge(Toml::string(toml))
            .extract::<GrrConfig>()
            .unwrap_or_else(|_| GrrConfig::default());
        assert_eq!(config.oauth.client_id, "id-123");
        assert_eq!(config.oauth.client_secret.as_deref(), Some("secret-456"));
    }

    #[test]
    fn oauth_accepts_kebab_case_keys() {
        let toml = r#"
[oauth]
client-id = "id-123"
client-secret = "secret-456"
"#;
        let config = Figment::new()
            .merge(Toml::string(toml))
            .extract::<GrrConfig>()
            .unwrap_or_else(|_| GrrConfig::default());
        assert_eq!(config.oauth.client_id, "id-123");
    }

    #[test]
    fn legacy_config_sections_are_ignored() {
        // Pre-zero-config configs carry [performance]/[output]/[runtime]
        // sections; they must still parse, just ignored.
        let toml = r#"
[oauth]
client-id = "id-123"
[performance]
max-concurrent = 99
[cache]
cache-dir = "/tmp/x"
"#;
        let config = Figment::new()
            .merge(Toml::string(toml))
            .extract::<GrrConfig>()
            .unwrap_or_else(|_| GrrConfig::default());
        assert_eq!(config.oauth.client_id, "id-123");
    }
}
