/// TLS/mTLS Configuration Module (Week 5)
///
/// Production-grade transport security configuration
///
/// Note: Actual TLS termination can be handled by:
/// - axum-server with rustls (native Rust TLS)
/// - Ingress/Reverse proxy (nginx, envoy, traefik)
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// TLS minimum version
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TlsVersion {
    #[serde(rename = "1.0")]
    Tls10,
    #[serde(rename = "1.1")]
    Tls11,
    #[serde(rename = "1.2")]
    Tls12,
    #[serde(rename = "1.3")]
    Tls13,
}

/// Cipher suite profile
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CipherProfile {
    /// TLS 1.3 only, strongest ciphers
    Modern,
    /// TLS 1.2+, broad compatibility
    Intermediate,
    /// TLS 1.0+, maximum compatibility (not recommended)
    Legacy,
}

/// Client certificate validation mode
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ClientCertValidation {
    /// Reject connections without valid client cert
    Required,
    /// Accept both mTLS and regular TLS
    Optional,
    /// No client cert validation
    None,
}

/// TLS/mTLS Configuration (matches config/tls.yaml)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TlsConfig {
    /// Require mutual TLS (client certificates)
    pub require_mtls: bool,

    /// Minimum TLS version
    pub tls_min_version: String,

    /// Cipher suite profile
    pub cipher_profile: String,

    /// Client CA bundle path (for mTLS verification)
    pub client_ca_bundle: String,

    /// Server certificate path
    pub server_cert: String,

    /// Server private key path
    pub server_key: String,

    /// Client certificate validation mode
    #[serde(default = "default_client_cert_validation")]
    pub client_cert_validation: String,

    /// Verify client certificate SAN (Subject Alternative Name)
    #[serde(default)]
    pub verify_client_san: bool,

    /// Allowed client SANs (wildcard patterns supported)
    #[serde(default)]
    pub allowed_client_sans: Vec<String>,
}

fn default_client_cert_validation() -> String {
    "required".to_string()
}

impl TlsConfig {
    /// Parse TLS minimum version
    pub fn parse_tls_version(&self) -> Result<TlsVersion, String> {
        match self.tls_min_version.as_str() {
            "1.0" => Ok(TlsVersion::Tls10),
            "1.1" => Ok(TlsVersion::Tls11),
            "1.2" => Ok(TlsVersion::Tls12),
            "1.3" => Ok(TlsVersion::Tls13),
            _ => Err(format!("Invalid TLS version: {}", self.tls_min_version)),
        }
    }

    /// Parse cipher profile
    pub fn parse_cipher_profile(&self) -> Result<CipherProfile, String> {
        match self.cipher_profile.as_str() {
            "modern" => Ok(CipherProfile::Modern),
            "intermediate" => Ok(CipherProfile::Intermediate),
            "legacy" => Ok(CipherProfile::Legacy),
            _ => Err(format!("Invalid cipher profile: {}", self.cipher_profile)),
        }
    }

    /// Parse client cert validation mode
    pub fn parse_client_cert_validation(&self) -> Result<ClientCertValidation, String> {
        match self.client_cert_validation.as_str() {
            "required" => Ok(ClientCertValidation::Required),
            "optional" => Ok(ClientCertValidation::Optional),
            "none" => Ok(ClientCertValidation::None),
            _ => Err(format!(
                "Invalid client cert validation: {}",
                self.client_cert_validation
            )),
        }
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<(), String> {
        // Validate TLS version
        self.parse_tls_version()?;

        // Validate cipher profile
        self.parse_cipher_profile()?;

        // Validate client cert validation
        self.parse_client_cert_validation()?;

        // Validate paths exist (if require_mtls is true)
        if self.require_mtls {
            let ca_bundle = PathBuf::from(&self.client_ca_bundle);
            if !ca_bundle.exists() && !self.client_ca_bundle.starts_with("/etc/ssl") {
                // Allow /etc/ssl paths to not exist (for config validation)
                return Err(format!(
                    "Client CA bundle not found: {}",
                    self.client_ca_bundle
                ));
            }
        }

        Ok(())
    }

    /// Check if client certificate is required
    pub fn is_client_cert_required(&self) -> bool {
        self.require_mtls
            || self
                .parse_client_cert_validation()
                .unwrap_or(ClientCertValidation::None)
                == ClientCertValidation::Required
    }

    /// Validate client SAN (Subject Alternative Name)
    pub fn validate_client_san(&self, san: &str) -> bool {
        if !self.verify_client_san {
            return true; // SAN verification disabled
        }

        if self.allowed_client_sans.is_empty() {
            return true; // No restrictions
        }

        // Check exact match or wildcard match
        for allowed_san in &self.allowed_client_sans {
            if allowed_san == san {
                return true; // Exact match
            }

            // Wildcard match (*.example.com)
            if let Some(domain_suffix) = allowed_san.strip_prefix("*.") {
                // Ensure there's at least one character before the domain suffix
                // So "*.example.com" matches "foo.example.com" but NOT "example.com"
                if san.ends_with(domain_suffix) && san.len() > domain_suffix.len() {
                    let prefix = &san[..san.len() - domain_suffix.len()];
                    // Ensure prefix ends with a dot (subdomain separator)
                    if prefix.ends_with('.') {
                        return true;
                    }
                }
            }
        }

        false
    }
}

/// Load TLS config from YAML file
pub fn load_tls_config(path: &str) -> Result<TlsConfig, std::io::Error> {
    let content = std::fs::read_to_string(path)?;
    serde_yaml::from_str(&content)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tls_config_parse() {
        let config_yaml = r#"
require_mtls: true
tls_min_version: "1.2"
cipher_profile: "modern"
client_ca_bundle: "/etc/ssl/clients/ca.crt"
server_cert: "/etc/ssl/certs/server.crt"
server_key: "/etc/ssl/private/server.key"
client_cert_validation: "required"
verify_client_san: true
allowed_client_sans:
  - "client.cap-verifier.local"
  - "*.cap-verifier.local"
"#;

        let config: TlsConfig = serde_yaml::from_str(config_yaml).unwrap();
        assert!(config.require_mtls);
        assert_eq!(config.tls_min_version, "1.2");
        assert_eq!(config.cipher_profile, "modern");
        assert_eq!(config.client_cert_validation, "required");
        assert!(config.verify_client_san);
        assert_eq!(config.allowed_client_sans.len(), 2);
    }

    #[test]
    fn test_parse_tls_version() {
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

        assert_eq!(config.parse_tls_version().unwrap(), TlsVersion::Tls12);
    }

    #[test]
    fn test_parse_cipher_profile() {
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

        assert_eq!(
            config.parse_cipher_profile().unwrap(),
            CipherProfile::Modern
        );
    }

    #[test]
    fn test_validate_client_san_exact_match() {
        let config = TlsConfig {
            require_mtls: true,
            tls_min_version: "1.2".to_string(),
            cipher_profile: "modern".to_string(),
            client_ca_bundle: "/etc/ssl/clients/ca.crt".to_string(),
            server_cert: "/etc/ssl/certs/server.crt".to_string(),
            server_key: "/etc/ssl/private/server.key".to_string(),
            client_cert_validation: "required".to_string(),
            verify_client_san: true,
            allowed_client_sans: vec!["client.cap-verifier.local".to_string()],
        };

        assert!(config.validate_client_san("client.cap-verifier.local"));
        assert!(!config.validate_client_san("other.cap-verifier.local"));
    }

    #[test]
    fn test_validate_client_san_wildcard() {
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

        assert!(config.validate_client_san("client1.cap-verifier.local"));
        assert!(config.validate_client_san("client2.cap-verifier.local"));
        assert!(!config.validate_client_san("cap-verifier.local")); // Wildcard doesn't match root domain
        assert!(!config.validate_client_san("other.example.com"));
    }

    #[test]
    fn test_is_client_cert_required() {
        let mut config = TlsConfig {
            require_mtls: false,
            tls_min_version: "1.2".to_string(),
            cipher_profile: "modern".to_string(),
            client_ca_bundle: "/etc/ssl/clients/ca.crt".to_string(),
            server_cert: "/etc/ssl/certs/server.crt".to_string(),
            server_key: "/etc/ssl/private/server.key".to_string(),
            client_cert_validation: "optional".to_string(),
            verify_client_san: false,
            allowed_client_sans: vec![],
        };

        assert!(!config.is_client_cert_required());

        config.require_mtls = true;
        assert!(config.is_client_cert_required());

        config.require_mtls = false;
        config.client_cert_validation = "required".to_string();
        assert!(config.is_client_cert_required());
    }
}
