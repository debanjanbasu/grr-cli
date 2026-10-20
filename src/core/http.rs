//! Service-agnostic HTTP core: the transport probe behind `grr transport`,
//! and the request-execution engine (`execute` plus the verb builders) it
//! hosts.
//!
//! Zero-config by construction: all tuning is compile-time constants.
//! tokio sizes OS threads to cores; the semaphore bounds only in-flight
//! HTTP requests so bursts stay under Google's per-user rate limits.

use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use reqwest::Client as ReqwestClient;
use reqwest::header::{ACCEPT_ENCODING, AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue};
use tokio::sync::Semaphore;
use tracing::{debug, info, warn};
use url::Url;

use crate::core::auth::GoogleAuth;
use crate::core::error::{GrrError, Result};

// ── Auto-tuned transport constants ────────────────────────────────────────

/// Hard ceiling on in-flight HTTP requests. Google's per-user rate limits
/// (not CPU) are the scarce resource; exceeding this burns quota into 429s.
pub(crate) const MAX_INFLIGHT_REQUESTS: usize = 64;
/// Whole-request timeout (includes server processing time).
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
/// TCP+TLS establishment only.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
/// Idle keepalive connections per host for instant reuse.
const POOL_IDLE_PER_HOST: usize = 32;
const POOL_IDLE_TIMEOUT: Duration = Duration::from_secs(90);
const TCP_KEEPALIVE: Duration = Duration::from_secs(60);
const H2_KEEPALIVE_INTERVAL: Duration = Duration::from_secs(30);
/// Transient-failure retries (connect/timeout/5xx/429).
const RETRY_ATTEMPTS: u32 = 3;
/// Exponential backoff base; grows 100ms -> 200ms -> 400ms. Also the wait a
/// 429 with no usable `Retry-After` takes, which is how that case stays
/// inside [`MAX_RATE_LIMIT_WAIT`] and therefore retryable at all.
const RETRY_BACKOFF_MS: u64 = 100;
/// Never sleep longer than this honoring a 429 Retry-After mid-command.
const MAX_RATE_LIMIT_WAIT: Duration = Duration::from_secs(30);

/// Month abbreviations of an `IMF-fixdate` (RFC 9110 §5.6.7), position + 1
/// giving the month number.
const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

/// Backoff before retry `attempt + 1`, doubling from [`RETRY_BACKOFF_MS`].
fn retry_backoff(attempt: u32) -> Duration {
    Duration::from_millis(RETRY_BACKOFF_MS * 2_u64.pow(attempt))
}

/// Seconds a 429 asks us to wait, per RFC 9110 §10.2.3: a bare
/// `delta-seconds` or an HTTP-date. `None` when the header is absent or
/// unusable — the caller then falls back to [`retry_backoff`]. A date in the
/// past means "retry now", i.e. zero.
fn parse_retry_after(value: Option<&str>, now: SystemTime) -> Option<Duration> {
    let raw = value?.trim();
    if let Ok(seconds) = raw.parse::<u64>() {
        return Some(Duration::from_secs(seconds));
    }
    let at = http_date_unix_secs(raw)?;
    let now = now.duration_since(UNIX_EPOCH).ok()?.as_secs() as i64;
    Some(Duration::from_secs(at.saturating_sub(now).max(0) as u64))
}

/// Unix seconds for an `IMF-fixdate` (`Sun, 06 Nov 1994 08:49:37 GMT`), the
/// only date format a `Retry-After` may carry today. Hand-rolled because the
/// alternative is a calendar crate for one header — the weekday is
/// redundant with the date and is not verified.
fn http_date_unix_secs(raw: &str) -> Option<i64> {
    let (_weekday, rest) = raw.split_once(", ")?;
    let mut fields = rest.split(' ');
    let day: i64 = fields.next()?.parse().ok()?;
    let month_name = fields.next()?;
    let month = MONTHS.iter().position(|m| *m == month_name)? as i64 + 1;
    let year: i64 = fields.next()?.parse().ok()?;
    let mut clock = fields.next()?.split(':');
    let hour: i64 = clock.next()?.parse().ok()?;
    let minute: i64 = clock.next()?.parse().ok()?;
    let second: i64 = clock.next()?.parse().ok()?;
    if fields.next()? != "GMT" {
        return None;
    }
    // 60 for a leap second, which RFC 9110 permits in this format.
    if !(1..=31).contains(&day) || hour > 23 || minute > 59 || second > 60 {
        return None;
    }
    Some(days_from_civil(year, month, day) * 86_400 + hour * 3600 + minute * 60 + second)
}

/// Days since 1970-01-01 for a proleptic Gregorian date — Howard Hinnant's
/// `days_from_civil`, integer-only arithmetic so no calendar crate is needed.
fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let year_of_era = year - era * 400;
    let day_of_year = (153 * ((month + 9) % 12) + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

pub(crate) fn parse_url(raw: &str, kind: &str) -> Result<Url> {
    Url::parse(raw).map_err(|error| GrrError::Config(format!("Invalid {kind} URL: {error}")))
}

#[derive(Debug, Clone)]
pub struct TransportInfo {
    pub negotiated_version: String,
    pub http3_requested: bool,
    pub http3_effective: bool,
    pub fell_back: bool,
}

impl Default for TransportInfo {
    fn default() -> Self {
        Self {
            negotiated_version: "not-probed".into(),
            http3_requested: true,
            http3_effective: false,
            fell_back: false,
        }
    }
}

/// The authenticated HTTP engine. [`HttpCore::connect`] negotiates the
/// transport version against a service base URL and records it in
/// [`TransportInfo`]; `grr transport` reports that result.
#[derive(Clone)]
pub struct HttpCore {
    auth: Arc<GoogleAuth>,
    http_client: ReqwestClient,
    semaphore: Arc<Semaphore>,
    transport_info: TransportInfo,
}

impl HttpCore {
    /// Build the core, probing the negotiated HTTP version against the
    /// service base URL (any authenticated probe path works; a dead
    /// stored token must not brick construction — `grr auth login`
    /// recovers without one).
    pub async fn connect(auth: GoogleAuth, base_url: &Url, probe_path: &str) -> Result<Self> {
        let mut info = TransportInfo {
            http3_requested: true,
            http3_effective: true,
            ..TransportInfo::default()
        };
        let mut http_client = build_http_client()?;

        match probe(
            &http_client,
            base_url,
            &auth,
            reqwest::Version::HTTP_3,
            probe_path,
        )
        .await
        {
            Ok(version) => info.negotiated_version = version,
            Err(GrrError::Auth(error)) => return Err(GrrError::Auth(error)),
            Err(error) => {
                warn!("HTTP/3 probe failed ({error}); rebuilding with HTTP/2");
                info.http3_effective = false;
                info.fell_back = true;
                http_client = build_http_client_with_mode(false)?;
                match probe(
                    &http_client,
                    base_url,
                    &auth,
                    reqwest::Version::HTTP_2,
                    probe_path,
                )
                .await
                {
                    Ok(version) => info.negotiated_version = version,
                    Err(error) => {
                        warn!("HTTP/2 probe failed ({error}); continuing unprobed");
                        info.negotiated_version = "not-probed".into();
                    }
                }
            }
        }

        info!(
            "HttpCore connected (probe {}: {}): http3_effective={}, max_inflight_requests={}",
            probe_path, info.negotiated_version, info.http3_effective, MAX_INFLIGHT_REQUESTS
        );

        Ok(Self {
            auth: Arc::new(auth),
            http_client,
            semaphore: Arc::new(Semaphore::new(MAX_INFLIGHT_REQUESTS)),
            transport_info: info,
        })
    }

    /// The underlying reqwest client (multipart uploads, raw control).
    pub fn http_client(&self) -> &ReqwestClient {
        &self.http_client
    }

    /// Transport negotiation details observed during connection.
    pub fn transport_info(&self) -> &TransportInfo {
        &self.transport_info
    }

    /// Execute a request: bearer auth, HTTP/3 version pinning, in-flight
    /// valve, and the retry policy the engine owns — a 429 honors
    /// `Retry-After` up to [`MAX_RATE_LIMIT_WAIT`] (a longer server wait is
    /// surfaced instead of slept through, and a missing or unusable header
    /// falls back to [`retry_backoff`]), a 401 fails fast with a
    /// `grr auth login` hint because the token was fetched once above the
    /// loop and a retry would re-send the same dead credential, and
    /// connect/timeout/5xx failures back off exponentially.
    ///
    /// Every other status comes back as `Ok`, including a 5xx that survived
    /// the retries: rendering a server error is the caller's business
    /// (`api.rs` names the missing scope on a 403 and quotes Google's own
    /// body for everything else), and the engine cannot know the verb and
    /// URL the caller wants in that message.
    pub async fn execute(&self, mut request: reqwest::RequestBuilder) -> Result<reqwest::Response> {
        let _permit = self
            .semaphore
            .acquire()
            .await
            .map_err(|_| GrrError::Internal("Semaphore closed".into()))?;

        let token = self.auth.get_access_token().await?;
        request = request.header(AUTHORIZATION, format!("Bearer {}", token));
        request = apply_transport_version(request, self.transport_info.http3_effective);

        let mut pending = Some(request);
        let mut last_error = None;

        for attempt in 0..=RETRY_ATTEMPTS {
            // Streaming bodies are non-replayable: try_clone() yields None and
            // only one send attempt is possible, so the builder is consumed.
            let to_send = match pending.take() {
                None => break,
                Some(builder) => match builder.try_clone() {
                    Some(replayable) => {
                        pending = Some(builder);
                        replayable
                    }
                    None => builder,
                },
            };

            match to_send.send().await {
                Ok(resp) => {
                    let status = resp.status();

                    match status.as_u16() {
                        401 => {
                            // The bearer token above was fetched once before
                            // this loop, so retrying re-sends the same stale
                            // credential and can never succeed — and clearing
                            // storage would drop callers into the implicit
                            // OAuth flow mid-request. Fail fast instead.
                            return Err(GrrError::Auth(
                                "request rejected as unauthorized (401); \
                                 your access token is expired or invalid — \
                                 rerun `grr auth login`"
                                    .into(),
                            ));
                        }
                        429 => {
                            // Rate limited. Honor the server's Retry-After and
                            // keep going when the wait is short; surface the
                            // wait immediately when it is not. No usable
                            // header means the transient backoff, which is
                            // under the cap by construction — a fixed default
                            // above it made every header-less 429 terminal.
                            let server_wait = parse_retry_after(
                                resp.headers()
                                    .get("retry-after")
                                    .and_then(|h| h.to_str().ok()),
                                SystemTime::now(),
                            );
                            let wait = server_wait.unwrap_or_else(|| retry_backoff(attempt));
                            if attempt < RETRY_ATTEMPTS && wait <= MAX_RATE_LIMIT_WAIT {
                                warn!(
                                    "rate limited; honoring a wait of {wait:?} \
                                     (attempt {}/{})",
                                    attempt + 1,
                                    RETRY_ATTEMPTS
                                );
                                tokio::time::sleep(wait).await;
                                continue;
                            }
                            // `retry_after_secs` reports what the server asked
                            // for; a header-less 429 has nothing to report.
                            return Err(GrrError::RateLimited {
                                retry_after_secs: wait.as_secs(),
                            });
                        }
                        // A 5xx is worth another attempt; the caller still
                        // needs the response when the attempts run out.
                        500..=599 if attempt < RETRY_ATTEMPTS => {}
                        _ => return Ok(resp),
                    }
                }
                Err(e) => {
                    // Only transport-level failures are transient. `e` is
                    // moved into the error, so the retryable test comes
                    // first.
                    let retryable = e.is_timeout() || e.is_connect() || e.is_request();
                    if !retryable || attempt == RETRY_ATTEMPTS {
                        return Err(GrrError::Http(e));
                    }
                    last_error = Some(GrrError::Http(e));
                }
            }

            let backoff = retry_backoff(attempt);
            debug!(
                "Request failed, retrying in {:?} (attempt {}/{})",
                backoff,
                attempt + 1,
                RETRY_ATTEMPTS
            );
            tokio::time::sleep(backoff).await;
        }

        Err(last_error.unwrap_or_else(|| GrrError::Internal("Max retries exceeded".into())))
    }
}

/// Apply the transport-level HTTP version override for data-plane requests.
///
/// reqwest only routes an individual request over HTTP/3 when the request
/// itself carries `version(HTTP_3)`; client-level `http3_prior_knowledge()`
/// merely builds the QUIC connector. When h3 is effective, every API request
/// must be explicitly versioned or it silently rides TCP.
pub fn apply_transport_version(
    builder: reqwest::RequestBuilder,
    http3_effective: bool,
) -> reqwest::RequestBuilder {
    if http3_effective {
        builder.version(reqwest::Version::HTTP_3)
    } else {
        builder
    }
}

/// Probe the negotiated HTTP version with a real authenticated request.
///
/// Any HTTP response — even a 404/401 — proves the transport negotiated
/// (some services, e.g. Forms, have no cheap "list" endpoint to hit).
/// Only transport-level failures (connect/timeout) fail the probe.
async fn probe(
    client: &ReqwestClient,
    base: &Url,
    auth: &GoogleAuth,
    version: reqwest::Version,
    probe_path: &str,
) -> std::result::Result<String, GrrError> {
    let token = auth.get_access_token().await?;
    let url = base.join(probe_path)?;
    let resp = client
        .get(url)
        .bearer_auth(&token)
        .version(version)
        .send()
        .await?;
    Ok(format!("{:?}", resp.version()))
}

/// Build HTTP client with auto-tuned transport settings.
///
/// Zero-config: every value is a constant tuned for Google frontends and
/// bounded async I/O; the tokio runtime already sizes OS threads to cores.
pub fn build_http_client() -> Result<ReqwestClient> {
    build_http_client_with_mode(true)
}

fn build_http_client_with_mode(use_http3: bool) -> Result<ReqwestClient> {
    let mut builder = ReqwestClient::builder()
        .timeout(REQUEST_TIMEOUT)
        .connect_timeout(CONNECT_TIMEOUT)
        .pool_max_idle_per_host(POOL_IDLE_PER_HOST)
        .pool_idle_timeout(POOL_IDLE_TIMEOUT)
        .tcp_keepalive(TCP_KEEPALIVE)
        .http2_keep_alive_interval(H2_KEEPALIVE_INTERVAL)
        .http2_adaptive_window(true)
        .brotli(true)
        .zstd(true)
        .gzip(true);

    builder = if use_http3 {
        builder.http3_prior_knowledge()
    } else {
        builder.http2_prior_knowledge()
    };

    let mut headers = HeaderMap::new();
    headers.insert(
        ACCEPT_ENCODING,
        HeaderValue::from_static("zstd, br, gzip, deflate"),
    );
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    builder
        .default_headers(headers)
        .build()
        .map_err(|error| GrrError::Config(format!("Failed to build HTTP client: {error}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retry_after_delta_seconds_is_read_directly() {
        let now = UNIX_EPOCH;
        assert_eq!(
            parse_retry_after(Some("5"), now),
            Some(Duration::from_secs(5))
        );
        assert_eq!(parse_retry_after(None, now), None);
        assert_eq!(parse_retry_after(Some("soon"), now), None);
    }

    #[test]
    fn retry_after_http_date_is_read_as_an_instant() {
        // RFC 9110 §10.2.3's own example: 1994-11-06T08:49:37Z.
        let header = Some("Sun, 06 Nov 1994 08:49:37 GMT");
        assert_eq!(
            parse_retry_after(header, UNIX_EPOCH),
            Some(Duration::from_secs(784_111_777))
        );
        // Seen two seconds earlier, the same date is a two-second wait.
        let now = UNIX_EPOCH + Duration::from_secs(784_111_775);
        assert_eq!(parse_retry_after(header, now), Some(Duration::from_secs(2)));
    }

    #[test]
    fn an_expired_retry_after_date_means_retry_now() {
        let now = UNIX_EPOCH + Duration::from_secs(1_700_000_000);
        assert_eq!(
            parse_retry_after(Some("Thu, 01 Jan 1970 00:00:00 GMT"), now),
            Some(Duration::ZERO)
        );
    }

    #[test]
    fn retry_after_dates_handle_a_leap_day() {
        assert_eq!(
            parse_retry_after(Some("Tue, 29 Feb 2000 00:00:00 GMT"), UNIX_EPOCH),
            Some(Duration::from_secs(951_782_400))
        );
    }

    #[test]
    fn a_malformed_retry_after_date_is_unusable_rather_than_wrong() {
        let now = UNIX_EPOCH;
        assert_eq!(
            parse_retry_after(Some("Sun, 06 Nov 1994 08:49:37 UTC"), now),
            None
        );
        assert_eq!(
            parse_retry_after(Some("Sun, 06 Foo 1994 08:49:37 GMT"), now),
            None
        );
        assert_eq!(parse_retry_after(Some("Sun, 06 Nov 1994"), now), None);
    }

    #[test]
    fn the_headerless_rate_limit_fallback_is_always_retryable() {
        // The whole point of the backoff fallback: it can never exceed the
        // cap, because a wait past the cap is surfaced instead of slept.
        for attempt in 0..=RETRY_ATTEMPTS {
            assert!(retry_backoff(attempt) <= MAX_RATE_LIMIT_WAIT);
        }
        assert_eq!(retry_backoff(0), Duration::from_millis(100));
        assert_eq!(retry_backoff(3), Duration::from_millis(800));
    }
}
