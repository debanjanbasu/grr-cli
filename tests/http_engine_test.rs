//! Behaviour of the shared request engine, `HttpCore` — the one every
//! generated command, `grr api call`, `grr auth status`, `grr ask --run` and
//! the MCP `tools/call` rides. These pin what a caller can observe from
//! outside: the bearer header, the 401 hint, and the three `Retry-After`
//! outcomes (waited out, capped, unusable).
//!
//! `wiremock` stands in for Google: it auto-detects HTTP/2, which is what the
//! engine's fallback client speaks once the HTTP/3 probe fails against a
//! plain-TCP mock. The engine is built through `HttpCore::connect`, so the
//! transport probe is one of the requests the mock sees — which is why every
//! mock is scoped to `CALL_PATH` and never to the probe's `/`.

use std::time::{Duration, Instant};

use grr_cli::core::http::HttpCore;
use grr_cli::core::{GoogleAuth, GrrConfig, GrrError, TokenStorage};
use serde_json::json;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const ACCESS_TOKEN: &str = "test-access-token";
const AUTH_HEADER: &str = "Bearer test-access-token";
/// The path the tests call. The probe goes to `/`, so nothing here counts it.
const CALL_PATH: &str = "/v1/call";

fn storage() -> TokenStorage {
    TokenStorage {
        access_token: ACCESS_TOKEN.into(),
        refresh_token: Some("test-refresh-token".into()),
        // Far in the future: the engine must use this token and never call the
        // token endpoint.
        expires_at: 4_000_000_000,
        token_type: "Bearer".into(),
        scope: "test-scope".into(),
    }
}

/// An engine whose transport probe lands on `server`, so the tests need no
/// network at all.
async fn engine(server: &MockServer) -> HttpCore {
    let auth = GoogleAuth::with_token(GrrConfig::default().oauth.clone(), storage())
        .await
        .unwrap()
        .with_token_endpoint(format!("{}/token", server.uri()));
    let base = url::Url::parse(&format!("{}/", server.uri())).unwrap();
    HttpCore::connect(auth, &base, "").await.unwrap()
}

/// One call through the engine, timed so a test can tell a wait apart from a
/// range limit surfaced without waiting.
async fn call_through_the_engine(
    server: &MockServer,
) -> (Duration, Result<reqwest::Response, GrrError>) {
    let engine = engine(server).await;
    let url = url::Url::parse(&format!("{}{CALL_PATH}", server.uri())).unwrap();
    let started = Instant::now();
    let outcome = engine.execute(engine.get(url)).await;
    (started.elapsed(), outcome)
}

/// Responses are consumed in the order they are mounted: the mock with the
/// lower priority answers the first attempt, then stops matching.
async fn mock_first_response(server: &MockServer, response: ResponseTemplate) {
    Mock::given(method("GET"))
        .and(path(CALL_PATH))
        .respond_with(response)
        .up_to_n_times(1)
        .with_priority(1)
        .mount(server)
        .await;
}

async fn mock_success(server: &MockServer) {
    Mock::given(method("GET"))
        .and(path(CALL_PATH))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"ok": true})))
        .mount(server)
        .await;
}

/// How many calls reached `CALL_PATH` — i.e. attempts the engine made. The
/// probe's own request is on `/` and is not counted.
async fn attempts(server: &MockServer) -> usize {
    server
        .received_requests()
        .await
        .unwrap()
        .iter()
        .filter(|request| request.url.path() == CALL_PATH)
        .count()
}

#[tokio::test]
async fn the_bearer_header_reaches_the_wire() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(CALL_PATH))
        .and(header("authorization", AUTH_HEADER))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"ok": true})))
        .mount(&server)
        .await;

    let (_, outcome) = call_through_the_engine(&server).await;
    // The mock only answers an authenticated request, so a 200 is already
    // evidence; the recorded request names the credential that was sent.
    assert_eq!(outcome.unwrap().status(), 200);

    let received = server.received_requests().await.unwrap();
    let call = received
        .iter()
        .find(|request| request.url.path() == CALL_PATH)
        .unwrap();
    assert_eq!(
        call.headers.get("authorization").unwrap().to_str().unwrap(),
        AUTH_HEADER
    );
}

#[tokio::test]
async fn a_401_fails_fast_with_the_login_hint() {
    let server = MockServer::start().await;
    mock_first_response(
        &server,
        ResponseTemplate::new(401).set_body_json(json!({
            "error": {"code": 401, "message": "Invalid Credentials"}
        })),
    )
    .await;

    let (_, outcome) = call_through_the_engine(&server).await;
    let message = outcome.unwrap_err().to_string();
    assert!(message.contains("401"), "{message}");
    assert!(message.contains("grr auth login"), "{message}");
    // The token is fetched once above the retry loop, so a retry would
    // re-send the same stale credential: exactly one attempt.
    assert_eq!(attempts(&server).await, 1);
}

#[tokio::test]
async fn a_retry_after_of_seconds_is_waited_out_and_the_call_retried() {
    let server = MockServer::start().await;
    mock_first_response(
        &server,
        ResponseTemplate::new(429).insert_header("retry-after", "1"),
    )
    .await;
    mock_success(&server).await;

    let (elapsed, outcome) = call_through_the_engine(&server).await;
    assert_eq!(outcome.unwrap().status(), 200);
    assert_eq!(attempts(&server).await, 2);
    assert!(
        elapsed >= Duration::from_secs(1),
        "the 1s Retry-After was not honored: {elapsed:?}"
    );
}

#[tokio::test]
async fn a_429_without_retry_after_retries_within_the_cap() {
    let server = MockServer::start().await;
    mock_first_response(&server, ResponseTemplate::new(429)).await;
    mock_success(&server).await;

    let (elapsed, outcome) = call_through_the_engine(&server).await;
    assert_eq!(outcome.unwrap().status(), 200);
    assert_eq!(attempts(&server).await, 2);
    // A header-less 429 used to default to a 60s wait, which exceeds the 30s
    // cap and so was surfaced instead of retried.
    assert!(
        elapsed < Duration::from_secs(5),
        "the header-less retry waited far too long: {elapsed:?}"
    );
}

#[tokio::test]
async fn a_429_beyond_the_cap_is_surfaced_without_waiting() {
    let server = MockServer::start().await;
    mock_first_response(
        &server,
        ResponseTemplate::new(429).insert_header("retry-after", "60"),
    )
    .await;
    mock_success(&server).await;

    let (elapsed, outcome) = call_through_the_engine(&server).await;
    let message = outcome.unwrap_err().to_string();
    assert!(message.contains("60"), "{message}");
    // Nothing is slept through past the cap, and nothing is retried.
    assert_eq!(attempts(&server).await, 1);
    assert!(
        elapsed < Duration::from_secs(5),
        "a 60s wait was slept instead of reported: {elapsed:?}"
    );
}

#[tokio::test]
async fn a_retry_after_http_date_is_honored() {
    let server = MockServer::start().await;
    // An expired date means "retry now"; the header used to be discarded
    // outright because only delta-seconds were parsed.
    mock_first_response(
        &server,
        ResponseTemplate::new(429).insert_header("retry-after", "Thu, 01 Jan 1970 00:00:00 GMT"),
    )
    .await;
    mock_success(&server).await;

    let (elapsed, outcome) = call_through_the_engine(&server).await;
    assert_eq!(outcome.unwrap().status(), 200);
    assert_eq!(attempts(&server).await, 2);
    assert!(
        elapsed < Duration::from_secs(5),
        "an expired Retry-After date was not retried promptly: {elapsed:?}"
    );
}
