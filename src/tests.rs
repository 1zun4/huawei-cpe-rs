use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;

use crate::client::{non_empty, parse_signal_value};
use crate::enums::{LteBandMask, NetworkBandMask, NetworkMode};
use crate::normalize_router_url;
use crate::session::HuaweiSession;

#[test]
fn test_normalize_url_trailing_slash() {
    let url = normalize_router_url("http://192.168.8.1").unwrap();
    assert_eq!(url, "http://192.168.8.1/");
}

#[test]
fn test_normalize_url_already_trailing_slash() {
    let url = normalize_router_url("http://192.168.8.1/").unwrap();
    assert_eq!(url, "http://192.168.8.1/");
}

#[test]
fn test_normalize_url_rejects_query() {
    let result = normalize_router_url("http://192.168.8.1/?foo=bar");
    assert!(result.is_err());
}

#[test]
fn test_normalize_url_rejects_fragment() {
    let result = normalize_router_url("http://192.168.8.1/#frag");
    assert!(result.is_err());
}

#[test]
fn test_normalize_url_rejects_invalid() {
    let result = normalize_router_url("not-a-url");
    assert!(result.is_err());
}

#[test]
fn test_encode_password_sha256() {
    let session = HuaweiSession::new("http://192.168.8.1").unwrap();
    // With no tokens, the token part is empty string
    let encoded = session.encode_password("admin", "test123", 4);
    // Should be base64-encoded SHA256 hex
    assert!(!encoded.is_empty());
    // Should be valid base64
    assert!(BASE64.decode(&encoded).is_ok());
}

#[test]
fn test_encode_password_base64() {
    let session = HuaweiSession::new("http://192.168.8.1").unwrap();
    let encoded = session.encode_password("admin", "test123", 0);
    let decoded = BASE64.decode(&encoded).unwrap();
    assert_eq!(String::from_utf8(decoded).unwrap(), "test123");
}

#[test]
fn test_non_empty_some_value() {
    let s = Some("hello".to_string());
    assert_eq!(non_empty(&s), Some("hello"));
}

#[test]
fn test_non_empty_empty_string() {
    let s = Some("".to_string());
    assert_eq!(non_empty(&s), None);
}

#[test]
fn test_non_empty_none() {
    let s: Option<String> = None;
    assert_eq!(non_empty(&s), None);
}

#[test]
fn test_parse_signal_value_dbm() {
    assert_eq!(parse_signal_value(Some("-75dBm")), Some(-75));
}

#[test]
fn test_parse_signal_value_db() {
    assert_eq!(parse_signal_value(Some("-12dB")), Some(-12));
}

#[test]
fn test_parse_signal_value_plain() {
    assert_eq!(parse_signal_value(Some("-95")), Some(-95));
}

#[test]
fn test_parse_signal_value_empty() {
    assert_eq!(parse_signal_value(Some("")), None);
    assert_eq!(parse_signal_value(None), None);
}

#[test]
fn test_network_mode_api_str() {
    assert_eq!(NetworkMode::Auto.as_api_str(), "00");
    assert_eq!(NetworkMode::Mode2G.as_api_str(), "01");
    assert_eq!(NetworkMode::Mode3G.as_api_str(), "02");
    assert_eq!(NetworkMode::Mode4G.as_api_str(), "03");
    assert_eq!(NetworkMode::Mode3G2GAuto.as_api_str(), "0201");
    assert_eq!(NetworkMode::Mode4G3GAuto.as_api_str(), "0302");
    assert_eq!(NetworkMode::Mode4G2GAuto.as_api_str(), "0301");
}

#[test]
fn test_huawei_client_new() {
    let client = crate::client::HuaweiClient::new("http://192.168.8.1");
    assert!(client.is_ok());
}

#[test]
fn test_huawei_client_new_invalid_url() {
    let client = crate::client::HuaweiClient::new("not-a-url");
    assert!(client.is_err());
}

#[test]
fn test_network_band_mask_or_and_hex() {
    let mask = NetworkBandMask::UMTS_B1_2100 | NetworkBandMask::UMTS_B5_850;
    assert_eq!(mask.bits(), 0x4400000);
    assert_eq!(mask.as_api_hex(), "4400000");
}

#[test]
fn test_lte_band_mask_or_and_hex() {
    let mask = LteBandMask::B3 | LteBandMask::B7 | LteBandMask::B20;
    assert_eq!(mask.bits(), 0x80044);
    assert_eq!(mask.as_api_hex(), "80044");
}
