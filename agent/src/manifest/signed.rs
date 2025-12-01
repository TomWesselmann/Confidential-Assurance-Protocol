//! Signed Manifest - Wrapper with signature
//!
//! Provides SignedManifest for manifests with attached signatures.

use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use super::types::{Manifest, SignatureInfo};

/// Signiertes Manifest
#[derive(Debug, Serialize, Deserialize)]
pub struct SignedManifest {
    pub manifest: Manifest,
    pub signature: SignatureInfo,
}

impl SignedManifest {
    /// Speichert signiertes Manifest als JSON
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn Error>> {
        let json = serde_json::to_string_pretty(self)?;
        let mut file = File::create(path)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }

    /// Lädt signiertes Manifest aus JSON-Datei
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let file = File::open(path)?;
        let signed: SignedManifest = serde_json::from_reader(file)?;
        Ok(signed)
    }
}
