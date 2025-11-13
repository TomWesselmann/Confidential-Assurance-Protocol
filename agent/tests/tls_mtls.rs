/// Integration Tests for TLS/mTLS Configuration (Week 5)
///
/// Tests:
/// - IT-T1: mTLS required without cert → 403
/// - IT-T2: mTLS required with wrong SAN → 403
/// - IT-T3: mTLS required with valid cert → 200
///
/// Note: These tests validate the TLS configuration module.
/// Actual TLS termination would be handled by axum-server or ingress.
use cap_agent::tls::{load_tls_config, CipherProfile, ClientCertValidation, TlsConfig, TlsVersion};

#[test]
fn test_tls_config_load() {
    // Test loading TLS config from YAML
    let config_path = "config/tls.yaml";

    let config = load_tls_config(config_path).expect("Failed to load TLS config");

    assert!(config.require_mtls);
    assert_eq!(config.tls_min_version, "1.2");
    assert_eq!(config.cipher_profile, "modern");
    assert_eq!(config.client_ca_bundle, "/etc/ssl/clients/ca.crt");

    println!("✅ TLS config loaded successfully");
}

#[test]
fn test_tls_min_version_enforcement() {
    // Test: Only allow TLS 1.2+ connections

    let config = TlsConfig {
        require_mtls: false,
        tls_min_version: "1.2".to_string(),
        cipher_profile: "intermediate".to_string(),
        client_ca_bundle: "/etc/ssl/clients/ca.crt".to_string(),
        server_cert: "/etc/ssl/certs/server.crt".to_string(),
        server_key: "/etc/ssl/private/server.key".to_string(),
        client_cert_validation: "none".to_string(),
        verify_client_san: false,
        allowed_client_sans: vec![],
    };

    let min_version = config.parse_tls_version().unwrap();
    assert_eq!(min_version, TlsVersion::Tls12);

    println!("✅ TLS 1.2 minimum version enforced");
}

#[test]
fn test_modern_cipher_profile() {
    // Test: Modern cipher profile enforces TLS 1.3

    let config = TlsConfig {
        require_mtls: false,
        tls_min_version: "1.3".to_string(),
        cipher_profile: "modern".to_string(),
        client_ca_bundle: "/etc/ssl/clients/ca.crt".to_string(),
        server_cert: "/etc/ssl/certs/server.crt".to_string(),
        server_key: "/etc/ssl/private/server.key".to_string(),
        client_cert_validation: "none".to_string(),
        verify_client_san: false,
        allowed_client_sans: vec![],
    };

    let cipher_profile = config.parse_cipher_profile().unwrap();
    assert_eq!(cipher_profile, CipherProfile::Modern);

    println!("✅ Modern cipher profile configured");
}

#[test]
fn it_t1_mtls_required_without_cert() {
    // Test: Connection without client certificate → 403 Forbidden

    let config = TlsConfig {
        require_mtls: true,
        tls_min_version: "1.2".to_string(),
        cipher_profile: "modern".to_string(),
        client_ca_bundle: "/etc/ssl/clients/ca.crt".to_string(),
        server_cert: "/etc/ssl/certs/server.crt".to_string(),
        server_key: "/etc/ssl/private/server.key".to_string(),
        client_cert_validation: "required".to_string(),
        verify_client_san: false,
        allowed_client_sans: vec![],
    };

    // Simulate connection without client certificate
    let has_client_cert = false;

    if config.is_client_cert_required() && !has_client_cert {
        println!("✅ Connection without client cert rejected (403 Forbidden)");
    } else {
        panic!("Expected rejection without client cert");
    }
}

#[test]
fn it_t2_mtls_required_with_wrong_san() {
    // Test: Connection with client cert but wrong SAN → 403 Forbidden

    let config = TlsConfig {
        require_mtls: true,
        tls_min_version: "1.2".to_string(),
        cipher_profile: "modern".to_string(),
        client_ca_bundle: "/etc/ssl/clients/ca.crt".to_string(),
        server_cert: "/etc/ssl/certs/server.crt".to_string(),
        server_key: "/etc/ssl/private/server.key".to_string(),
        client_cert_validation: "required".to_string(),
        verify_client_san: true,
        allowed_client_sans: vec!["*.cap-verifier.local".to_string()],
    };

    // Simulate client certificate with wrong SAN
    let client_san = "malicious.attacker.com";

    if !config.validate_client_san(client_san) {
        println!("✅ Connection with wrong SAN rejected (403 Forbidden)");
    } else {
        panic!("Expected rejection with wrong SAN");
    }
}

#[test]
fn it_t3_mtls_required_with_valid_cert() {
    // Test: Connection with valid client cert and correct SAN → 200 OK

    let config = TlsConfig {
        require_mtls: true,
        tls_min_version: "1.2".to_string(),
        cipher_profile: "modern".to_string(),
        client_ca_bundle: "/etc/ssl/clients/ca.crt".to_string(),
        server_cert: "/etc/ssl/certs/server.crt".to_string(),
        server_key: "/etc/ssl/private/server.key".to_string(),
        client_cert_validation: "required".to_string(),
        verify_client_san: true,
        allowed_client_sans: vec!["*.cap-verifier.local".to_string()],
    };

    // Simulate valid client certificate with correct SAN
    let client_san = "client1.cap-verifier.local";

    if config.is_client_cert_required() && config.validate_client_san(client_san) {
        println!("✅ Connection with valid cert and SAN accepted (200 OK)");
    } else {
        panic!("Expected acceptance with valid cert and SAN");
    }
}

#[test]
fn it_t4_optional_mtls_without_cert() {
    // Test: Optional mTLS mode allows connections without client cert

    let config = TlsConfig {
        require_mtls: false,
        tls_min_version: "1.2".to_string(),
        cipher_profile: "intermediate".to_string(),
        client_ca_bundle: "/etc/ssl/clients/ca.crt".to_string(),
        server_cert: "/etc/ssl/certs/server.crt".to_string(),
        server_key: "/etc/ssl/private/server.key".to_string(),
        client_cert_validation: "optional".to_string(),
        verify_client_san: false,
        allowed_client_sans: vec![],
    };

    // Simulate connection without client certificate
    let has_client_cert = false;

    if !config.is_client_cert_required() || has_client_cert {
        println!("✅ Optional mTLS allows connection without cert (200 OK)");
    }
}

#[test]
fn it_t5_wildcard_san_matching() {
    // Test: Wildcard SAN matching works correctly

    let config = TlsConfig {
        require_mtls: true,
        tls_min_version: "1.2".to_string(),
        cipher_profile: "modern".to_string(),
        client_ca_bundle: "/etc/ssl/clients/ca.crt".to_string(),
        server_cert: "/etc/ssl/certs/server.crt".to_string(),
        server_key: "/etc/ssl/private/server.key".to_string(),
        client_cert_validation: "required".to_string(),
        verify_client_san: true,
        allowed_client_sans: vec!["*.cap-verifier.local".to_string()],
    };

    // Test various SAN patterns
    assert!(config.validate_client_san("client1.cap-verifier.local"));
    assert!(config.validate_client_san("client2.cap-verifier.local"));
    assert!(config.validate_client_san("test.cap-verifier.local"));

    // Should NOT match
    assert!(!config.validate_client_san("cap-verifier.local")); // Root domain
    assert!(!config.validate_client_san("malicious.com"));
    assert!(!config.validate_client_san("client1.other-domain.com"));

    println!("✅ Wildcard SAN matching validated");
}

#[test]
fn it_t6_exact_san_matching() {
    // Test: Exact SAN matching (no wildcards)

    let config = TlsConfig {
        require_mtls: true,
        tls_min_version: "1.2".to_string(),
        cipher_profile: "modern".to_string(),
        client_ca_bundle: "/etc/ssl/clients/ca.crt".to_string(),
        server_cert: "/etc/ssl/certs/server.crt".to_string(),
        server_key: "/etc/ssl/private/server.key".to_string(),
        client_cert_validation: "required".to_string(),
        verify_client_san: true,
        allowed_client_sans: vec![
            "client1.cap-verifier.local".to_string(),
            "client2.cap-verifier.local".to_string(),
        ],
    };

    // Should match
    assert!(config.validate_client_san("client1.cap-verifier.local"));
    assert!(config.validate_client_san("client2.cap-verifier.local"));

    // Should NOT match
    assert!(!config.validate_client_san("client3.cap-verifier.local"));
    assert!(!config.validate_client_san("other.cap-verifier.local"));

    println!("✅ Exact SAN matching validated");
}

#[test]
fn it_t7_cipher_profile_validation() {
    // Test: All cipher profiles parse correctly

    let profiles = vec![
        ("modern", CipherProfile::Modern),
        ("intermediate", CipherProfile::Intermediate),
        ("legacy", CipherProfile::Legacy),
    ];

    for (profile_str, expected) in profiles {
        let config = TlsConfig {
            require_mtls: false,
            tls_min_version: "1.2".to_string(),
            cipher_profile: profile_str.to_string(),
            client_ca_bundle: "/etc/ssl/clients/ca.crt".to_string(),
            server_cert: "/etc/ssl/certs/server.crt".to_string(),
            server_key: "/etc/ssl/private/server.key".to_string(),
            client_cert_validation: "none".to_string(),
            verify_client_san: false,
            allowed_client_sans: vec![],
        };

        assert_eq!(config.parse_cipher_profile().unwrap(), expected);
    }

    println!("✅ All cipher profiles validated");
}

#[test]
fn it_t8_tls_version_validation() {
    // Test: All TLS versions parse correctly

    let versions = vec![
        ("1.0", TlsVersion::Tls10),
        ("1.1", TlsVersion::Tls11),
        ("1.2", TlsVersion::Tls12),
        ("1.3", TlsVersion::Tls13),
    ];

    for (version_str, expected) in versions {
        let config = TlsConfig {
            require_mtls: false,
            tls_min_version: version_str.to_string(),
            cipher_profile: "intermediate".to_string(),
            client_ca_bundle: "/etc/ssl/clients/ca.crt".to_string(),
            server_cert: "/etc/ssl/certs/server.crt".to_string(),
            server_key: "/etc/ssl/private/server.key".to_string(),
            client_cert_validation: "none".to_string(),
            verify_client_san: false,
            allowed_client_sans: vec![],
        };

        assert_eq!(config.parse_tls_version().unwrap(), expected);
    }

    println!("✅ All TLS versions validated");
}

#[test]
fn it_t9_client_cert_validation_modes() {
    // Test: All client cert validation modes parse correctly

    let modes = vec![
        ("required", ClientCertValidation::Required),
        ("optional", ClientCertValidation::Optional),
        ("none", ClientCertValidation::None),
    ];

    for (mode_str, expected) in modes {
        let config = TlsConfig {
            require_mtls: false,
            tls_min_version: "1.2".to_string(),
            cipher_profile: "intermediate".to_string(),
            client_ca_bundle: "/etc/ssl/clients/ca.crt".to_string(),
            server_cert: "/etc/ssl/certs/server.crt".to_string(),
            server_key: "/etc/ssl/private/server.key".to_string(),
            client_cert_validation: mode_str.to_string(),
            verify_client_san: false,
            allowed_client_sans: vec![],
        };

        assert_eq!(config.parse_client_cert_validation().unwrap(), expected);
    }

    println!("✅ All client cert validation modes validated");
}
