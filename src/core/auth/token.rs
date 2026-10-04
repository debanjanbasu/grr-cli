//! Token type and in-memory expiry tracking.

use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::core::error::Result;

/// Token storage with automatic refresh.
///
/// Every field carries `#[serde(default)]`: a token JSON written by an
/// older (or newer) grr must still deserialize. A field added later
/// defaults on old files; a field removed later is ignored (serde's
/// default behavior for unknown keys). Without the defaults a minimal
/// old-shape file — just `access_token` + `expires_at` — would fail.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenStorage {
    #[serde(default)]
    pub access_token: String,
    #[serde(default)]
    pub refresh_token: Option<String>,
    #[serde(default)]
    pub expires_at: u64, // Unix timestamp
    #[serde(default = "default_token_type")]
    pub token_type: String,
    #[serde(default)]
    pub scope: String,
}

/// The OAuth token type every endpoint returns; used when an old token
/// file predates the field.
fn default_token_type() -> String {
    "Bearer".to_string()
}

impl TokenStorage {
    /// Check if token is expired or about to expire (within 60 seconds)
    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        now + 60 >= self.expires_at
    }

    /// Get remaining lifetime in seconds
    pub fn remaining_secs(&self) -> u64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.expires_at.saturating_sub(now)
    }
}

impl super::GoogleAuth {
    /// Save token via the active store (OS keyring, file fallback).
    pub(crate) async fn save_token(&self, storage: &TokenStorage) -> Result<()> {
        self.store.save(storage).await?;
        debug!("Token persisted via {} store", self.store.backend());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // The core module denies `unwrap`/`expect`/`panic`, so these tests
    // return a `Result` and let `?` fail loudly.
    #[test]
    fn minimal_old_shape_token_json_still_deserializes()
    -> std::result::Result<(), Box<dyn std::error::Error>> {
        // A token file written before token_type / scope / refresh_token
        // were recorded: only access_token + expires_at are present.
        let old = r#"{"access_token":"old-token","expires_at":4102444800}"#;
        let storage: TokenStorage = serde_json::from_str(old)?;
        assert_eq!(storage.access_token, "old-token");
        assert_eq!(storage.expires_at, 4_102_444_800);
        assert_eq!(storage.refresh_token, None);
        assert_eq!(storage.token_type, "Bearer");
        assert_eq!(storage.scope, "");
        Ok(())
    }

    #[test]
    fn token_json_with_unknown_future_fields_is_ignored()
    -> std::result::Result<(), Box<dyn std::error::Error>> {
        // Forward compatibility the other way: a token written by a newer
        // grr with an extra field must not fail deserialization.
        let newer = r#"{"access_token":"t","expires_at":1,"token_type":"Bearer","scope":"s","future_field":42}"#;
        let storage: TokenStorage = serde_json::from_str(newer)?;
        assert_eq!(storage.access_token, "t");
        Ok(())
    }

    #[test]
    fn current_shape_round_trips() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let storage = TokenStorage {
            access_token: "a".into(),
            refresh_token: Some("r".into()),
            expires_at: 5,
            token_type: "Bearer".into(),
            scope: "s".into(),
        };
        let encoded = serde_json::to_string(&storage)?;
        let decoded: TokenStorage = serde_json::from_str(&encoded)?;
        assert_eq!(decoded.access_token, "a");
        assert_eq!(decoded.refresh_token.as_deref(), Some("r"));
        assert_eq!(decoded.expires_at, 5);
        assert_eq!(decoded.token_type, "Bearer");
        assert_eq!(decoded.scope, "s");
        Ok(())
    }
}
