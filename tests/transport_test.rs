//! Transport selection unit tests for the shared HTTP core.

use grr_cli::core::http::{TransportInfo, apply_transport_version};

#[test]
fn default_transport_info_requests_http3() {
    let info = TransportInfo::default();
    assert!(info.http3_requested);
    assert!(!info.http3_effective);
}

#[test]
fn apply_transport_version_pins_h3_when_effective() {
    let builder = reqwest::Client::new().get("https://example.com/users/me/profile");
    let request = apply_transport_version(builder, true).build().unwrap();
    assert_eq!(request.version(), reqwest::Version::HTTP_3);
}

#[test]
fn apply_transport_version_is_noop_without_h3() {
    let builder = reqwest::Client::new().get("https://example.com/users/me/profile");
    let request = apply_transport_version(builder, false).build().unwrap();
    assert_eq!(request.version(), reqwest::Version::HTTP_11);
}
