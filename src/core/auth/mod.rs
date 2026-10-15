//! OAuth2 authentication: PKCE loopback flow (RFC 8252) with an RFC 8628
//! device-flow fallback, tokens in the OS keyring.
//!
//! Multi-account: one OAuth client, one token per Google account. An auth
//! handle is bound to one account's store entry (the default account keeps
//! the legacy naming byte-for-byte); named accounts log in independently.

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::LazyLock;
use std::time::Duration;

use anyhow::anyhow;
use reqwest::Client as HttpClient;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::core::config::OAuthConfig;
use crate::core::error::{GrrError, Result};

mod device;
mod oauth;
mod server;
mod store;
mod token;

pub use device::DeviceAuthChallenge;
pub use token::TokenStorage;

use store::TokenStore;

const GOOGLE_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";

/// Loopback redirect for the native-app flow (RFC 8252 §7.3).
pub(crate) const REDIRECT_URI: &str = "http://localhost:3434/oauth/callback";

/// Every scope the embedded index names, minus the ones Google will not show
/// a user (see [`APP_AUTH_SCOPES`]), plus a curated floor for scopes no
/// method declares. This is the *superset* a credential may hold — what
/// `grr auth login --all-scopes` asks for, and what `grr api describe` counts
/// against — but deliberately not what a login requests by default: see
/// [`login_scopes`] for why the default is small. A token keeps whatever it
/// was granted, so a build that widens this list needs one `grr auth login`
/// to take effect.
pub fn scopes() -> &'static [String] {
    static SCOPES: LazyLock<Vec<String>> = LazyLock::new(|| {
        let mut set: Vec<String> = CURATED_SCOPES.iter().map(|s| (*s).to_owned()).collect();
        set.extend(crate::discovery::all_scopes());
        set.retain(|scope| !APP_AUTH_SCOPES.contains(&scope.as_str()));
        set.sort_unstable();
        set.dedup();
        set
    });
    &SCOPES
}

/// What `grr auth login` asks for when no flag says otherwise: the services
/// every grr user touches, small enough that Google's consent screen
/// completes with them ticked.
///
/// The size is the point. Google's granular-permission screen puts one
/// checkbox per requested scope, and on an unverified project its submit
/// stops completing somewhere between the 27 boxes that go through in a
/// second and the 74 that never do — no error, no redirect, just a spinner
/// until the callback times out. The whole index is ~112 scopes, so
/// requesting all of it means a first-time user cannot finish a login at all.
/// Google's own guidance for that screen says the same thing in policy terms:
/// avoid bundling scopes at sign-in, and grow the grant with incremental
/// authorization.
///
/// Everything else stays reachable: `grr auth login --service X` asks for one
/// service's scopes, `--all-scopes` for the whole index, and because every
/// login carries `include_granted_scopes` those requests *merge* with what the
/// account already holds instead of replacing it.
///
/// `https://mail.google.com/` subsumes Gmail's granular scopes, so only it is
/// requested here; the granular set stays reachable via
/// `grr auth login --service gmail` for anyone who wants a narrower grant.
const LOGIN_SCOPES: &[&str] = &[
    "https://mail.google.com/",
    "https://www.googleapis.com/auth/drive",
    "https://www.googleapis.com/auth/calendar",
    "https://www.googleapis.com/auth/calendar.events",
    "https://www.googleapis.com/auth/tasks",
    "https://www.googleapis.com/auth/documents",
    "https://www.googleapis.com/auth/spreadsheets",
    "https://www.googleapis.com/auth/presentations",
    "https://www.googleapis.com/auth/userinfo.email",
    "https://www.googleapis.com/auth/userinfo.profile",
    "openid",
];

/// [`LOGIN_SCOPES`], filtered to scopes the embedded index actually declares —
/// requesting one no build can declare is a consent hang of its own.
pub fn login_scopes() -> Vec<String> {
    let available = scopes();
    LOGIN_SCOPES
        .iter()
        .filter(|scope| available.iter().any(|known| known.as_str() == **scope))
        .map(|scope| (*scope).to_owned())
        .collect()
}

/// Every scope one service's methods declare for `grr auth login --service`.
/// The index is the same source the command tree and `grr api describe` read,
/// so the set matches exactly what that service can be asked to do — and stays
/// far below the size at which Google's consent screen stops submitting.
/// `None` when the index has no such service.
pub fn scopes_for_service(service: &str) -> Option<Vec<String>> {
    let mut set: Vec<String> = Vec::new();
    let entry = crate::discovery::services().get(service)?;
    for method in &entry.methods {
        for scope in &method.scopes {
            if !APP_AUTH_SCOPES.contains(&scope.as_str()) {
                set.push(scope.clone());
            }
        }
    }
    set.sort_unstable();
    set.dedup();
    Some(set)
}

/// Scopes every release has carried since long before the set became
/// index-derived. The index names all of them today; keeping them as a floor
/// means a discovery regression cannot silently drop a scope that an
/// existing credential and an existing user rely on.
const CURATED_SCOPES: &[&str] = &[
    // Sign-in. No method in the index declares these, but every credential
    // needs them to name the account: without `openid`/`userinfo.*` the token
    // cannot answer `grr auth status`, and a login that omits them looks
    // healthy while every profile call 403s.
    "openid",
    "https://www.googleapis.com/auth/userinfo.email",
    "https://www.googleapis.com/auth/userinfo.profile",
    // Gmail
    "https://www.googleapis.com/auth/gmail.readonly",
    "https://www.googleapis.com/auth/gmail.compose",
    "https://www.googleapis.com/auth/gmail.modify",
    "https://www.googleapis.com/auth/gmail.labels",
    "https://mail.google.com/",
    // Calendar
    "https://www.googleapis.com/auth/calendar",
    // Drive
    "https://www.googleapis.com/auth/drive",
    // People (contacts)
    "https://www.googleapis.com/auth/contacts",
    "https://www.googleapis.com/auth/contacts.other.readonly",
    // Chat
    "https://www.googleapis.com/auth/chat.messages",
    "https://www.googleapis.com/auth/chat.spaces",
    "https://www.googleapis.com/auth/chat.delete",
    "https://www.googleapis.com/auth/chat.memberships",
    "https://www.googleapis.com/auth/chat.messages.reactions",
    // Forms
    "https://www.googleapis.com/auth/forms.body",
    "https://www.googleapis.com/auth/forms.responses.readonly",
    // Search Console
    "https://www.googleapis.com/auth/webmasters.readonly",
];

/// Scopes the index names that are granted to a *service account*, never to a
/// user, and that Google therefore refuses to show on a user consent screen.
/// They are Chat's app-authentication surface: `chat.bot` "only supports app
/// authentication", the `chat.app.*` family is used by a Chat app's service
/// account (with administrator approval), and `chat.import` exists solely for
/// domain-wide delegation to a service account.
///
/// One of them in the request fails the *entire* authorization with
/// `Error 400: invalid_scope — Some requested scopes cannot be shown:
/// [...]`, granting nothing, so they must stay out of [`scopes`]. Nothing is
/// lost by leaving them out: a user credential cannot call the methods behind
/// them, because those methods authenticate as the app, not as the user.
///
/// Keep in sync with Google's Chat authentication docs
/// (<https://developers.google.com/workspace/chat/authenticate-authorize> and
/// <https://developers.google.com/workspace/chat/authorize-import>): a refresh
/// that adds another `chat.app.*` scope needs it listed here too, or login
/// stops working for everyone.
pub(crate) const APP_AUTH_SCOPES: &[&str] = &[
    "https://www.googleapis.com/auth/chat.bot",
    "https://www.googleapis.com/auth/chat.import",
    "https://www.googleapis.com/auth/chat.app.delete",
    "https://www.googleapis.com/auth/chat.app.memberships",
    "https://www.googleapis.com/auth/chat.app.memberships.readonly",
    "https://www.googleapis.com/auth/chat.app.messages.readonly",
    "https://www.googleapis.com/auth/chat.app.spaces",
    "https://www.googleapis.com/auth/chat.app.spaces.create",
    "https://www.googleapis.com/auth/chat.app.spaces.readonly",
    "https://www.googleapis.com/auth/chat.app.all.memberships.readonly",
    "https://www.googleapis.com/auth/chat.app.all.messages.readonly",
    "https://www.googleapis.com/auth/chat.app.all.spaces.readonly",
];

/// OAuth2 client with PKCE support
#[derive(Clone)]
pub struct GoogleAuth {
    config: OAuthConfig,
    http_client: HttpClient,
    token_storage: Arc<RwLock<Option<TokenStorage>>>,
    store: TokenStore,
    token_endpoint: String,
}

impl GoogleAuth {
    /// Create new GoogleAuth from config
    pub async fn new(config: OAuthConfig) -> Result<Self> {
        Self::with_store(config, TokenStore::auto().await?).await
    }

    /// Create an auth handle bound to one named account's own token
    /// (multi-account support). Same OAuth client and scopes; every store
    /// read and write targets that account's keyring entry / fallback file
    /// exclusively. The name is validated and normalized here (rejects
    /// spaces, colons, and path separators; lowercased).
    pub async fn new_for(config: OAuthConfig, account: impl Into<String>) -> Result<Self> {
        let account = store::validate_account_name(account.into().as_str())?;
        let store = TokenStore::auto_named(&account).await?;
        Self::with_store(config, store).await
    }

    async fn with_store(config: OAuthConfig, store: TokenStore) -> Result<Self> {
        let http_client = HttpClient::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| GrrError::Config(format!("Failed to create HTTP client: {}", e)))?;
        // The store is NOT read here. Reading the OS keyring raises an
        // approval prompt on unsigned binaries (macOS re-prompts per build),
        // and the constructor runs for every command — including `--dry-run`,
        // `schema`, and `api list`, which never need a credential. The first
        // read happens in `get_access_token`, the first moment a credential
        // is actually required.
        Ok(Self {
            config,
            http_client,
            token_storage: Arc::new(RwLock::new(None)),
            store,
            token_endpoint: GOOGLE_TOKEN_URL.to_string(),
        })
    }

    /// A handle bound to `account`'s own token store, reusing this handle's
    /// config (and thus the same OAuth client). The token cache is NOT
    /// shared: the new handle loads the named account's stored token.
    ///
    /// This is what `commands/auth.rs` uses to rebind a default handle when
    /// a per-command `--account` is present.
    pub async fn with_account_store(&self, account: impl Into<String>) -> Result<Self> {
        Self::new_for(self.config.clone(), account).await
    }

    /// A fresh handle on the DEFAULT account's store — today's exact entry
    /// naming, no account suffix.
    ///
    /// `grr auth login`/`status` rebuild their handle through this so a
    /// named `--account` and the default take the same code path; without
    /// an account it is behavior-identical to the original construction.
    pub async fn with_default_account_store(&self) -> Result<Self> {
        Self::with_store(self.config.clone(), TokenStore::default_named().await?).await
    }

    /// Token storage backend in use (for logs and `auth login` output).
    pub fn token_backend(&self) -> &'static str {
        self.store.backend()
    }

    /// The shared HTTP client, which speaks HTTP/3 (QUIC) with an HTTP/2
    /// fallback.
    ///
    /// Exposed for `grr api`, which issues requests to arbitrary
    /// Discovery-derived URLs. Routing those through the same client is the
    /// point: the dynamic surface gets the same transport as the curated
    /// commands rather than quietly dropping to HTTP/1.1.
    pub fn http(&self) -> &HttpClient {
        &self.http_client
    }

    /// Construct an auth handle pre-loaded with an in-memory token.
    ///
    /// Test hook (`#[doc(hidden)]`): skips OAuth and persistent storage
    /// entirely until a test explicitly installs a token path.
    #[doc(hidden)]
    pub async fn with_token(config: OAuthConfig, storage: TokenStorage) -> Result<Self> {
        let this = Self::with_store(config, TokenStore::Memory).await?;
        *this.token_storage.write().await = Some(storage);
        Ok(this)
    }

    /// Override the OAuth2 token endpoint URL (test injection hook).
    #[doc(hidden)]
    pub fn with_token_endpoint(mut self, url: impl Into<String>) -> Self {
        self.token_endpoint = url.into();
        self
    }

    /// Redirect all token persistence to an explicit file (test injection
    /// hook). Production code never touches the real credential store
    /// because tests always pass a tempdir path here.
    #[doc(hidden)]
    pub fn with_token_path(mut self, path: PathBuf) -> Self {
        self.store = TokenStore::file(path);
        self
    }

    /// Get valid access token, refreshing if necessary
    pub async fn get_access_token(&self) -> Result<String> {
        let mut storage_guard = self.token_storage.write().await;

        // First credential use in this process (the constructor never reads
        // the store — see `with_store`). A logged-in user must never fall
        // into the implicit OAuth flow just because the token was not loaded
        // yet, so the store read happens before the None-means-fresh-install
        // branch below.
        if storage_guard.is_none() {
            *storage_guard = self.store.load().await?;
        }

        if let Some(storage) = storage_guard.as_ref() {
            if !storage.is_expired() {
                debug!(
                    "Using cached access token ({}s remaining)",
                    storage.remaining_secs()
                );
                return Ok(storage.access_token.clone());
            }

            // A stored credential that cannot produce a fresh token must never
            // trigger the implicit browser flow: `run_oauth_flow` binds port
            // 3434 mid-API-call and wedges callers for the callback timeout.
            let Some(refresh_token) = storage.refresh_token.clone() else {
                return Err(GrrError::Auth(
                    anyhow!(
                        "stored token is expired and has no refresh token; rerun `grr auth login`"
                    )
                    .into(),
                ));
            };

            // A refresh request carries no scope list, and RFC 6749 §5.1
            // lets Google omit `scope` when the grant is unchanged — so the
            // response's absent scope must fall back to what the stored
            // credential already holds, never to the index superset
            // `scopes()`. A fabricated grant makes a later 403 look like a
            // stale token instead of a scope the account never approved.
            let stored_scopes: Vec<String> = storage
                .scope
                .split_whitespace()
                .map(str::to_owned)
                .collect();
            info!("Refreshing expired access token");
            match self.refresh_token(&refresh_token, &stored_scopes).await {
                Ok(new_storage) => {
                    *storage_guard = Some(new_storage.clone());
                    self.save_token(&new_storage).await?;
                    return Ok(new_storage.access_token);
                }
                Err(e) => {
                    warn!("Token refresh failed: {}", e);
                    // Keep the expired storage in place: clearing it would
                    // route subsequent callers into the implicit OAuth flow.
                    return Err(GrrError::Auth(
                        anyhow!("{e}; rerun `grr auth login`").into(),
                    ));
                }
            }
        }

        // Fresh install / implicit install path: no stored token at all. The
        // default set keeps the consent page small; widening is an explicit
        // `grr auth login --service …` / `--all-scopes` after this.
        info!("No valid token, starting OAuth flow");
        let storage = self.run_oauth_flow(&login_scopes()).await?;
        *storage_guard = Some(storage.clone());
        self.save_token(&storage).await?;
        Ok(storage.access_token)
    }

    /// POST one form to the OAuth token endpoint and parse the JSON body a
    /// single time. `client_secret` is appended when the caller names one —
    /// a parameter rather than a field read, because the device flow
    /// authenticates as a *different* client (type "TVs and Limited Input
    /// devices") than the browser flow, and one request carrying both
    /// secrets would be rejected.
    ///
    /// Returns `(status, body)` rather than making a success decision here,
    /// so each caller keeps its own logic and messages: the PKCE exchange
    /// and the refresh grant report Google's error detail on non-2xx, while
    /// the device poll reads `error` codes out of a 400 body. Three callers
    /// previously duplicated this request/parse plumbing byte for byte.
    async fn token_request(
        &self,
        client_secret: Option<&str>,
        mut form: Vec<(&str, String)>,
    ) -> Result<(reqwest::StatusCode, serde_json::Value)> {
        if let Some(secret) = client_secret.filter(|s| !s.is_empty()) {
            form.push(("client_secret", secret.to_string()));
        }

        let response = self
            .http_client
            .post(self.token_endpoint.as_str())
            .form(&form)
            .send()
            .await
            .map_err(GrrError::Http)?;

        let status = response.status();
        let body_text = response.text().await.map_err(GrrError::Http)?;
        let body: serde_json::Value = serde_json::from_str(&body_text).map_err(|e| {
            GrrError::Auth(
                anyhow!("token endpoint returned {status}: unparseable body ({e})").into(),
            )
        })?;
        Ok((status, body))
    }

    /// Refresh access token using refresh token.
    /// The secret is sent only when configured (Google mandates it even
    /// for Desktop clients; PKCE-only providers omit it entirely).
    ///
    /// `requested_scopes` is the scope the stored credential actually holds:
    /// the refresh request sends no scope list, so when Google omits `scope`
    /// from the response (RFC 6749 §5.1, unchanged grant) the stored grant is
    /// the only truthful fallback — never the index superset.
    async fn refresh_token(
        &self,
        refresh_token: &str,
        requested_scopes: &[String],
    ) -> Result<TokenStorage> {
        let (status, token_data) = self
            .token_request(
                self.config.client_secret.as_deref(),
                vec![
                    ("client_id", self.config.client_id.clone()),
                    ("refresh_token", refresh_token.to_string()),
                    ("grant_type", "refresh_token".to_string()),
                ],
            )
            .await?;

        if !status.is_success() {
            // Google reports rejection reasons (e.g. invalid_grant) in the
            // response body; surface them instead of a generic parse failure.
            return Err(GrrError::Auth(
                anyhow!(
                    "token endpoint returned {}: {}",
                    status,
                    crate::core::error::json_error_detail(&token_data)
                )
                .into(),
            ));
        }

        let mut storage = self::device::token_storage_from_response(&token_data, requested_scopes)?;

        // Google only sometimes rotates the refresh token; when the
        // response omits one, keep the stored value instead of clobbering
        // it with None (which would brick all future refreshes).
        if storage.refresh_token.is_none() {
            storage.refresh_token = Some(refresh_token.to_string());
        }

        Ok(storage)
    }

    /// Revoke current token
    pub async fn revoke(&self) -> Result<()> {
        let mut storage_guard = self.token_storage.write().await;
        *storage_guard = None;
        self.store.delete().await?;
        info!("Token revoked and removed");
        Ok(())
    }
}

pub struct AuthConfigBuilder {
    config: OAuthConfig,
}

impl AuthConfigBuilder {
    pub fn new() -> Self {
        Self {
            config: OAuthConfig::default(),
        }
    }

    /// Take the whole [`OAuthConfig`] instead of copying fields one at a
    /// time: the config carries the device-flow client too, and a
    /// field-by-field copy silently dropped it — `grr auth login --device`
    /// then refused a device client the user had configured.
    pub fn oauth(mut self, config: OAuthConfig) -> Self {
        self.config = config;
        // Empty strings behave as absent: the secret is omitted everywhere.
        self.config.client_secret = self.config.client_secret.take().filter(|s| !s.is_empty());
        self.config.device_client_secret = self
            .config
            .device_client_secret
            .take()
            .filter(|s| !s.is_empty());
        self
    }

    pub async fn build(self) -> Result<GoogleAuth> {
        GoogleAuth::new(self.config).await
    }
}

impl Default for AuthConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Auth's own checks: the consent request shape, and the default login set
/// that keeps Google's consent page submittable — small, inside [`scopes`],
/// and free of everything a personal account can never be granted.
#[cfg(test)]
mod tests {
    use super::*;

    /// The regression this guards: an app-authentication scope in the consent
    /// request makes Google reject the whole authorization — no scope is
    /// granted at all, so login cannot complete. And the filter must be
    /// surgical: Chat's *user* scopes stay in.
    #[test]
    fn the_consent_set_drops_app_authentication_scopes_only() {
        let consent = scopes();
        for scope in APP_AUTH_SCOPES {
            assert!(
                !consent.iter().any(|consented| consented == scope),
                "{scope} is app-authentication-only; Google refuses to show it and \
                 fails the entire authorization request"
            );
        }
        for scope in [
            "https://www.googleapis.com/auth/chat.messages",
            "https://www.googleapis.com/auth/chat.admin.spaces",
            "https://www.googleapis.com/auth/chat.memberships.app",
        ] {
            assert!(
                consent.iter().any(|consented| consented == scope),
                "{scope} is a user-consent scope and must stay in the request"
            );
        }
    }

    /// The default login set has to stay small enough that Google's consent
    /// screen completes with every requested scope ticked.
    #[test]
    fn the_default_login_set_is_small_and_inside_the_full_set() {
        let login = login_scopes();
        let full = scopes();

        // Past roughly 27 ticked boxes an unverified project's consent screen
        // stops submitting with no error at all, so the default has to stay
        // well below that — the number is the insurance policy.
        assert!(
            login.len() <= 20,
            "the default login set is {} scopes: Google's consent screen stops \
             submitting past ~27 ticked boxes, so this is a regression",
            login.len()
        );
        assert!(
            login.len() >= 10,
            "the default set lost its core services: {login:?}"
        );
        for scope in &login {
            assert!(
                full.iter().any(|known| known == scope),
                "{scope} is not in scopes() — requesting an undeclared scope hangs too"
            );
        }
        // Not enumerated here: admin, `groups`, and legacy GData scopes cannot
        // be granted to a personal account, only enlarge the consent page, and
        // so must never enter the default set.
    }

    #[test]
    fn service_scopes_come_from_the_index_and_stay_narrow() {
        let docs = scopes_for_service("docs").unwrap_or_default();
        assert!(!docs.is_empty(), "docs is one of the 14 services");
        assert!(
            docs.iter()
                .any(|scope| scope == "https://www.googleapis.com/auth/documents")
        );
        assert!(
            docs.len() < scopes().len(),
            "one service's scopes must stay far below the whole index"
        );
        for scope in &docs {
            assert!(
                scopes().iter().any(|known| known == scope),
                "{scope} is not in scopes()"
            );
        }
        assert!(scopes_for_service("not-a-service").is_none());
    }
}
