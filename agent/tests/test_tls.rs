/// Integration Tests für tls.rs
///
/// Diese Tests wurden aus inline test modules extrahiert um Tarpaulin Coverage-Tracking zu ermöglichen.
/// Tarpaulin hat eine bekannte Limitation mit #[cfg(test)] inline modules.
///
/// Note: Tests für private Funktionen (load_certs, load_private_key) bleiben im inline module,
/// da Integration Tests nur die öffentliche API testen sollten.

use cap_agent::api::tls::*;

#[test]
fn test_tls_config_creation() {
    let config = TlsConfig::new(
        "certs/server.crt".to_string(),
        "certs/server.key".to_string(),
    );

    assert_eq!(config.mode, TlsMode::Tls);
    assert!(config.ca_cert_path.is_none());
}

#[test]
fn test_mtls_config_creation() {
    let config = TlsConfig::new(
        "certs/server.crt".to_string(),
        "certs/server.key".to_string(),
    )
    .with_mtls("certs/ca.crt".to_string());

    assert_eq!(config.mode, TlsMode::Mtls);
    assert!(config.ca_cert_path.is_some());
    assert_eq!(config.ca_cert_path.unwrap(), "certs/ca.crt");
}

#[test]
fn test_tls_error_no_certs_display() {
    let err = TlsError::NoCerts;
    let display = format!("{}", err);
    assert!(display.contains("No certificates found"));
}

#[test]
fn test_tls_error_no_keys_display() {
    let err = TlsError::NoKeys;
    let display = format!("{}", err);
    assert!(display.contains("No private keys found"));
}

#[test]
fn test_tls_error_invalid_cert_display() {
    let err = TlsError::InvalidCert("bad format".to_string());
    let display = format!("{}", err);
    assert!(display.contains("Invalid certificate"));
    assert!(display.contains("bad format"));
}

#[test]
fn test_tls_error_invalid_key_display() {
    let err = TlsError::InvalidKey("bad format".to_string());
    let display = format!("{}", err);
    assert!(display.contains("Invalid private key"));
    assert!(display.contains("bad format"));
}

#[test]
fn test_tls_mode_equality() {
    assert_eq!(TlsMode::Disabled, TlsMode::Disabled);
    assert_eq!(TlsMode::Tls, TlsMode::Tls);
    assert_eq!(TlsMode::Mtls, TlsMode::Mtls);

    assert_ne!(TlsMode::Disabled, TlsMode::Tls);
    assert_ne!(TlsMode::Tls, TlsMode::Mtls);
}

#[test]
fn test_validate_tls_files_missing_cert() {
    let config = TlsConfig {
        mode: TlsMode::Tls,
        cert_path: "/nonexistent/cert.pem".to_string(),
        key_path: "/nonexistent/key.pem".to_string(),
        ca_cert_path: None,
    };

    let result = validate_tls_files(&config);

    // Should return error about missing cert file
    assert!(result.is_err());
    let err_msg = format!("{:?}", result.unwrap_err());
    assert!(err_msg.contains("cert") || err_msg.contains("Certificate"));
}

#[test]
fn test_validate_tls_files_missing_key() {
    use tempfile::NamedTempFile;

    // Create temp cert file
    let temp_cert = NamedTempFile::new().unwrap();

    let config = TlsConfig {
        mode: TlsMode::Tls,
        cert_path: temp_cert.path().to_str().unwrap().to_string(),
        key_path: "/nonexistent/key.pem".to_string(),
        ca_cert_path: None,
    };

    let result = validate_tls_files(&config);

    // Should return error about missing key file
    assert!(result.is_err());
    let err_msg = format!("{:?}", result.unwrap_err());
    assert!(err_msg.contains("key") || err_msg.contains("Private"));
}

#[test]
fn test_validate_tls_files_missing_ca() {
    use tempfile::NamedTempFile;

    // Create temp cert and key files
    let temp_cert = NamedTempFile::new().unwrap();
    let temp_key = NamedTempFile::new().unwrap();

    let config = TlsConfig {
        mode: TlsMode::Mtls,
        cert_path: temp_cert.path().to_str().unwrap().to_string(),
        key_path: temp_key.path().to_str().unwrap().to_string(),
        ca_cert_path: Some("/nonexistent/ca.pem".to_string()),
    };

    let result = validate_tls_files(&config);

    // Should return error about missing CA file
    assert!(result.is_err());
    let err_msg = format!("{:?}", result.unwrap_err());
    assert!(err_msg.contains("CA") || err_msg.contains("ca"));
}

#[test]
fn test_validate_tls_files_success() {
    use tempfile::NamedTempFile;

    // Create temp cert and key files
    let temp_cert = NamedTempFile::new().unwrap();
    let temp_key = NamedTempFile::new().unwrap();

    let config = TlsConfig {
        mode: TlsMode::Tls,
        cert_path: temp_cert.path().to_str().unwrap().to_string(),
        key_path: temp_key.path().to_str().unwrap().to_string(),
        ca_cert_path: None,
    };

    let result = validate_tls_files(&config);

    // Should succeed (files exist, no CA required)
    assert!(result.is_ok());
}

#[test]
fn test_build_server_config_disabled_mode() {
    let config = TlsConfig {
        mode: TlsMode::Disabled,
        cert_path: String::new(),
        key_path: String::new(),
        ca_cert_path: None,
    };

    let result = config.build_server_config();

    // Should return error for disabled mode
    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("disabled") || err_msg.contains("Disabled"));
}
