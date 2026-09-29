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

    #[serde(default)]
    pub systemone: SystemOneConfig,
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

/// System One provider configuration — the only tunable surface of the
/// natural-language entry point (`grr ask`).
///
/// Provider-agnostic by design: any endpoint speaking the System One
/// contract (POST `state` + typed `questions`, back structured `answers`)
/// works by pointing `endpoint` and `model` at it. The defaults are
/// TypeSafe's hosted endpoint and its flagship Jev model.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[derive(Default)]
pub struct SystemOneConfig {
    /// Endpoint URL. Any provider speaking the same contract works.
    #[serde(default)]
    pub endpoint: String,

    /// Model name, e.g. "jev-latest".
    #[serde(default)]
    pub model: String,

    /// API key, sent as `Authorization: Bearer`. Empty behaves as absent —
    /// the same convention as the OAuth client secret. It must never be
    /// committed; the env route (`TYPESAFE_API_KEY`) is preferred because
    /// a key in a config file can leak with the file.
    #[serde(default, alias = "api_key")]
    pub api_key: Option<String>,

    /// Method-choice confidence below which `grr ask` flags the answer.
    /// Code owns the threshold; the model supplies the probability.
    #[serde(default, alias = "confidence_threshold")]
    pub confidence_threshold: Option<f64>,
}

/// TypeSafe's hosted System One endpoint — the default `endpoint`.
pub const DEFAULT_SYSTEMONE_ENDPOINT: &str = "https://api.typesafe.ai/v1/systemone";

/// TypeSafe's flagship System One model — the default `model`.
pub const DEFAULT_SYSTEMONE_MODEL: &str = "jev-latest";

/// The method-choice confidence below which `grr ask` flags its answer.
/// A cookbook-style starting point, to be re-tuned on real requests.
pub const DEFAULT_CONFIDENCE_THRESHOLD: f64 = 0.6;

impl SystemOneConfig {
    /// The endpoint URL, filling TypeSafe's default when unset or blank.
    pub fn endpoint_or_default(&self) -> &str {
        match self.endpoint.trim() {
            "" => DEFAULT_SYSTEMONE_ENDPOINT,
            trimmed => trimmed,
        }
    }

    /// The model name, filling TypeSafe's Jev default when unset or blank.
    pub fn model_or_default(&self) -> &str {
        match self.model.trim() {
            "" => DEFAULT_SYSTEMONE_MODEL,
            trimmed => trimmed,
        }
    }

    /// The trimmed API key, or `None` when absent or blank.
    pub fn bearer_key(&self) -> Option<&str> {
        self.api_key
            .as_deref()
            .map(str::trim)
            .filter(|key| !key.is_empty())
    }

    /// The method-choice confidence threshold.
    pub fn threshold(&self) -> f64 {
        self.confidence_threshold
            .unwrap_or(DEFAULT_CONFIDENCE_THRESHOLD)
    }
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

/// Explain how to supply a System One API key, for when none could be
/// resolved. Written for someone who has never used TypeSafe: name the
/// env route first (the key cannot be committed either way), then the
/// config-file route, then the provider-agnostic escape hatch.
pub const NO_API_KEY_HELP: &str = "\
no System One API key is configured.

Pick whichever fits:

  1. Set the TYPESAFE_API_KEY environment variable (recommended: the key
     never sits in a file that can be committed).

  2. Or put it in ~/.grr/config.toml:

       [systemone]
       api-key = \"...\"

  3. Or point [systemone] endpoint/model at any provider speaking the
     same System One contract (its own key rules then apply).";

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

    #[test]
    fn systemone_section_parses_with_defaults_and_snake_case_aliases() {
        // [systemone] is the `grr ask` surface. Kebab-case keys are native;
        // snake_case aliases keep TS-era spellings working, and a config
        // with no [systemone] section at all must extract to defaults.
        let toml = r#"
[systemone]
endpoint = "https://s1.example.com/v1/systemone"
model = "my-jev-fork"
api_key = "key-123"
confidence_threshold = 0.75
"#;
        let config = Figment::new()
            .merge(Toml::string(toml))
            .extract::<GrrConfig>()
            .unwrap_or_else(|_| GrrConfig::default());
        assert_eq!(
            config.systemone.endpoint,
            "https://s1.example.com/v1/systemone"
        );
        assert_eq!(config.systemone.model, "my-jev-fork");
        assert_eq!(config.systemone.bearer_key(), Some("key-123"));
        assert_eq!(config.systemone.confidence_threshold, Some(0.75));

        let empty = GrrConfig::default();
        assert_eq!(
            empty.systemone.endpoint_or_default(),
            crate::core::config::DEFAULT_SYSTEMONE_ENDPOINT
        );
        assert_eq!(empty.systemone.model_or_default(), "jev-latest");
        assert_eq!(empty.systemone.bearer_key(), None);
        assert!((empty.systemone.threshold() - 0.6).abs() < f64::EPSILON);
    }
}
