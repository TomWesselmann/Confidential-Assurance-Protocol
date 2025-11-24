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

/// Private Anchor (lokaler Audit-Tip)
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct TimeAnchorPrivate {
    pub audit_tip_hex: String, // 0x-prefixed SHA3-256 hash
    pub created_at: String,    // RFC3339 Timestamp
}

/// Public Chain Enum für Blockchain-Anker
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PublicChain {
    Ethereum,
    Hedera,
    #[serde(rename = "btc")]
    Btc,
}

/// Public Anchor (öffentlicher Ledger-Verweis)
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct TimeAnchorPublic {
    pub chain: PublicChain,
    pub txid: String,       // Transaction ID (format depends on chain)
    pub digest: String,     // 0x-prefixed hash of audit_tip for on-chain notarization
    pub created_at: String, // RFC3339 Timestamp
}

/// Zeitanker für externe Timestamps (Dual-Anchor Support v0.9.0)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TimeAnchor {
    pub kind: String,          // "tsa", "blockchain", "file", "none"
    pub reference: String,     // Pfad, TxID oder URI
    pub audit_tip_hex: String, // Audit-Chain-Tip zum Zeitpunkt des Anchors
    pub created_at: String,    // RFC3339 Timestamp

    // Dual-Anchor Fields (v0.9.0, optional, additive)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private: Option<TimeAnchorPrivate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public: Option<TimeAnchorPublic>,
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

        let mut last_digest =
            "0x0000000000000000000000000000000000000000000000000000000000000000".to_string();
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
            private: None,
            public: None,
        });
    }

    /// Setzt den Private Anchor (Dual-Anchor v0.9.0)
    ///
    /// # Argumente
    /// * `audit_tip_hex` - Audit-Chain-Tip (0x-prefixed)
    /// * `created_at` - RFC3339 Timestamp (optional, None = jetzt)
    ///
    /// # Errors
    /// Returns error if time_anchor is not initialized or if audit_tip doesn't match
    pub fn set_private_anchor(
        &mut self,
        audit_tip_hex: String,
        created_at: Option<String>,
    ) -> Result<(), Box<dyn Error>> {
        let anchor = self
            .time_anchor
            .as_mut()
            .ok_or("time_anchor must be initialized before setting private anchor")?;

        // Consistency check: private.audit_tip_hex must match time_anchor.audit_tip_hex
        if audit_tip_hex != anchor.audit_tip_hex {
            return Err(format!(
                "Private audit_tip_hex ({}) does not match time_anchor.audit_tip_hex ({})",
                audit_tip_hex, anchor.audit_tip_hex
            )
            .into());
        }

        anchor.private = Some(TimeAnchorPrivate {
            audit_tip_hex,
            created_at: created_at.unwrap_or_else(|| Utc::now().to_rfc3339()),
        });

        Ok(())
    }

    /// Setzt den Public Anchor (Dual-Anchor v0.9.0)
    ///
    /// # Argumente
    /// * `chain` - Blockchain (ethereum, hedera, btc)
    /// * `txid` - Transaction ID
    /// * `digest` - Hash des Audit-Tip (0x-prefixed)
    /// * `created_at` - RFC3339 Timestamp (optional, None = jetzt)
    ///
    /// # Errors
    /// Returns error if time_anchor is not initialized
    pub fn set_public_anchor(
        &mut self,
        chain: PublicChain,
        txid: String,
        digest: String,
        created_at: Option<String>,
    ) -> Result<(), Box<dyn Error>> {
        let anchor = self
            .time_anchor
            .as_mut()
            .ok_or("time_anchor must be initialized before setting public anchor")?;

        anchor.public = Some(TimeAnchorPublic {
            chain,
            txid,
            digest,
            created_at: created_at.unwrap_or_else(|| Utc::now().to_rfc3339()),
        });

        Ok(())
    }

    /// Validiert Dual-Anchor-Konsistenz
    ///
    /// # Returns
    /// Ok(()) wenn konsistent, Err mit Details bei Inkonsistenz
    pub fn validate_dual_anchor(&self) -> Result<(), Box<dyn Error>> {
        let anchor = match &self.time_anchor {
            Some(a) => a,
            None => return Ok(()), // No anchor = OK
        };

        // Check private anchor consistency
        if let Some(private) = &anchor.private {
            if private.audit_tip_hex != anchor.audit_tip_hex {
                return Err(format!(
                    "Private anchor audit_tip_hex mismatch: expected {}, got {}",
                    anchor.audit_tip_hex, private.audit_tip_hex
                )
                .into());
            }
        }

        // Check public anchor format (basic validation)
        if let Some(public) = &anchor.public {
            if !public.digest.starts_with("0x") || public.digest.len() != 66 {
                return Err(
                    format!("Public anchor digest has invalid format: {}", public.digest).into(),
                );
            }
            if public.txid.is_empty() {
                return Err("Public anchor txid cannot be empty".into());
            }
        }

        Ok(())
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

    // ====================================================================================
    // Neue Tests für Coverage-Erweiterung (53% -> 70%+)
    // ====================================================================================

    // --- set_private_anchor() Tests ---

    #[test]
    fn test_set_private_anchor_success() {
        let mut manifest = create_test_manifest();
        let audit_tip = "0x1234567890123456789012345678901234567890123456789012345678901234".to_string();

        manifest.set_time_anchor("tsa".to_string(), "./tsa/test.tsr".to_string(), audit_tip.clone());

        let result = manifest.set_private_anchor(audit_tip.clone(), None);
        assert!(result.is_ok());

        let anchor = manifest.time_anchor.unwrap();
        assert!(anchor.private.is_some());
        let private = anchor.private.unwrap();
        assert_eq!(private.audit_tip_hex, audit_tip);
        assert!(!private.created_at.is_empty());
    }

    #[test]
    fn test_set_private_anchor_not_initialized() {
        let mut manifest = create_test_manifest();
        let audit_tip = "0x1234567890123456789012345678901234567890123456789012345678901234".to_string();

        // time_anchor is None
        let result = manifest.set_private_anchor(audit_tip, None);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("time_anchor must be initialized"));
    }

    #[test]
    fn test_set_private_anchor_mismatch() {
        let mut manifest = create_test_manifest();
        let anchor_tip = "0x1111111111111111111111111111111111111111111111111111111111111111".to_string();
        let private_tip = "0x2222222222222222222222222222222222222222222222222222222222222222".to_string();

        manifest.set_time_anchor("tsa".to_string(), "./tsa/test.tsr".to_string(), anchor_tip);

        let result = manifest.set_private_anchor(private_tip, None);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("does not match"));
    }

    // --- set_public_anchor() Tests ---

    #[test]
    fn test_set_public_anchor_ethereum_success() {
        let mut manifest = create_test_manifest();
        let audit_tip = "0x1234567890123456789012345678901234567890123456789012345678901234".to_string();

        manifest.set_time_anchor("blockchain".to_string(), "ethereum".to_string(), audit_tip);

        let result = manifest.set_public_anchor(
            PublicChain::Ethereum,
            "0xabc123def456".to_string(),
            "0x9999999999999999999999999999999999999999999999999999999999999999".to_string(),
            None,
        );

        assert!(result.is_ok());
        let anchor = manifest.time_anchor.unwrap();
        assert!(anchor.public.is_some());
        let public = anchor.public.unwrap();
        assert_eq!(public.chain, PublicChain::Ethereum);
        assert_eq!(public.txid, "0xabc123def456");
        assert!(!public.created_at.is_empty());
    }

    #[test]
    fn test_set_public_anchor_hedera() {
        let mut manifest = create_test_manifest();
        manifest.set_time_anchor("blockchain".to_string(), "hedera".to_string(), "0x1234567890123456789012345678901234567890123456789012345678901234".to_string());

        let result = manifest.set_public_anchor(
            PublicChain::Hedera,
            "0.0.12345@1234567890.123456789".to_string(),
            "0x8888888888888888888888888888888888888888888888888888888888888888".to_string(),
            None,
        );

        assert!(result.is_ok());
        let public = manifest.time_anchor.unwrap().public.unwrap();
        assert_eq!(public.chain, PublicChain::Hedera);
    }

    #[test]
    fn test_set_public_anchor_btc() {
        let mut manifest = create_test_manifest();
        manifest.set_time_anchor("blockchain".to_string(), "btc".to_string(), "0x1234567890123456789012345678901234567890123456789012345678901234".to_string());

        let result = manifest.set_public_anchor(
            PublicChain::Btc,
            "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa".to_string(),
            "0x7777777777777777777777777777777777777777777777777777777777777777".to_string(),
            None,
        );

        assert!(result.is_ok());
        let public = manifest.time_anchor.unwrap().public.unwrap();
        assert_eq!(public.chain, PublicChain::Btc);
    }

    #[test]
    fn test_set_public_anchor_not_initialized() {
        let mut manifest = create_test_manifest();

        let result = manifest.set_public_anchor(
            PublicChain::Ethereum,
            "0xabc".to_string(),
            "0x1234567890123456789012345678901234567890123456789012345678901234".to_string(),
            None,
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("time_anchor must be initialized"));
    }

    // --- validate_dual_anchor() Tests ---

    #[test]
    fn test_validate_dual_anchor_no_anchor() {
        let manifest = create_test_manifest();
        let result = manifest.validate_dual_anchor();
        assert!(result.is_ok()); // No anchor = OK
    }

    #[test]
    fn test_validate_dual_anchor_private_mismatch() {
        let mut manifest = create_test_manifest();
        manifest.set_time_anchor("tsa".to_string(), "./tsa/test.tsr".to_string(), "0x1111111111111111111111111111111111111111111111111111111111111111".to_string());

        // Manually set inconsistent private anchor
        if let Some(anchor) = &mut manifest.time_anchor {
            anchor.private = Some(TimeAnchorPrivate {
                audit_tip_hex: "0x2222222222222222222222222222222222222222222222222222222222222222".to_string(),
                created_at: Utc::now().to_rfc3339(),
            });
        }

        let result = manifest.validate_dual_anchor();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("mismatch"));
    }

    #[test]
    fn test_validate_dual_anchor_invalid_public_digest() {
        let mut manifest = create_test_manifest();
        manifest.set_time_anchor("blockchain".to_string(), "eth".to_string(), "0x1234567890123456789012345678901234567890123456789012345678901234".to_string());

        // Set public anchor with invalid digest (wrong format)
        if let Some(anchor) = &mut manifest.time_anchor {
            anchor.public = Some(TimeAnchorPublic {
                chain: PublicChain::Ethereum,
                txid: "0xabc".to_string(),
                digest: "invalid".to_string(), // Invalid format
                created_at: Utc::now().to_rfc3339(),
            });
        }

        let result = manifest.validate_dual_anchor();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("invalid format"));
    }

    #[test]
    fn test_validate_dual_anchor_empty_txid() {
        let mut manifest = create_test_manifest();
        manifest.set_time_anchor("blockchain".to_string(), "eth".to_string(), "0x1234567890123456789012345678901234567890123456789012345678901234".to_string());

        // Set public anchor with empty txid
        if let Some(anchor) = &mut manifest.time_anchor {
            anchor.public = Some(TimeAnchorPublic {
                chain: PublicChain::Ethereum,
                txid: "".to_string(), // Empty
                digest: "0x1234567890123456789012345678901234567890123456789012345678901234".to_string(),
                created_at: Utc::now().to_rfc3339(),
            });
        }

        let result = manifest.validate_dual_anchor();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("txid cannot be empty"));
    }

    #[test]
    fn test_validate_dual_anchor_success() {
        let mut manifest = create_test_manifest();
        let audit_tip = "0x1234567890123456789012345678901234567890123456789012345678901234".to_string();

        manifest.set_time_anchor("blockchain".to_string(), "eth".to_string(), audit_tip.clone());
        manifest.set_private_anchor(audit_tip.clone(), None).unwrap();
        manifest.set_public_anchor(
            PublicChain::Ethereum,
            "0xabc123".to_string(),
            "0x5555555555555555555555555555555555555555555555555555555555555555".to_string(),
            None,
        ).unwrap();

        let result = manifest.validate_dual_anchor();
        assert!(result.is_ok());
    }

    // --- read_audit_tail() Edge Cases ---

    #[test]
    fn test_read_audit_tail_empty_file() {
        use std::fs;
        let temp_audit = "/tmp/test_empty_audit.jsonl";
        fs::write(temp_audit, "").unwrap();

        let result = Manifest::read_audit_tail(temp_audit);
        assert!(result.is_ok());
        let (digest, count) = result.unwrap();
        assert_eq!(digest, "0x0000000000000000000000000000000000000000000000000000000000000000");
        assert_eq!(count, 0);

        fs::remove_file(temp_audit).ok();
    }

    #[test]
    fn test_read_audit_tail_multiple_entries() {
        use std::fs;
        let temp_audit = "/tmp/test_multiple_audit.jsonl";
        fs::write(temp_audit,
            r#"{"digest":"0x1111111111111111111111111111111111111111111111111111111111111111","seq":1}
{"digest":"0x2222222222222222222222222222222222222222222222222222222222222222","seq":2}
{"digest":"0x3333333333333333333333333333333333333333333333333333333333333333","seq":3}"#
        ).unwrap();

        let result = Manifest::read_audit_tail(temp_audit);
        assert!(result.is_ok());
        let (digest, count) = result.unwrap();
        assert_eq!(digest, "0x3333333333333333333333333333333333333333333333333333333333333333"); // Last digest
        assert_eq!(count, 3);

        fs::remove_file(temp_audit).ok();
    }

    #[test]
    fn test_read_audit_tail_file_not_found() {
        let result = Manifest::read_audit_tail("/nonexistent/path/audit.jsonl");
        assert!(result.is_err());
    }

    // --- to_canonical_json() Test ---

    #[test]
    fn test_to_canonical_json() {
        let manifest = create_test_manifest();
        let result = manifest.to_canonical_json();
        assert!(result.is_ok());

        let json_str = result.unwrap();
        assert!(json_str.contains("manifest.v1.0"));
        assert!(json_str.contains("0xabc"));
        assert!(json_str.contains("Test"));
    }

    // --- SignedManifest roundtrip Test ---

    #[test]
    fn test_signed_manifest_roundtrip() {
        use std::fs;
        let temp_path = "/tmp/test_signed_manifest.json";

        let manifest = create_test_manifest();
        let signature = SignatureInfo {
            alg: "Ed25519".to_string(),
            signer: "company".to_string(),
            pubkey_hex: "0x123abc".to_string(),
            sig_hex: "0xdeadbeef".to_string(),
        };

        let signed = SignedManifest { manifest, signature };
        signed.save(temp_path).unwrap();

        let loaded = SignedManifest::load(temp_path).unwrap();
        assert_eq!(loaded.manifest.version, "manifest.v1.0");
        assert_eq!(loaded.signature.alg, "Ed25519");
        assert_eq!(loaded.signature.sig_hex, "0xdeadbeef");

        fs::remove_file(temp_path).ok();
    }

    // --- PublicChain serde Test ---

    #[test]
    fn test_public_chain_serde() {
        // Test all PublicChain variants serialize/deserialize correctly
        let chains = vec![
            (PublicChain::Ethereum, "\"ethereum\""),
            (PublicChain::Hedera, "\"hedera\""),
            (PublicChain::Btc, "\"btc\""),
        ];

        for (chain, expected_json) in chains {
            let json = serde_json::to_string(&chain).unwrap();
            assert_eq!(json, expected_json);

            let deserialized: PublicChain = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, chain);
        }
    }

    // --- Helper ---

    fn create_test_manifest() -> Manifest {
        Manifest {
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
        }
    }
}
