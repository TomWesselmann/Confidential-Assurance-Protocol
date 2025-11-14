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
}
