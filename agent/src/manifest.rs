use crate::commitment::Commitments;
use crate::policy::PolicyInfo;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

/// Manifest Schema Version (JSON Schema Draft 2020-12)
pub const MANIFEST_SCHEMA_VERSION: &str = "manifest.v1.0";

/// Audit-Informationen für Manifest
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuditInfo {
    pub tail_digest: String,
    pub events_count: u64,
}

/// Proof-Informationen für Manifest
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProofInfo {
    #[serde(rename = "type")]
    pub proof_type: String,
    pub status: String,
}

/// Signatur-Informationen
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SignatureInfo {
    pub alg: String,
    pub signer: String,
    pub pubkey_hex: String,
    pub sig_hex: String,
}

/// Zeitanker für externe Timestamps
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TimeAnchor {
    pub kind: String,           // "tsa", "blockchain", "file"
    pub reference: String,       // Pfad, TxID oder URI
    pub audit_tip_hex: String,  // Audit-Chain-Tip zum Zeitpunkt des Anchors
    pub created_at: String,     // RFC3339 Timestamp
}

/// Manifest-Datenstruktur
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Manifest {
    pub version: String,
    pub created_at: String,
    pub supplier_root: String,
    pub ubo_root: String,
    pub company_commitment_root: String,
    pub policy: PolicyInfo,
    pub audit: AuditInfo,
    pub proof: ProofInfo,
    pub signatures: Vec<SignatureInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_anchor: Option<TimeAnchor>,
}

/// Signiertes Manifest
#[derive(Debug, Serialize, Deserialize)]
pub struct SignedManifest {
    pub manifest: Manifest,
    pub signature: SignatureInfo,
}

impl Manifest {
    /// Erstellt ein neues Manifest aus Commitments, Policy und Audit-Log
    ///
    /// # Argumente
    /// * `commitments` - Die berechneten Merkle-Roots
    /// * `policy_info` - Policy-Informationen mit Hash
    /// * `audit_log_path` - Pfad zum Audit-Log
    ///
    /// # Rückgabe
    /// Neues Manifest-Objekt
    pub fn build(
        commitments: &Commitments,
        policy_info: PolicyInfo,
        audit_log_path: &str,
    ) -> Result<Self, Box<dyn Error>> {
        // Lese Audit-Log-Tail
        let (tail_digest, events_count) = Self::read_audit_tail(audit_log_path)?;

        Ok(Manifest {
            version: MANIFEST_SCHEMA_VERSION.to_string(),
            created_at: Utc::now().to_rfc3339(),
            supplier_root: commitments.supplier_root.clone(),
            ubo_root: commitments.ubo_root.clone(),
            company_commitment_root: commitments.company_commitment_root.clone(),
            policy: policy_info,
            audit: AuditInfo {
                tail_digest,
                events_count,
            },
            proof: ProofInfo {
                proof_type: "none".to_string(),
                status: "none".to_string(),
            },
            signatures: Vec::new(),
            time_anchor: None,
        })
    }

    /// Liest Tail-Digest und Event-Count aus Audit-Log
    ///
    /// # Argumente
    /// * `path` - Pfad zum Audit-Log
    ///
    /// # Rückgabe
    /// Tuple (tail_digest, events_count)
    fn read_audit_tail(path: &str) -> Result<(String, u64), Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let mut last_digest = "0x0000000000000000000000000000000000000000000000000000000000000000".to_string();
        let mut count: u64 = 0;

        for line in reader.lines() {
            let line = line?;
            if !line.trim().is_empty() {
                let entry: serde_json::Value = serde_json::from_str(&line)?;
                if let Some(digest) = entry.get("digest").and_then(|v| v.as_str()) {
                    last_digest = digest.to_string();
                }
                count += 1;
            }
        }

        Ok((last_digest, count))
    }

    /// Speichert Manifest als JSON
    ///
    /// # Argumente
    /// * `path` - Zielpfad für JSON-Datei
    ///
    /// # Rückgabe
    /// Result
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn Error>> {
        let json = serde_json::to_string_pretty(self)?;
        let mut file = File::create(path)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }

    /// Lädt Manifest aus JSON-Datei
    ///
    /// # Argumente
    /// * `path` - Pfad zur JSON-Datei
    ///
    /// # Rückgabe
    /// Manifest-Objekt
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let file = File::open(path)?;
        let manifest: Manifest = serde_json::from_reader(file)?;
        Ok(manifest)
    }

    /// Aktualisiert Proof-Informationen im Manifest
    ///
    /// # Argumente
    /// * `proof_type` - Typ des Proofs ("mock", "zkp", etc.)
    /// * `status` - Status ("ok", "pending", etc.)
    #[allow(dead_code)]
    pub fn update_proof(&mut self, proof_type: String, status: String) {
        self.proof = ProofInfo { proof_type, status };
    }

    /// Setzt den Zeitanker im Manifest
    ///
    /// # Argumente
    /// * `kind` - Art des Zeitankers ("tsa", "blockchain", "file")
    /// * `reference` - Referenz (Pfad, TxID, URI)
    /// * `audit_tip_hex` - Audit-Chain-Tip (Hex-String)
    pub fn set_time_anchor(&mut self, kind: String, reference: String, audit_tip_hex: String) {
        self.time_anchor = Some(TimeAnchor {
            kind,
            reference,
            audit_tip_hex,
            created_at: Utc::now().to_rfc3339(),
        });
    }

    /// Serialisiert Manifest zu kanonischem JSON (für Signierung)
    ///
    /// # Rückgabe
    /// JSON-String
    pub fn to_canonical_json(&self) -> Result<String, Box<dyn Error>> {
        Ok(serde_json::to_string(self)?)
    }
}

impl SignedManifest {
    /// Speichert signiertes Manifest als JSON
    ///
    /// # Argumente
    /// * `path` - Zielpfad für JSON-Datei
    ///
    /// # Rückgabe
    /// Result
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn Error>> {
        let json = serde_json::to_string_pretty(self)?;
        let mut file = File::create(path)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }

    /// Lädt signiertes Manifest aus JSON-Datei
    ///
    /// # Argumente
    /// * `path` - Pfad zur JSON-Datei
    ///
    /// # Rückgabe
    /// SignedManifest-Objekt
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let file = File::open(path)?;
        let signed: SignedManifest = serde_json::from_reader(file)?;
        Ok(signed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commitment::Commitments;
    use crate::policy::PolicyInfo;

    #[test]
    fn test_manifest_creation() {
        let commitments = Commitments {
            supplier_root: "0xabc".to_string(),
            ubo_root: "0xdef".to_string(),
            company_commitment_root: "0x123".to_string(),
            supplier_count: Some(5),
            ubo_count: Some(2),
        };

        let policy_info = PolicyInfo {
            name: "Test".to_string(),
            version: "lksg.v1".to_string(),
            hash: "0xpolicy".to_string(),
        };

        // Erstelle temporäres Audit-Log
        let temp_audit = "/tmp/test_manifest_audit.jsonl";
        std::fs::write(temp_audit, r#"{"digest":"0xtest","seq":1}"#).unwrap();

        let manifest = Manifest::build(&commitments, policy_info, temp_audit).unwrap();

        assert_eq!(manifest.version, "manifest.v1.0");
        assert_eq!(manifest.supplier_root, "0xabc");
        assert_eq!(manifest.proof.proof_type, "none");

        std::fs::remove_file(temp_audit).ok();
    }

    #[test]
    fn test_manifest_update_proof() {
        let mut manifest = Manifest {
            version: "manifest.v1.0".to_string(),
            created_at: "2025-10-25T10:00:00Z".to_string(),
            supplier_root: "0xabc".to_string(),
            ubo_root: "0xdef".to_string(),
            company_commitment_root: "0x123".to_string(),
            policy: PolicyInfo {
                name: "Test".to_string(),
                version: "lksg.v1".to_string(),
                hash: "0xhash".to_string(),
            },
            audit: AuditInfo {
                tail_digest: "0xtail".to_string(),
                events_count: 5,
            },
            proof: ProofInfo {
                proof_type: "none".to_string(),
                status: "none".to_string(),
            },
            signatures: Vec::new(),
            time_anchor: None,
        };

        manifest.update_proof("mock".to_string(), "ok".to_string());
        assert_eq!(manifest.proof.proof_type, "mock");
        assert_eq!(manifest.proof.status, "ok");
    }

    #[test]
    fn time_anchor_roundtrip_ok() {
        use std::fs;

        let temp_path = "/tmp/test_manifest_anchor.json";

        let mut manifest = Manifest {
            version: "manifest.v1.0".to_string(),
            created_at: "2025-10-29T10:00:00Z".to_string(),
            supplier_root: "0xabc".to_string(),
            ubo_root: "0xdef".to_string(),
            company_commitment_root: "0x123".to_string(),
            policy: PolicyInfo {
                name: "Test".to_string(),
                version: "lksg.v1".to_string(),
                hash: "0xhash".to_string(),
            },
            audit: AuditInfo {
                tail_digest: "0xtail".to_string(),
                events_count: 5,
            },
            proof: ProofInfo {
                proof_type: "none".to_string(),
                status: "none".to_string(),
            },
            signatures: Vec::new(),
            time_anchor: None,
        };

        // Setze Zeitanker
        manifest.set_time_anchor(
            "tsa".to_string(),
            "./tsa/test.tsr".to_string(),
            "0x83a8779d".to_string(),
        );

        // Speichere und lade
        manifest.save(temp_path).unwrap();
        let loaded = Manifest::load(temp_path).unwrap();

        // Prüfe Zeitanker
        assert!(loaded.time_anchor.is_some());
        let anchor = loaded.time_anchor.unwrap();
        assert_eq!(anchor.kind, "tsa");
        assert_eq!(anchor.reference, "./tsa/test.tsr");
        assert_eq!(anchor.audit_tip_hex, "0x83a8779d");
        assert!(!anchor.created_at.is_empty());

        fs::remove_file(temp_path).ok();
    }
}
