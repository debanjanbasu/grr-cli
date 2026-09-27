use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

const CLIENT_ID: &str = "GRR_CLIENT_ID";
const CLIENT_SECRET: &str = "GRR_CLIENT_SECRET";

fn main() {
    println!("cargo:rerun-if-env-changed=RUSTC");
    println!("cargo:rerun-if-env-changed={CLIENT_ID}");
    println!("cargo:rerun-if-env-changed={CLIENT_SECRET}");
    println!("cargo:rerun-if-changed=build.rs");

    require_nightly();

    inject_default_oauth_client();
}

/// grr needs Rust nightly: HTTP/3 (rustls + quinn) needs reqwest's
/// unstable `http3` feature, which requires `-Z`-era nightly APIs.
fn require_nightly() {
    let rustc = env::var_os("RUSTC").unwrap_or_else(|| "rustc".into());
    let Ok(output) = Command::new(rustc).arg("--version").output() else {
        return;
    };
    if !output.status.success() {
        return;
    }

    let version = String::from_utf8_lossy(&output.stdout);
    let Some(toolchain) = version.split_whitespace().nth(1) else {
        return;
    };
    if !toolchain.contains("-nightly") {
        panic!(
            "grr requires Rust nightly: HTTP/3 (rustls + quinn) needs reqwest's unstable `http3` feature, which requires `-Z`-era nightly APIs. Install it with `rustup toolchain install nightly`."
        );
    }
}

/// Compile the default OAuth client into the binary when the build
/// environment provides one, so release binaries need zero configuration.
///
/// Values come from the process environment first, then a repo-root `.env`
/// (the local-development path documented in `.env.example`). They are
/// emitted as `rustc-env` so `option_env!` in `core::config` can read them.
/// Nothing is ever printed: build logs are world-readable on CI, and an
/// installed-app client secret is a published value in shipped binaries
/// anyway, but it must not leak into build logs or the repository.
fn inject_default_oauth_client() {
    let env_path = Path::new(".env");
    if env_path.exists() {
        println!("cargo:rerun-if-changed=.env");
    }

    let dotenv = read_dotenv(env_path);
    let client_id = lookup(CLIENT_ID, &dotenv);
    let client_secret = lookup(CLIENT_SECRET, &dotenv);

    match (&client_id, &client_secret) {
        (Some(_), Some(_)) => {}
        (Some(_), None) | (None, Some(_)) => {
            println!(
                "cargo:warning={CLIENT_ID}/{CLIENT_SECRET} is only partially set; the embedded default OAuth client will be ignored. Set both, or neither."
            );
        }
        (None, None) => {}
    }

    if let Some(value) = &client_id {
        println!("cargo:rustc-env={CLIENT_ID}={value}");
    }
    if let Some(value) = &client_secret {
        println!("cargo:rustc-env={CLIENT_SECRET}={value}");
    }

    if client_id.is_some() && client_secret.is_some() {
        println!("cargo:warning=embedded default OAuth client: configured");
    } else {
        println!(
            "cargo:warning=embedded default OAuth client: not configured (users supply their own client id via `grr auth setup`, or GRR_OAUTH__* env vars)"
        );
    }
}

fn lookup(key: &str, dotenv: &[(String, String)]) -> Option<String> {
    let value = env::var(key)
        .ok()
        .filter(|v| !v.trim().is_empty())
        .or_else(|| {
            dotenv
                .iter()
                .find(|(k, _)| k == key)
                .map(|(_, v)| v.clone())
        })
        .filter(|v| !v.trim().is_empty())?;
    Some(value.trim().to_owned())
}

/// Minimal `.env` reader: `KEY=VALUE` per line, `#` comments, optional
/// matching quotes. Deliberately hand-rolled so the build script keeps its
/// zero-dependency guarantee.
fn read_dotenv(path: &Path) -> Vec<(String, String)> {
    let Ok(contents) = fs::read_to_string(path) else {
        return Vec::new();
    };

    contents
        .lines()
        .filter_map(|line| {
            let line = line.trim().trim_start_matches('\u{feff}');
            if line.is_empty() || line.starts_with('#') {
                return None;
            }
            let (key, value) = line.split_once('=')?;
            let key = key.trim();
            if key.is_empty() {
                return None;
            }
            let value = value.trim();
            let value = value
                .strip_prefix('"')
                .and_then(|v| v.strip_suffix('"'))
                .or_else(|| value.strip_prefix('\'').and_then(|v| v.strip_suffix('\'')))
                .unwrap_or(value);
            Some((key.to_owned(), value.to_owned()))
        })
        .collect()
}
