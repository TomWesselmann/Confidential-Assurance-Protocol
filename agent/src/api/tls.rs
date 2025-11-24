/// TLS/mTLS Configuration Module (Phase 1)
///
/// Provides TLS configuration with rustls for production-ready API deployment.
/// Supports both TLS (server-side only) and mTLS (mutual authentication).
use rustls::{Certificate, PrivateKey, ServerConfig};
use rustls_pemfile::{certs, pkcs8_private_keys};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::sync::Arc;

/// TLS Configuration Error
#[derive(Debug)]
pub enum TlsError {
    IoError(std::io::Error),
    InvalidCert(String),
    InvalidKey(String),
    NoCerts,
    NoKeys,
}

impl std::fmt::Display for TlsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TlsError::IoError(e) => write!(f, "IO error: {}", e),
            TlsError::InvalidCert(msg) => write!(f, "Invalid certificate: {}", msg),
            TlsError::InvalidKey(msg) => write!(f, "Invalid private key: {}", msg),
            TlsError::NoCerts => write!(f, "No certificates found in file"),
            TlsError::NoKeys => write!(f, "No private keys found in file"),
        }
    }
}

impl std::error::Error for TlsError {}

impl From<std::io::Error> for TlsError {
    fn from(err: std::io::Error) -> Self {
        TlsError::IoError(err)
    }
}

/// TLS Mode Configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TlsMode {
    /// No TLS (development only)
    Disabled,
    /// TLS only (server certificate validation)
    Tls,
    /// Mutual TLS (client + server certificate validation)
    Mtls,
}

/// TLS Configuration
#[derive(Debug, Clone)]
pub struct TlsConfig {
    /// TLS mode
    pub mode: TlsMode,
    /// Path to server certificate (PEM)
    pub cert_path: String,
    /// Path to server private key (PEM, PKCS#8)
    pub key_path: String,
    /// Path to CA certificate for mTLS client verification (optional)
    pub ca_cert_path: Option<String>,
}

impl TlsConfig {
    /// Create new TLS configuration
    pub fn new(cert_path: String, key_path: String) -> Self {
        Self {
            mode: TlsMode::Tls,
            cert_path,
            key_path,
            ca_cert_path: None,
        }
    }

    /// Enable mTLS with CA certificate for client verification
    pub fn with_mtls(mut self, ca_cert_path: String) -> Self {
        self.mode = TlsMode::Mtls;
        self.ca_cert_path = Some(ca_cert_path);
        self
    }

    /// Build rustls ServerConfig from this configuration
    pub fn build_server_config(&self) -> Result<Arc<ServerConfig>, TlsError> {
        match self.mode {
            TlsMode::Disabled => Err(TlsError::InvalidCert("TLS is disabled".to_string())),
            TlsMode::Tls => self.build_tls_config(),
            TlsMode::Mtls => self.build_mtls_config(),
        }
    }

    /// Build TLS-only configuration (server cert only)
    fn build_tls_config(&self) -> Result<Arc<ServerConfig>, TlsError> {
        // Load certificates
        let certs = load_certs(&self.cert_path)?;

        // Load private key
        let key = load_private_key(&self.key_path)?;

        // Build ServerConfig
        let config = ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_single_cert(certs, key)
            .map_err(|e| TlsError::InvalidCert(e.to_string()))?;

        Ok(Arc::new(config))
    }

    /// Build mTLS configuration (mutual authentication)
    fn build_mtls_config(&self) -> Result<Arc<ServerConfig>, TlsError> {
        let ca_cert_path = self
            .ca_cert_path
            .as_ref()
            .ok_or_else(|| TlsError::InvalidCert("CA cert required for mTLS".to_string()))?;

        // Load server certificates
        let certs = load_certs(&self.cert_path)?;

        // Load server private key
        let key = load_private_key(&self.key_path)?;

        // Load CA certificate for client verification
        let ca_certs = load_certs(ca_cert_path)?;

        // Build RootCertStore from CA certificates
        let mut root_store = rustls::RootCertStore::empty();
        for cert in ca_certs {
            root_store
                .add(&cert)
                .map_err(|e| TlsError::InvalidCert(format!("Failed to add CA cert: {}", e)))?;
        }

        // Build client certificate verifier (rustls 0.21 API)
        let client_cert_verifier = rustls::server::AllowAnyAuthenticatedClient::new(root_store);

        // Build ServerConfig with client auth
        let config = ServerConfig::builder()
            .with_safe_defaults()
            .with_client_cert_verifier(Arc::new(client_cert_verifier))
            .with_single_cert(certs, key)
            .map_err(|e| TlsError::InvalidCert(e.to_string()))?;

        Ok(Arc::new(config))
    }
}

/// Load certificates from PEM file
fn load_certs(path: &str) -> Result<Vec<Certificate>, TlsError> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let certs: Vec<_> = certs(&mut reader)
        .map_err(|e| TlsError::InvalidCert(format!("Failed to parse certs: {}", e)))?
        .into_iter()
        .map(Certificate)
        .collect();

    if certs.is_empty() {
        return Err(TlsError::NoCerts);
    }

    Ok(certs)
}

/// Load private key from PEM file (PKCS#8 format)
fn load_private_key(path: &str) -> Result<PrivateKey, TlsError> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let keys: Vec<_> = pkcs8_private_keys(&mut reader)
        .map_err(|e| TlsError::InvalidKey(format!("Failed to parse key: {}", e)))?;

    if keys.is_empty() {
        return Err(TlsError::NoKeys);
    }

    Ok(PrivateKey(keys[0].clone()))
}

/// Helper: Check if TLS files exist
pub fn validate_tls_files(config: &TlsConfig) -> Result<(), TlsError> {
    if !Path::new(&config.cert_path).exists() {
        return Err(TlsError::IoError(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Certificate file not found: {}", config.cert_path),
        )));
    }

    if !Path::new(&config.key_path).exists() {
        return Err(TlsError::IoError(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Private key file not found: {}", config.key_path),
        )));
    }

    if let Some(ref ca_path) = config.ca_cert_path {
        if !Path::new(ca_path).exists() {
            return Err(TlsError::IoError(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("CA certificate file not found: {}", ca_path),
            )));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_load_certs_nonexistent_file() {
        let result = load_certs("/nonexistent/path/cert.pem");

        // Should return IO error
        assert!(result.is_err());
    }

    #[test]
    fn test_load_certs_invalid_pem() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        // Create temp file with invalid PEM data
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "not a valid PEM certificate").unwrap();

        let result = load_certs(temp_file.path().to_str().unwrap());

        // Should return NoCerts error (no valid certs found)
        assert!(result.is_err());
        if let Err(e) = result {
            match e {
                TlsError::NoCerts => {} // Expected
                other => panic!("Expected NoCerts error, got: {:?}", other),
            }
        }
    }

    #[test]
    fn test_load_private_key_nonexistent_file() {
        let result = load_private_key("/nonexistent/path/key.pem");

        // Should return IO error
        assert!(result.is_err());
    }

    #[test]
    fn test_load_private_key_invalid_pem() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        // Create temp file with invalid PEM data
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "not a valid PEM private key").unwrap();

        let result = load_private_key(temp_file.path().to_str().unwrap());

        // Should return NoKeys error (no valid keys found)
        assert!(result.is_err());
        if let Err(e) = result {
            match e {
                TlsError::NoKeys => {} // Expected
                other => panic!("Expected NoKeys error, got: {:?}", other),
            }
        }
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
}
