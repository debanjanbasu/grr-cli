//! Token persistence: OS keyring by default (Windows Credential Manager,
//! macOS Keychain, Linux Secret Service via D-Bus), with a plain platform
//! file fallback for headless systems without a keyring daemon.
//!
//! Fully automatic: no configuration, no env knobs. If the OS keyring is
//! unavailable the store degrades to `<cache-dir>/grr/token.json`; a token
//! found in the fallback file is imported into the keyring and the file
//! removed only after the keyring write succeeds, so machines regain keyring
//! storage without any user action.
//!
//! Multi-account: each account gets its OWN token. The DEFAULT account (no
//! `--account`) keeps the legacy entry naming byte-for-byte; a named account
//! derives a collision-free keyring entry (`google-oauth:<name>`) and its
//! own fallback file (`token-<name>.json`), so no account can ever read or
//! clobber another's credential.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use tracing::{debug, info, warn};

use crate::core::error::{GrrError, Result};

use super::TokenStorage;

const KEYRING_SERVICE: &str = "grr";
const KEYRING_ACCOUNT: &str = "google-oauth";

/// Longest accepted account alias. The keyring account string is
/// `google-oauth:<name>`, so this bounds every platform's entry size.
const MAX_ACCOUNT_LEN: usize = 64;

/// Validate and normalize an account alias for the token store.
///
/// Rule (reject, not encode): non-empty, at most [`MAX_ACCOUNT_LEN`]
/// characters, and only ASCII letters, digits, `-`, `_`, `@`, and `.`.
/// Spaces, colons, and path separators are rejected outright — they would
/// leak into the keyring account string and the fallback filename. Aliases
/// are lowercased: two that differ only by case would collide through the
/// file fallback on case-insensitive filesystems (Windows, macOS).
pub(crate) fn validate_account_name(name: &str) -> Result<String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(GrrError::Config("account name is empty".into()));
    }
    if trimmed.len() > MAX_ACCOUNT_LEN {
        return Err(GrrError::Config(format!(
            "account name is longer than {MAX_ACCOUNT_LEN} characters"
        )));
    }
    if !trimmed
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '@' | '.'))
    {
        return Err(GrrError::Config(
            "account name may only contain ASCII letters, digits, '-', '_', '@', and '.' \
             (spaces, colons, and path separators are not allowed)"
                .into(),
        ));
    }
    Ok(trimmed.to_ascii_lowercase())
}

/// The keyring account string for a named account: the legacy account
/// string plus `:<name>`. Strictly injective — the colon guarantees a
/// named entry can never equal the default (`google-oauth`), and distinct
/// names can never produce the same string.
fn keyring_account_for(account: &str) -> String {
    format!("{KEYRING_ACCOUNT}:{account}")
}

/// Where tokens live.
#[derive(Clone)]
pub(crate) enum TokenStore {
    /// Keyring entries are cheap handles (service + account strings) and
    /// are NOT Clone in keyring v4, so we re-create the handle on demand
    /// inside the blocking task instead of holding a borrowed Entry.
    ///
    /// `account` is the keyring account field this store's entry lives
    /// under; `fallback` is the per-account file this store degrades to
    /// when the keyring write fails — never another account's file.
    ///
    /// `used` records the backend the most recent successful load/save
    /// actually touched. [`TokenStore::detect`] only probes `Entry::new`,
    /// which never contacts the daemon, so the variant is a *preference*:
    /// on a headless box a keyring-flavoured store still degrades to
    /// `fallback`, and `backend()` must report that, not the preference.
    Keyring {
        account: String,
        fallback: PathBuf,
        used: Arc<Mutex<&'static str>>,
    },
    File(PathBuf),
    Memory,
}

impl TokenStore {
    /// Auto-detect: keyring when the OS provides one, file otherwise.
    /// DEFAULT account: legacy entry naming, byte-for-byte unchanged.
    pub async fn auto() -> Result<Self> {
        Self::detect(KEYRING_ACCOUNT.to_string(), Self::fallback_path()).await
    }

    /// Auto-detect for a NAMED account: derived entry naming and its own
    /// fallback file. Callers must pass an already-validated name.
    pub async fn auto_named(account: &str) -> Result<Self> {
        Self::detect(
            keyring_account_for(account),
            Self::fallback_path_for(account),
        )
        .await
    }

    /// Auto-detect for the DEFAULT account: today's exact entry naming and
    /// the original fallback path, so an existing login is untouched.
    pub async fn default_named() -> Result<Self> {
        Self::auto().await
    }

    async fn detect(keyring_account: String, fallback: PathBuf) -> Result<Self> {
        if Self::forced_file() {
            info!("GRR_TOKEN_STORE=file: file token store, no keyring access");
            return Ok(Self::File(fallback));
        }

        let probe = keyring_account.clone();
        let result = tokio::task::spawn_blocking(move || {
            keyring::Entry::new(KEYRING_SERVICE, &probe).map(|_| ())
        })
        .await
        .map_err(|e| GrrError::Internal(e.to_string()))?;

        match result {
            Ok(()) => Ok(Self::Keyring {
                account: keyring_account,
                fallback,
                used: Arc::new(Mutex::new("os-keyring")),
            }),
            Err(e) => {
                debug!("keyring unavailable ({e}); using file token store");
                Ok(Self::File(fallback))
            }
        }
    }

    /// Explicit file store (test injection).
    pub fn file(path: PathBuf) -> Self {
        Self::File(path)
    }

    /// `GRR_TOKEN_STORE=file` forces the fallback file store instead of the
    /// OS keyring.
    ///
    /// The keyring is the better home for a token — encrypted, and gated by
    /// the OS. But macOS re-prompts for approval on every rebuild of an
    /// unsigned binary, which parks the command behind a dialog nobody is
    /// watching: fatal for a script, an agent, or a CI run. This is the
    /// switch those callers set once. Any other value (or none) keeps
    /// auto-detection.
    fn forced_file() -> bool {
        is_file_store_requested(std::env::var("GRR_TOKEN_STORE").ok().as_deref())
    }

    fn fallback_path() -> PathBuf {
        dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("grr")
            .join("token.json")
    }

    fn fallback_path_for(account: &str) -> PathBuf {
        dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("grr")
            .join(format!("token-{account}.json"))
    }

    /// Backend name for logs and `auth login` output: the backend the most
    /// recent successful load/save actually used (the detected preference
    /// before any operation touches the store).
    pub fn backend(&self) -> &'static str {
        match self {
            Self::Keyring { used, .. } => {
                *used.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
            }
            Self::File(_) => "file",
            Self::Memory => "memory",
        }
    }

    /// Load the stored token, if any.
    pub async fn load(&self) -> Result<Option<TokenStorage>> {
        match self {
            Self::Keyring {
                account,
                fallback,
                used,
            } => {
                let probe = account.clone();
                let res = tokio::task::spawn_blocking(move || {
                    keyring::Entry::new(KEYRING_SERVICE, &probe)
                        .and_then(|entry| entry.get_password())
                })
                .await
                .map_err(|e| GrrError::Internal(e.to_string()))?;
                match res {
                    Ok(secret) if !secret.trim().is_empty() => {
                        let storage: TokenStorage = serde_json::from_str(&secret)?;
                        record_backend(used, "os-keyring");
                        info!("Loaded token from OS keyring");
                        Ok(Some(storage))
                    }
                    Ok(_) => Ok(None),
                    Err(keyring::Error::NoEntry) => {
                        if let Some(storage) = Self::read_fallback_file(fallback).await? {
                            record_backend(used, "file");
                            info!("Importing token from fallback file into OS keyring");
                            if Self::write_keyring(
                                account.clone(),
                                serde_json::to_string(&storage)?,
                            )
                            .await?
                            {
                                Self::remove_fallback_file(fallback).await;
                            }
                            return Ok(Some(storage));
                        }
                        Ok(None)
                    }
                    Err(e) => {
                        warn!("keyring read failed ({e}); falling back to file token");
                        let loaded = Self::read_fallback_file(fallback).await?;
                        if loaded.is_some() {
                            record_backend(used, "file");
                        }
                        Ok(loaded)
                    }
                }
            }
            Self::File(path) => Self::read_fallback_file(path).await,
            Self::Memory => Ok(None),
        }
    }

    /// Persist the token. Keyring failures degrade to this account's own
    /// fallback file so login never fails because the keyring daemon
    /// hiccuped — and never clobbers another account's file.
    pub async fn save(&self, storage: &TokenStorage) -> Result<()> {
        let secret = serde_json::to_string(storage)?;
        match self {
            Self::Keyring {
                account,
                fallback,
                used,
            } => {
                if Self::write_keyring(account.clone(), secret.clone()).await? {
                    record_backend(used, "os-keyring");
                } else {
                    warn!("keyring write failed; writing token to fallback file");
                    Self::write_fallback_file(fallback, &secret).await?;
                    record_backend(used, "file");
                }
            }
            Self::File(path) => {
                if let Some(parent) = path.parent() {
                    tokio::fs::create_dir_all(parent).await?;
                }
                tokio::fs::write(path, &secret).await?;
            }
            Self::Memory => {}
        }
        Ok(())
    }

    /// Remove the stored credential everywhere (this account's keyring
    /// entry and this account's fallback file only).
    pub async fn delete(&self) -> Result<()> {
        match self {
            Self::Keyring {
                account, fallback, ..
            } => {
                let account = account.clone();
                let res = tokio::task::spawn_blocking(move || {
                    keyring::Entry::new(KEYRING_SERVICE, &account)
                        .and_then(|entry| entry.delete_credential())
                })
                .await
                .map_err(|e| GrrError::Internal(e.to_string()))?;
                match res {
                    Ok(()) | Err(keyring::Error::NoEntry) => {}
                    Err(e) => warn!("keyring delete failed: {e}"),
                }
                Self::remove_fallback_file(fallback).await;
            }
            Self::File(path) => match tokio::fs::remove_file(path).await {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => return Err(e.into()),
            },
            Self::Memory => {}
        }
        Ok(())
    }

    async fn write_keyring(account: String, secret: String) -> Result<bool> {
        let result = tokio::task::spawn_blocking(move || {
            keyring::Entry::new(KEYRING_SERVICE, &account)
                .and_then(|entry| entry.set_password(&secret))
        })
        .await
        .map_err(|e| GrrError::Internal(e.to_string()))?;

        match result {
            Ok(()) => Ok(true),
            Err(e) => {
                warn!("keyring write failed: {e}");
                Ok(false)
            }
        }
    }

    async fn read_fallback_file(path: &Path) -> Result<Option<TokenStorage>> {
        match tokio::fs::read_to_string(path).await {
            Ok(content) => Ok(Some(serde_json::from_str(&content)?)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    async fn write_fallback_file(path: &Path, secret: &str) -> Result<()> {
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        tokio::fs::write(path, secret).await?;
        Ok(())
    }

    async fn remove_fallback_file(path: &Path) {
        match tokio::fs::remove_file(path).await {
            Ok(()) => debug!("Removed fallback token file {:?}", path),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => warn!("Could not remove fallback token file {:?}: {e}", path),
        }
    }
}

/// Record which backend actually served the credential, so `backend()`
/// reports reality rather than [`TokenStore::detect`]'s preference.
fn record_backend(used: &Mutex<&'static str>, backend: &'static str) {
    *used.lock().unwrap_or_else(|poisoned| poisoned.into_inner()) = backend;
}

/// Whether `GRR_TOKEN_STORE` asks for the file store. Split from the env
/// read so the rule is testable without mutating process-wide state.
fn is_file_store_requested(value: Option<&str>) -> bool {
    value
        .map(|value| value.trim().eq_ignore_ascii_case("file"))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::{
        KEYRING_ACCOUNT, KEYRING_SERVICE, MAX_ACCOUNT_LEN, TokenStore, keyring_account_for,
        validate_account_name,
    };
    use crate::core::config::OAuthConfig;
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex};

    #[test]
    fn default_entry_naming_is_unchanged() {
        // An existing logged-in user must keep working with zero changes.
        assert_eq!(KEYRING_ACCOUNT, "google-oauth");
        assert_eq!(KEYRING_SERVICE, "grr");
        let fallback = TokenStore::fallback_path();
        assert!(fallback.ends_with(std::path::Path::new("grr").join("token.json")));
    }

    /// `GRR_TOKEN_STORE=file` is the escape hatch for anything that cannot
    /// answer a keyring prompt (script, agent, CI). Only `file` selects it:
    /// a typo must not silently move somebody's token to a plain file.
    #[test]
    fn only_the_word_file_selects_the_file_store() {
        assert!(super::is_file_store_requested(Some("file")));
        assert!(super::is_file_store_requested(Some("  FILE ")));
        assert!(!super::is_file_store_requested(Some("keyring")));
        assert!(!super::is_file_store_requested(Some("files")));
        assert!(!super::is_file_store_requested(Some("")));
        assert!(!super::is_file_store_requested(None));
    }

    #[test]
    fn named_entry_naming_is_derived() {
        assert_eq!(keyring_account_for("work"), "google-oauth:work");
        let fallback = TokenStore::fallback_path_for("work");
        assert!(
            fallback.ends_with(std::path::Path::new("grr").join("token-work.json")),
            "{fallback:?}"
        );
    }

    #[test]
    fn named_entries_never_collide() {
        let names = [
            "work",
            "personal",
            "a.b",
            "me@work.com",
            "x-y_z",
            "google-oauth",
        ];
        let mut keyring_entries: Vec<_> = names.iter().map(|n| keyring_account_for(n)).collect();
        keyring_entries.sort();
        keyring_entries.dedup();
        assert_eq!(keyring_entries.len(), names.len());
        // A named entry can never equal the default (the colon).
        assert!(keyring_entries.iter().all(|k| *k != KEYRING_ACCOUNT));

        let mut files: Vec<_> = names
            .iter()
            .map(|n| TokenStore::fallback_path_for(n))
            .collect();
        files.sort();
        files.dedup();
        assert_eq!(files.len(), names.len());
    }

    #[test]
    fn account_names_with_weird_characters_are_rejected() {
        for bad in [
            "",
            "  ",
            "work account",
            "work:home",
            "a/b",
            "a\\b",
            "héllo",
            "a\tb",
            "a\nb",
        ] {
            assert!(
                validate_account_name(bad).is_err(),
                "{bad:?} should be rejected"
            );
        }
        // Trimmed and lowercased: email-style aliases work, mixed case is
        // normalized so `Work` and `WORK` cannot collide on a
        // case-insensitive filesystem.
        assert!(validate_account_name("  Work ").is_ok_and(|name| name == "work"));
        assert!(validate_account_name("Me@Work.COM").is_ok_and(|name| name == "me@work.com"));
        // Length cap.
        assert!(validate_account_name(&"a".repeat(MAX_ACCOUNT_LEN + 1)).is_err());
        assert!(validate_account_name(&"a".repeat(MAX_ACCOUNT_LEN)).is_ok());
    }

    #[tokio::test]
    async fn file_stores_are_isolated_per_account() -> anyhow::Result<()> {
        let dir = tempfile::tempdir()?;
        let work = TokenStore::file(dir.path().join("token-work.json"));
        let home = TokenStore::file(dir.path().join("token-home.json"));
        let storage = test_storage("work-access");

        work.save(&storage).await?;

        // "home"'s store never returns "work"'s token...
        assert!(home.load().await.is_ok_and(|t| t.is_none()));
        // ...while "work"'s store round-trips its own.
        assert!(
            work.load()
                .await
                .is_ok_and(|t| t.as_ref().is_some_and(|s| s.access_token == "work-access"))
        );

        // Deleting "work" leaves "home" untouched.
        work.delete().await?;
        assert!(work.load().await.is_ok_and(|t| t.is_none()));
        assert!(home.load().await.is_ok_and(|t| t.is_none()));
        Ok(())
    }

    #[tokio::test]
    async fn account_bound_handle_never_reads_another_accounts_token() -> anyhow::Result<()> {
        let dir = tempfile::tempdir()?;
        let work = test_auth("work-access", dir.path().join("token-work.json")).await?;
        let home = test_auth("home-access", dir.path().join("token-home.json")).await?;

        // "work" persists its token to its own file...
        work.save_token(&test_storage("work-access")).await?;

        // ...and "home"'s store never returns it.
        assert!(home.store.load().await.is_ok_and(|t| t.is_none()));
        assert!(
            work.store
                .load()
                .await
                .is_ok_and(|t| t.as_ref().is_some_and(|s| s.access_token == "work-access"))
        );

        // Revoking "work" leaves "home" untouched.
        work.revoke().await?;
        assert!(home.store.load().await.is_ok_and(|t| t.is_none()));
        Ok(())
    }

    /// The keyring store's *variant* is only a preference: `detect` probes
    /// `Entry::new`, which never touches the daemon, so a headless box gets a
    /// keyring-flavoured store whose token actually landed in the fallback
    /// file. `backend()` must report where the credential really went.
    #[test]
    fn backend_reports_the_store_that_actually_served_the_token() {
        // Deterministic variants are always honest.
        assert_eq!(
            TokenStore::file(PathBuf::from("/tmp/token.json")).backend(),
            "file"
        );
        assert_eq!(TokenStore::Memory.backend(), "memory");

        let used = Arc::new(Mutex::new("os-keyring"));
        let store = TokenStore::Keyring {
            account: KEYRING_ACCOUNT.to_string(),
            fallback: PathBuf::from("/tmp/token.json"),
            used: Arc::clone(&used),
        };
        // Before any operation it reports the detected preference...
        assert_eq!(store.backend(), "os-keyring");
        // ...and once a save degrades to the fallback file, the truth.
        super::record_backend(&used, "file");
        assert_eq!(store.backend(), "file");
    }

    #[tokio::test]
    async fn auto_named_derives_the_entry_naming() -> anyhow::Result<()> {
        // `Entry::new` only builds a handle — no keychain access, no write.
        let store = TokenStore::auto_named("work").await?;
        let ok = match &store {
            TokenStore::Keyring { account, .. } => account == "google-oauth:work",
            // Headless systems without a keyring daemon: file fallback.
            TokenStore::File(path) => {
                path.ends_with(std::path::Path::new("grr").join("token-work.json"))
            }
            TokenStore::Memory => false,
        };
        assert!(ok);
        Ok(())
    }

    fn test_storage(access_token: &str) -> super::TokenStorage {
        super::TokenStorage {
            access_token: access_token.into(),
            refresh_token: None,
            expires_at: u64::MAX,
            token_type: "Bearer".into(),
            scope: String::new(),
        }
    }

    /// An auth handle whose persistence points at a per-account tempdir
    /// file: `with_token` installs an in-memory token without touching the
    /// real keyring, then `with_token_path` isolates the store.
    async fn test_auth(
        access_token: &str,
        token_path: PathBuf,
    ) -> anyhow::Result<super::super::GoogleAuth> {
        let config = OAuthConfig {
            client_id: "test-id.apps.googleusercontent.com".into(),
            client_secret: None,
            ..Default::default()
        };
        let storage = test_storage(access_token);
        Ok(super::super::GoogleAuth::with_token(config, storage)
            .await?
            .with_token_path(token_path))
    }
}
