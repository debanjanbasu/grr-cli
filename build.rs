use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

const CLIENT_ID: &str = "GRR_CLIENT_ID";
const CLIENT_SECRET: &str = "GRR_CLIENT_SECRET";
/// The device flow (`grr auth login --device`) authenticates as a client of
/// type "TVs and Limited Input devices"; Google's device endpoint rejects the
/// Desktop client above. Absent means `--device` refuses locally with setup
/// instructions — the browser flow is unaffected.
const DEVICE_CLIENT_ID: &str = "GRR_DEVICE_CLIENT_ID";
const DEVICE_CLIENT_SECRET: &str = "GRR_DEVICE_CLIENT_SECRET";

fn main() {
    println!("cargo:rerun-if-env-changed=RUSTC");
    println!("cargo:rerun-if-env-changed={CLIENT_ID}");
    println!("cargo:rerun-if-env-changed={CLIENT_SECRET}");
    println!("cargo:rerun-if-env-changed={DEVICE_CLIENT_ID}");
    println!("cargo:rerun-if-env-changed={DEVICE_CLIENT_SECRET}");
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

    let device_id = lookup(DEVICE_CLIENT_ID, &dotenv);
    let device_secret = lookup(DEVICE_CLIENT_SECRET, &dotenv);
    match (&device_id, &device_secret) {
        (Some(_), None) | (None, Some(_)) => {
            println!(
                "cargo:warning={DEVICE_CLIENT_ID}/{DEVICE_CLIENT_SECRET} is only partially set; the embedded device-flow client will be ignored. Set both, or neither."
            );
        }
        _ => {}
    }
    if let Some(value) = &device_id {
        println!("cargo:rustc-env={DEVICE_CLIENT_ID}={value}");
    }
    if let Some(value) = &device_secret {
        println!("cargo:rustc-env={DEVICE_CLIENT_SECRET}={value}");
    }

    if client_id.is_some() && client_secret.is_some() {
        let device = if device_id.is_some() && device_secret.is_some() {
            "configured"
        } else {
            "absent (`grr auth login --device` explains the client type it needs)"
        };
        println!(
            "cargo:warning=embedded default OAuth client: configured; device-flow client: {device}"
        );
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
