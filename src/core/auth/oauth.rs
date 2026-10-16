//! OAuth authorization flow with PKCE (RFC 7636), always on.

use base64::Engine;
use rand::Rng;
use sha2::{Digest, Sha256};
use tracing::{info, warn};
use url::Url;

use crate::core::error::{GrrError, Result};

use super::{REDIRECT_URI, TokenStorage};

impl super::GoogleAuth {
    /// Run the full OAuth2 flow with PKCE, asking for exactly `scopes`.
    ///
    /// The caller picks the set — [`super::login_scopes`] by default,
    /// [`super::scopes_for_service`] or [`super::scopes`] when a flag widens
    /// it. `include_granted_scopes` makes the request *merge* with what the
    /// account already granted, so a later, narrower login never drops access
    /// an earlier one obtained: the incremental-authorization half of
    /// Google's guidance, and what keeps every consent page small enough to
    /// submit.
    pub(crate) async fn run_oauth_flow(&self, scopes: &[String]) -> Result<TokenStorage> {
        let mut verifier_bytes = [0u8; 32];
        rand::rng().fill_bytes(&mut verifier_bytes);
        let verifier = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(verifier_bytes);
        let challenge = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .encode(Sha256::digest(verifier.as_bytes()).as_slice());

        let mut state_bytes = [0u8; 32];
        rand::rng().fill_bytes(&mut state_bytes);
        let state = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(state_bytes);

        let mut auth_url = Url::parse("https://accounts.google.com/o/oauth2/v2/auth")?;
        auth_url
            .query_pairs_mut()
            .append_pair("response_type", "code")
            .append_pair("client_id", &self.config.client_id)
            .append_pair("redirect_uri", REDIRECT_URI)
            .append_pair("scope", &scopes.join(" "))
            .append_pair("include_granted_scopes", "true")
            .append_pair("access_type", "offline")
            .append_pair("prompt", "consent")
            .append_pair("code_challenge", &challenge)
            .append_pair("code_challenge_method", "S256")
            .append_pair("state", &state)
            .finish();

        info!("Opening browser for authentication");

        let browser_url = auth_url.to_string();
        let launch_url = browser_url.clone();
        match tokio::task::spawn_blocking(move || open::that(launch_url)).await {
            Ok(Ok(())) => {}
            Ok(Err(e)) => warn!("Failed to open browser: {e}. Please manually open: {browser_url}"),
            Err(e) => warn!("Browser launcher failed: {e}. Please manually open: {browser_url}"),
        }

        let (code, returned_state) = self.start_callback_server().await?;
        if returned_state != state {
            return Err(GrrError::Auth(
                anyhow::anyhow!("OAuth callback state did not match request").into(),
            ));
        }

        let (status, token_data) = self
            .token_request(
                &self.token_endpoint,
                self.config.client_secret.as_deref(),
                vec![
                    ("code", code),
                    ("client_id", self.config.client_id.clone()),
                    ("redirect_uri", REDIRECT_URI.to_string()),
                    ("grant_type", "authorization_code".to_string()),
                    ("code_verifier", verifier),
                ],
            )
            .await?;

        if !status.is_success() {
            return Err(GrrError::Auth(
                anyhow::anyhow!(
                    "token endpoint returned {status}: {}",
                    crate::core::error::json_error_detail(&token_data)
                )
                .into(),
            ));
        }

        super::device::token_storage_from_response(&token_data, scopes)
    }
}
