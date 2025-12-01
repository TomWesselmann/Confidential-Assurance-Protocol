//! Manifest I/O - Build, save, load operations
//!
//! Provides manifest construction and persistence.

use crate::commitment::Commitments;
use crate::policy::PolicyInfo;
use chrono::Utc;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

use super::anchor::{PublicChain, TimeAnchor, TimeAnchorPrivate, TimeAnchorPublic};
use super::types::{AuditInfo, Manifest, ProofInfo, MANIFEST_SCHEMA_VERSION};

impl Manifest {
    /// Erstellt ein neues Manifest aus Commitments, Policy und Audit-Log
    pub fn build(
        commitments: &Commitments,
        policy_info: PolicyInfo,
        audit_log_path: &str,
    ) -> Result<Self, Box<dyn Error>> {
        let (tail_digest, events_count) = read_audit_tail(audit_log_path)?;

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

    /// Speichert Manifest als JSON
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn Error>> {
        let json = serde_json::to_string_pretty(self)?;
        let mut file = File::create(path)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }

    /// Lädt Manifest aus JSON-Datei
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let file = File::open(path)?;
        let manifest: Manifest = serde_json::from_reader(file)?;
        Ok(manifest)
    }

    /// Aktualisiert Proof-Informationen im Manifest
    #[allow(dead_code)]
    pub fn update_proof(&mut self, proof_type: String, status: String) {
        self.proof = ProofInfo { proof_type, status };
    }

    /// Setzt den Zeitanker im Manifest
    pub fn set_time_anchor(&mut self, kind: String, reference: String, audit_tip_hex: String) {
        self.time_anchor = Some(TimeAnchor::new(kind, reference, audit_tip_hex));
    }

    /// Setzt den Private Anchor (Dual-Anchor v0.9.0)
    pub fn set_private_anchor(
        &mut self,
        audit_tip_hex: String,
        created_at: Option<String>,
    ) -> Result<(), Box<dyn Error>> {
        let anchor = self
            .time_anchor
            .as_mut()
            .ok_or("time_anchor must be initialized before setting private anchor")?;

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
    pub fn validate_dual_anchor(&self) -> Result<(), Box<dyn Error>> {
        let anchor = match &self.time_anchor {
            Some(a) => a,
            None => return Ok(()),
        };

        if let Some(private) = &anchor.private {
            if private.audit_tip_hex != anchor.audit_tip_hex {
                return Err(format!(
                    "Private anchor audit_tip_hex mismatch: expected {}, got {}",
                    anchor.audit_tip_hex, private.audit_tip_hex
                )
                .into());
            }
        }

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
    pub fn to_canonical_json(&self) -> Result<String, Box<dyn Error>> {
        Ok(serde_json::to_string(self)?)
    }
}

/// Liest Tail-Digest und Event-Count aus Audit-Log
pub fn read_audit_tail(path: &str) -> Result<(String, u64), Box<dyn Error>> {
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
