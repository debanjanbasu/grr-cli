//! OAuth 2.0 Device Authorization Grant (RFC 8628).
//!
//! Headless fallback for the default PKCE loopback flow: the CLI shows a
//! URL plus a user code, the user approves in any browser (on any device),
//! and the CLI polls the token endpoint until approval lands.
//!
//! A configured client secret is sent when present; secretless PKCE
//! providers work with it absent.

use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use anyhow::anyhow;

use crate::core::error::{GrrError, Result, json_error_detail};
use tracing::warn;

use super::TokenStorage;

const DEVICE_CODE_URL: &str = "https://oauth2.googleapis.com/device/code";
const DEVICE_GRANT: &str = "urn:ietf:params:oauth:grant-type:device_code";

/// The scopes Google's device flow will actually grant.
///
/// The flow keeps its own allowlist: anything outside it comes back as
/// `invalid_scope: Invalid device flow scope: <scope>`, which fails the whole
/// request. It lives per flow, not in the Discovery index, so it cannot be
/// derived from the embedded data.
///
/// Verified against the live device endpoint on 2026-10-13 with a "TVs and
/// Limited Input devices" client — one scope at a time, because Google names
/// only the first offender. Google's own page (below) is *not* the endpoint's
/// truth: it lists `drive.appdata` and `drive.metadata.readonly`, which the
/// endpoint rejects, and omits the legacy GData scopes, which it accepts.
///
/// Source: <https://developers.google.com/identity/protocols/oauth2/limited-input-device>
const DEVICE_FLOW_SCOPES: &[&str] = &[
    "https://www.google.com/calendar/feeds",
    "https://www.google.com/m8/feeds",
    "https://www.googleapis.com/auth/calendar",
    "https://www.googleapis.com/auth/calendar.readonly",
    "https://www.googleapis.com/auth/drive.file",
    "https://www.googleapis.com/auth/userinfo.email",
    "https://www.googleapis.com/auth/userinfo.profile",
    "openid",
];

/// The consent string the device flow may send: [`super::scopes`] filtered to
/// what a TV/limited-input client can be granted.
///
/// A device login is therefore a *narrower* credential than a browser login —
/// Gmail, Chat, Docs, Sheets, Slides, Tasks, Forms, Apps Script, Analytics and
/// the admin surfaces are all outside Google's allowlist. That is Google's
/// rule, not a choice here; asking anyway would grant nothing at all.
pub(crate) fn device_flow_scopes_joined() -> String {
    super::scopes()
        .iter()
        .filter(|scope| DEVICE_FLOW_SCOPES.contains(&scope.as_str()))
        .cloned()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Pending device authorization: show `verification_url` + `user_code` to
/// the user, then poll with [`super::GoogleAuth::poll_device_code`] until done.
pub struct DeviceAuthChallenge {
    /// URL the user must open (e.g. <https://www.google.com/device>).
    pub verification_url: String,
    /// Short code the user types at `verification_url`.
    pub user_code: String,
    pub(crate) device_code: String,
    pub(crate) interval: Duration,
    pub(crate) deadline: Instant,
}

impl DeviceAuthChallenge {
    /// How long to wait before the next poll (grows on `slow_down`).
    pub fn retry_after(&self) -> Duration {
        self.interval
    }

    /// Whether the user code has expired and the flow must restart.
    pub fn is_expired(&self) -> bool {
        Instant::now() >= self.deadline
    }
}

impl super::GoogleAuth {
    /// The client the device flow must authenticate as, plus its secret.
    ///
    /// Google's device endpoint accepts one client type only — "TVs and
    /// Limited Input devices" — and answers the browser flow's Desktop
    /// client with `invalid_client: Invalid client type`, which tells a
    /// user nothing. Refusing locally, with the console steps, is the fix.
    fn device_client(&self) -> Result<(String, Option<String>)> {
        let id = self
            .config
            .device_client_id
            .as_deref()
            .map(str::trim)
            .filter(|id| !id.is_empty())
            .ok_or_else(|| {
                GrrError::Auth(
                    anyhow!(
                        "the device flow needs an OAuth client of type \"TVs and Limited Input \
                         devices\": Google rejects the Desktop client with `Invalid client type`.\n\
                         Create one in the Google Cloud console (APIs & Services → Credentials → \
                         Create credentials → OAuth client ID → TVs and Limited Input devices), \
                         then supply it as GRR_OAUTH__DEVICE_CLIENT_ID / \
                         GRR_OAUTH__DEVICE_CLIENT_SECRET, or as device-client-id / \
                         device-client-secret in the [oauth] block of ~/.grr/config.toml. \
                         Building from source can also bake it in: GRR_DEVICE_CLIENT_ID / \
                         GRR_DEVICE_CLIENT_SECRET (or a repo-root .env).\n\
                         The browser flow (`grr auth login`) needs none of this."
                    )
                    .into(),
                )
            })?;

        let secret = self
            .config
            .device_client_secret
            .as_deref()
            .map(str::trim)
            .filter(|secret| !secret.is_empty())
            .map(str::to_owned);
        Ok((id.to_owned(), secret))
    }

    /// Start a device flow: returns the challenge to display to the user.
    pub async fn request_device_code(&self) -> Result<DeviceAuthChallenge> {
        let (client_id, client_secret) = self.device_client()?;
        let scopes = device_flow_scopes_joined();
        if scopes.is_empty() {
            return Err(GrrError::Auth(
                anyhow!(
                    "no configured scope is available to Google's device flow; the browser flow \
                     (`grr auth login`) is the way in"
                )
                .into(),
            ));
        }
        // The user must know the grant is narrower than a browser login's
        // before they approve it, not after a 403 surprises them.
        warn!(
            "device flow: Google grants a TV/limited-input client only {} of this build's scopes \
             (drive.file, calendar and legacy Calendar/Contacts, userinfo, openid). Gmail, Chat, \
             Docs, Sheets, Slides, Tasks, Forms, Apps Script, Analytics and the admin APIs need \
             the browser flow (`grr auth login`).",
            scopes.split(' ').count()
        );

        let mut form = vec![("client_id", client_id), ("scope", scopes)];
        if let Some(secret) = client_secret {
            form.push(("client_secret", secret));
        }

        let response = self
            .http_client
            .post(DEVICE_CODE_URL)
            .form(&form)
            .send()
            .await
            .map_err(GrrError::Http)?;

        let status = response.status();
        let body_text = response.text().await.map_err(GrrError::Http)?;
        let body: serde_json::Value = serde_json::from_str(&body_text).map_err(|e| {
            GrrError::Auth(
                anyhow!("device endpoint returned {status}: unparseable body ({e})").into(),
            )
        })?;

        if !status.is_success() {
            // `invalid_scope` here means Google's per-flow allowlist moved:
            // the scope is one this flow cannot show, so the whole request is
            // refused. Say that, rather than repeating a bare scope URL, and
            // point at the list that has to be updated.
            let detail = json_error_detail(&body);
            let hint = if body["error"].as_str() == Some("invalid_scope") {
                "\nGoogle's device flow keeps its own scope allowlist (DEVICE_FLOW_SCOPES in \
                 src/core/auth/device.rs, verified 2026-10-13). It has moved: update that list, \
                 or use the browser flow (`grr auth login`), which has no such ceiling."
            } else {
                ""
            };
            return Err(GrrError::Auth(
                anyhow!("device endpoint returned {status}: {detail}{hint}").into(),
            ));
        }

        let device_code = body["device_code"]
            .as_str()
            .ok_or_else(|| GrrError::Auth(anyhow!("device endpoint omitted device_code").into()))?;
        let user_code = body["user_code"]
            .as_str()
            .ok_or_else(|| GrrError::Auth(anyhow!("device endpoint omitted user_code").into()))?;
        let verification_url = body
            .get("verification_url")
            .or_else(|| body.get("verification_uri"))
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| {
                GrrError::Auth(anyhow!("device endpoint omitted verification_url").into())
            })?;

        let expires_in = body["expires_in"].as_u64().unwrap_or(1800);
        let interval = body["interval"]
            .as_u64()
            .map(Duration::from_secs)
            .filter(|d| !d.is_zero())
            .unwrap_or_else(|| Duration::from_secs(5));

        Ok(DeviceAuthChallenge {
            verification_url: verification_url.to_string(),
            user_code: user_code.to_string(),
            device_code: device_code.to_string(),
            interval,
            deadline: Instant::now() + Duration::from_secs(expires_in),
        })
    }

    /// Poll once. Returns `Ok(None)` while the user has not approved yet
    /// (caller sleeps `retry_after()` and retries), `Ok(Some)` with the
    /// stored token on approval, or `Err` on denial/expiry/failure.
    pub async fn poll_device_code(
        &self,
        challenge: &mut DeviceAuthChallenge,
    ) -> Result<Option<TokenStorage>> {
        // Device polling reports status via 400 + JSON error codes, so the
        // body — not the HTTP status — drives the decision below.
        let (client_id, client_secret) = self.device_client()?;
        let (_, body) = self
            .token_request(
                client_secret.as_deref(),
                vec![
                    ("client_id", client_id),
                    ("device_code", challenge.device_code.clone()),
                    ("grant_type", DEVICE_GRANT.to_string()),
                ],
            )
            .await?;

        if let Some(code) = body["error"].as_str() {
            match code {
                "authorization_pending" => return Ok(None),
                "slow_down" => {
                    challenge.interval += Duration::from_secs(5);
                    return Ok(None);
                }
                _ => {
                    return Err(GrrError::Auth(
                        anyhow!("device poll failed: {}", json_error_detail(&body)).into(),
                    ));
                }
            }
        }

        let storage = token_storage_from_response(&body, super::scopes())?;
        *self.token_storage.write().await = Some(storage.clone());
        self.save_token(&storage).await?;
        Ok(Some(storage))
    }

    /// Fresh interactive login: drops any stored (possibly dead) token, runs
    /// the PKCE browser flow with exactly `scopes`, saves and returns the new
    /// token. The caller decides the set — see [`super::login_scopes`],
    /// [`super::scopes_for_service`] and [`super::scopes`].
    pub async fn login(&self, scopes: &[String]) -> Result<TokenStorage> {
        // Best effort: a dead on-disk token must never block fresh consent.
        let _ = self.revoke().await;
        let storage = self.run_oauth_flow(scopes).await?;
        *self.token_storage.write().await = Some(storage.clone());
        self.save_token(&storage).await?;
        Ok(storage)
    }
}

/// Build [`TokenStorage`] from a successful token-endpoint JSON body.
/// Shared by the PKCE exchange and the device poll.
///
/// `scopes` is what was *requested*; Google's response carries what was
/// actually *granted* in `scope`, and the two genuinely differ — the consent
/// screen lets a user approve a subset, and a project whose consent screen
/// does not declare a scope (Google Auth Platform → Data access) is granted
/// nothing for it, silently. Storing the request as if it were the grant
/// makes a later 403 look like a stale token instead of a scope the account
/// never approved, so the response wins whenever it says anything; the
/// requested list is only the fallback for providers that omit it.
pub(crate) fn token_storage_from_response(
    token_data: &serde_json::Value,
    scopes: &[String],
) -> Result<TokenStorage> {
    let access_token = token_data["access_token"]
        .as_str()
        .ok_or_else(|| GrrError::Auth(anyhow!("No access token in response").into()))?
        .to_string();

    let expires_in = token_data["expires_in"].as_u64().unwrap_or(3600);
    let refresh_token = token_data["refresh_token"].as_str().map(str::to_string);

    let expires_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
        + expires_in;

    let granted = token_data["scope"]
        .as_str()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| scopes.join(" "));

    Ok(TokenStorage {
        access_token,
        refresh_token,
        expires_at,
        token_type: "Bearer".to_string(),
        scope: granted,
    })
}

#[cfg(test)]
mod tests {
    use super::super::{GoogleAuth, TokenStorage};
    use crate::core::config::OAuthConfig;

    async fn auth(device: Option<(&str, &str)>) -> crate::core::error::Result<GoogleAuth> {
        let config = OAuthConfig {
            client_id: "desktop-id.apps.googleusercontent.com".into(),
            client_secret: Some("desktop-secret".into()),
            device_client_id: device.map(|(id, _)| id.to_owned()),
            device_client_secret: device.map(|(_, secret)| secret.to_owned()),
        };
        let storage = TokenStorage {
            access_token: "test-token".into(),
            refresh_token: None,
            expires_at: u64::MAX,
            token_type: "Bearer".into(),
            scope: String::new(),
        };
        GoogleAuth::with_token(config, storage).await
    }

    /// The token response says what was granted; the request only says what
    /// was asked for. Google drops scopes a project has not declared (Data
    /// access), and the consent screen lets a user approve a subset — so the
    /// stored grant is what makes a later 403 diagnosable instead of looking
    /// like a stale token.
    #[test]
    fn the_stored_scope_is_the_grant_not_the_request() -> anyhow::Result<()> {
        let requested = vec!["https://www.googleapis.com/auth/gmail.readonly".to_owned()];

        let granted = super::token_storage_from_response(
            &serde_json::json!({
                "access_token": "t",
                "expires_in": 3600,
                "scope": "openid https://www.googleapis.com/auth/userinfo.email",
            }),
            &requested,
        )?;
        assert_eq!(
            granted.scope,
            "openid https://www.googleapis.com/auth/userinfo.email"
        );

        // A response that says nothing about scope falls back to the request.
        let silent = super::token_storage_from_response(
            &serde_json::json!({ "access_token": "t", "expires_in": 3600 }),
            &requested,
        )?;
        assert_eq!(silent.scope, requested.join(" "));

        // So does a blank one (a provider placeholder must not erase the set).
        let blank = super::token_storage_from_response(
            &serde_json::json!({ "access_token": "t", "scope": "   " }),
            &requested,
        )?;
        assert_eq!(blank.scope, requested.join(" "));

        Ok(())
    }

    /// Google rejects every scope outside its device-flow allowlist with
    /// `invalid_scope` — one bad scope fails the whole request — so the device
    /// consent string is the intersection, and it is deliberately narrower
    /// than the browser one (Gmail and Chat are not in it).
    #[test]
    fn the_device_flow_requests_only_scopes_google_grants() {
        let device = super::device_flow_scopes_joined();
        let requested: Vec<&str> = device.split(' ').collect();

        // The flow stays usable: the two scopes that matter are in it.
        for useful in [
            "https://www.googleapis.com/auth/drive.file",
            "https://www.googleapis.com/auth/calendar.readonly",
        ] {
            assert!(requested.contains(&useful), "{useful} was dropped");
        }
        // Scopes Google's device endpoint rejects — one bad scope fails the
        // whole request. mail.google.com and chat.* are outside the flow's
        // allowlist entirely; the other two are on Google's *page* for the
        // flow but rejected by the endpoint itself (verified 2026-10-13).
        for rejected in [
            "https://mail.google.com/",
            "https://www.googleapis.com/auth/drive.appdata",
            "https://www.googleapis.com/auth/drive.metadata.readonly",
        ] {
            assert!(
                !requested.contains(&rejected),
                "{rejected} must never reach the device endpoint"
            );
        }
        assert!(
            !requested.iter().any(|s| s.contains("/auth/chat.")),
            "Chat is not a device-flow scope: {device}"
        );
        // Never wider than the browser consent set.
        for scope in &requested {
            assert!(
                crate::core::auth::scopes().iter().any(|s| s == scope),
                "{scope} is not in the consent set at all"
            );
        }
    }

    /// Google answers the Desktop client's device request with
    /// `invalid_client: Invalid client type` — a message that names neither
    /// the cause nor the fix. The flow refuses locally instead, naming the
    /// client type and every knob that supplies it.
    #[tokio::test]
    async fn a_missing_device_client_is_refused_locally_with_instructions() -> anyhow::Result<()> {
        let Err(err) = auth(None).await?.device_client() else {
            return Err(anyhow::anyhow!(
                "a build with no device client must refuse locally, not call Google"
            ));
        };
        let err = err.to_string();
        assert!(
            err.contains("TVs and Limited Input devices"),
            "does not name the client type: {err}"
        );
        assert!(
            err.contains("GRR_OAUTH__DEVICE_CLIENT_ID"),
            "the runtime env knob is not named: {err}"
        );
        assert!(
            err.contains("device-client-id"),
            "no config-file knob named: {err}"
        );
        Ok(())
    }

    /// The device flow authenticates as the device client, so the Desktop id
    /// never reaches Google's device endpoint (which would reject it).
    #[tokio::test]
    async fn the_configured_device_client_is_the_one_used() -> anyhow::Result<()> {
        let (id, secret) = auth(Some((
            "device-id.apps.googleusercontent.com",
            "device-secret",
        )))
        .await?
        .device_client()?;
        assert_eq!(id, "device-id.apps.googleusercontent.com");
        assert_eq!(secret.as_deref(), Some("device-secret"));
        assert_ne!(
            id, "desktop-id.apps.googleusercontent.com",
            "the browser client must not be sent to the device endpoint"
        );
        Ok(())
    }
}
