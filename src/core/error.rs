//! Error types for grr-core

use thiserror::Error;

#[derive(Error, Debug)]
pub enum GrrError {
    #[error("Authentication error: {0}")]
    Auth(Box<dyn std::error::Error + Send + Sync>),

    #[error("HTTP request error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("URL parse error: {0}")]
    Url(#[from] url::ParseError),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Rate limited: retry after {retry_after_secs}s")]
    RateLimited { retry_after_secs: u64 },

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Timeout: {0}")]
    Timeout(String),

    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
}

pub type Result<T, E = GrrError> = std::result::Result<T, E>;

pub(crate) fn json_error_detail(body: &serde_json::Value) -> String {
    let reason = body
        .get("error")
        .and_then(|value| {
            value
                .as_str()
                .or_else(|| value.get("message").and_then(serde_json::Value::as_str))
        })
        .or_else(|| body.get("message").and_then(serde_json::Value::as_str))
        .unwrap_or("unknown error");
    let description = body
        .get("error_description")
        .and_then(serde_json::Value::as_str)
        .or_else(|| body.get("description").and_then(serde_json::Value::as_str))
        .unwrap_or("");
    if description.is_empty() || description == reason {
        reason.to_string()
    } else {
        format!("{reason}: {description}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn extracts_oauth_error_details() {
        let value = json!({"error":"invalid_grant", "error_description":"expired"});
        assert_eq!(json_error_detail(&value), "invalid_grant: expired");
    }
}
