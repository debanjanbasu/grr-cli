//! Configuration loading with TOML file and environment variable support

use crate::core::config::{GrrConfig, embedded_oauth_client};
use crate::core::error::{GrrError, Result};
use figment::{
    Figment, Provider,
    providers::Format,
    providers::{Env, Toml},
};
use std::path::PathBuf;
use tracing::info;

/// Configuration loader that merges TOML file with environment variables
pub struct ConfigLoader;

impl ConfigLoader {
    /// Load configuration from TOML file with environment variable overrides
    ///
    /// Priority order (highest to lowest):
    /// 1. Environment variables (`GRR_OAUTH__CLIENT_ID` / `GRR_OAUTH__CLIENT_SECRET`)
    /// 2. TOML config file (`~/.grr/config.toml`)
    /// 3. The OAuth client compiled in at build time (release binaries)
    pub async fn load() -> Result<GrrConfig> {
        let config_path = Self::config_path()?;

        info!("Loading config from: {:?}", config_path);

        let mut config = tokio::task::spawn_blocking(move || {
            Figment::new()
                .merge(Toml::file(&config_path))
                .merge(Self::env_provider())
                .extract::<GrrConfig>()
                .map_err(|e| e.to_string())
        })
        .await
        .map_err(|e| GrrError::Config(e.to_string()))?
        .map_err(|e| GrrError::Config(e.to_string()))?;

        config = Self::apply_embedded_default(config);

        info!("Loaded config: client_id={}", config.oauth.client_id);
        match config.oauth.client_secret.as_deref() {
            Some(s) if !s.is_empty() => info!("client_secret: configured"),
            _ => info!("client_secret: absent (PKCE-only)"),
        }
        Ok(config)
    }

    /// Fill in the build-time OAuth client when nothing else supplied one.
    ///
    /// This is the zero-config path: release binaries are compiled with a
    /// client, so a fresh install can `grr auth login` with no setup at all.
    fn apply_embedded_default(config: GrrConfig) -> GrrConfig {
        Self::apply_client(config, embedded_oauth_client())
    }

    fn apply_client(mut config: GrrConfig, embedded: Option<(&str, &str)>) -> GrrConfig {
        if !config.oauth.client_id.trim().is_empty() {
            return config;
        }

        let Some((client_id, client_secret)) = embedded else {
            return config;
        };

        info!("Using the OAuth client built into this binary");
        config.oauth.client_id = client_id.to_owned();
        config.oauth.client_secret = Some(client_secret.to_owned());
        config
    }

    /// Create an Env provider that maps GRR_* env vars to kebab-case keys
    /// matching the serde rename_all = "kebab-case" setting
    fn env_provider() -> impl Provider {
        Env::raw().filter_map(|key| {
            let key = key.as_str();
            if let Some(stripped) = key.strip_prefix("GRR_") {
                // GRR_OAUTH__CLIENT_ID -> oauth.client-id
                // First replace __ with . for nesting, then _ with - for field names
                let key = stripped.replace("__", ".");
                let key = key.replace('_', "-");
                let key = key.to_ascii_lowercase();
                Some(key.into())
            } else {
                None
            }
        })
    }

    /// Determine the config file path
    ///
    /// Checks in order:
    /// 1. GRR_CONFIG_PATH environment variable
    /// 2. ~/.grr/config.toml (the only location — no legacy fallbacks)
    ///
    /// Public because `grr auth setup` writes to the same place, and a
    /// second copy of this logic would eventually drift.
    pub fn config_path() -> Result<PathBuf> {
        if let Ok(path) = std::env::var("GRR_CONFIG_PATH") {
            return Ok(PathBuf::from(path));
        }

        let home = dirs::home_dir()
            .ok_or_else(|| GrrError::Config("Could not find home directory".into()))?;

        Ok(home.join(".grr").join("config.toml"))
    }
}

#[cfg(test)]
mod resolution_order_tests {
    use super::*;
    use crate::core::config::OAuthConfig;

    fn config_with(id: &str, secret: Option<&str>) -> GrrConfig {
        GrrConfig {
            oauth: OAuthConfig {
                client_id: id.to_owned(),
                client_secret: secret.map(str::to_owned),
            },
        }
    }

    const EMBEDDED: (&str, &str) = ("embedded-id.apps.googleusercontent.com", "embedded-secret");

    #[test]
    fn embedded_client_fills_an_empty_config() {
        let resolved = ConfigLoader::apply_client(config_with("", None), Some(EMBEDDED));
        assert_eq!(resolved.oauth.client_id, EMBEDDED.0);
        assert_eq!(resolved.oauth.client_secret.as_deref(), Some(EMBEDDED.1));
    }

    #[test]
    fn a_configured_client_id_is_never_overwritten() {
        // A user-supplied client (file or GRR_OAUTH__CLIENT_ID) must win,
        // even if only the secret was left empty.
        let resolved = ConfigLoader::apply_client(
            config_with("mine.apps.googleusercontent.com", None),
            Some(EMBEDDED),
        );
        assert_eq!(resolved.oauth.client_id, "mine.apps.googleusercontent.com");
        assert!(resolved.oauth.client_secret.is_none());
    }

    #[test]
    fn a_whitespace_only_client_id_still_falls_back_to_embedded() {
        // figment can hand back "" for a missing key; a stray space should
        // not defeat the zero-config path.
        let resolved = ConfigLoader::apply_client(config_with("   ", None), Some(EMBEDDED));
        assert_eq!(resolved.oauth.client_id, EMBEDDED.0);
    }

    #[test]
    fn without_an_embedded_client_the_config_is_left_alone() {
        // Source builds with no .env: the guard in cli.rs reports the
        // actionable "no OAuth client" help instead.
        let resolved = ConfigLoader::apply_client(config_with("", None), None);
        assert!(resolved.oauth.client_id.is_empty());
        assert!(resolved.oauth.client_secret.is_none());
    }

    #[test]
    fn embedded_defaults_are_all_or_nothing() {
        // A half-set pair must not produce a client id with no secret:
        // Google rejects that token exchange with an opaque 400. This
        // assertion is conditional because whether the pair is present
        // depends on how the test binary was compiled (CI sets no .env, a
        // release build does) — the invariant is the all-or-nothing shape.
        if let Some((id, secret)) = crate::core::config::embedded_oauth_client() {
            assert!(!id.is_empty(), "id must not be blank");
            assert!(!secret.is_empty(), "secret must not be blank");
        }
    }

    #[test]
    fn the_no_client_message_names_every_route() {
        // This string is the first thing a source-build user sees, so it
        // must point at all three ways out.
        let help = crate::core::config::NO_CLIENT_HELP;
        assert!(help.contains("release"));
        assert!(help.contains(".env"));
        assert!(help.contains("auth setup"));
    }
}
