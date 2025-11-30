use crate::manifest::Manifest;
use crate::policy::Policy;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};
use std::error::Error;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

/// Constraint-Check-Ergebnis
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConstraintCheck {
    pub name: String,
    pub ok: bool,
}

/// Proof-Daten (Mock-Format, ZK-Ready)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProofData {
    pub checked_constraints: Vec<ConstraintCheck>,
}

/// Proof-Objekt (v0 - Mock, später ZK)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Proof {
    pub version: String,
    #[serde(rename = "type")]
    pub proof_type: String,
    pub statement: String,
    pub manifest_hash: String,
    pub policy_hash: String,
    pub proof_data: ProofData,
    pub status: String,
}

impl Proof {
    /// Erstellt einen neuen Proof aus Policy, Manifest und Daten
    ///
    /// # Argumente
    /// * `policy` - Die Policy mit Constraints
    /// * `manifest` - Das Manifest mit Commitments
    /// * `supplier_count` - Anzahl der Suppliers (für Validierung)
    /// * `ubo_count` - Anzahl der UBOs (für Validierung)
    ///
    /// # Rückgabe
    /// Neuer Proof oder Fehler
    pub fn build(
        policy: &Policy,
        manifest: &Manifest,
        supplier_count: usize,
        ubo_count: usize,
    ) -> Result<Self, Box<dyn Error>> {
        let mut checks = Vec::new();

        // Check 1: Mindestens ein UBO erforderlich
        if policy.constraints.require_at_least_one_ubo {
            checks.push(ConstraintCheck {
                name: "require_at_least_one_ubo".to_string(),
                ok: ubo_count >= 1,
            });
        }

        // Check 2: Supplier-Anzahl innerhalb des Limits
        let supplier_check_ok = (supplier_count as u32) <= policy.constraints.supplier_count_max;
        checks.push(ConstraintCheck {
            name: format!(
                "supplier_count_max_{}",
                policy.constraints.supplier_count_max
            ),
            ok: supplier_check_ok,
        });

        // Gesamtstatus: alle Checks müssen OK sein
        let all_ok = checks.iter().all(|c| c.ok);

        // Berechne Manifest-Hash
        let manifest_hash = Self::compute_manifest_hash(manifest)?;

        Ok(Proof {
            version: "proof.v0".to_string(),
            proof_type: "mock".to_string(),
            statement: format!("policy:{}", policy.version),
            manifest_hash,
            policy_hash: manifest.policy.hash.clone(),
            proof_data: ProofData {
                checked_constraints: checks,
            },
            status: if all_ok {
                "ok".to_string()
            } else {
                "failed".to_string()
            },
        })
    }

    /// Berechnet SHA3-256 Hash eines Manifests
    ///
    /// # Argumente
    /// * `manifest` - Das Manifest
    ///
    /// # Rückgabe
    /// Hex-String des Manifest-Hashes
    pub fn compute_manifest_hash(manifest: &Manifest) -> Result<String, Box<dyn Error>> {
        let json = serde_json::to_string(manifest)?;
        let mut hasher = Sha3_256::new();
        hasher.update(json.as_bytes());
        let result = hasher.finalize();
        Ok(format!("0x{}", hex::encode(result)))
    }

    /// Verifiziert einen Proof gegen ein Manifest
    ///
    /// # Argumente
    /// * `manifest` - Das Manifest zur Verifikation
    ///
    /// # Rückgabe
    /// Result mit () bei Erfolg oder Fehler
    pub fn verify(&self, manifest: &Manifest) -> Result<(), Box<dyn Error>> {
        // Prüfe Manifest-Hash
        let expected_hash = Self::compute_manifest_hash(manifest)?;
        if self.manifest_hash != expected_hash {
            return Err(format!(
                "Manifest-Hash-Mismatch: erwartet {}, gefunden {}",
                expected_hash, self.manifest_hash
            )
            .into());
        }

        // Prüfe Policy-Hash
        if self.policy_hash != manifest.policy.hash {
            return Err(format!(
                "Policy-Hash-Mismatch: erwartet {}, gefunden {}",
                manifest.policy.hash, self.policy_hash
            )
            .into());
        }

        // Prüfe Status
        if self.status != "ok" {
            return Err(format!("Proof-Status ist nicht OK: {}", self.status).into());
        }

        // Prüfe alle Constraints
        for check in &self.proof_data.checked_constraints {
            if !check.ok {
                return Err(format!("Constraint '{}' ist fehlgeschlagen", check.name).into());
            }
        }

        Ok(())
    }

    /// Speichert Proof als JSON-Datei
    ///
    /// # Argumente
    /// * `path` - Zielpfad
    ///
    /// # Rückgabe
    /// Result
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn Error>> {
        let json = serde_json::to_string_pretty(self)?;
        let mut file = File::create(path)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }

    /// Lädt Proof aus JSON-Datei
    ///
    /// # Argumente
    /// * `path` - Pfad zur Datei
    ///
    /// # Rückgabe
    /// Proof-Objekt
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let file = File::open(path)?;
        let proof: Proof = serde_json::from_reader(file)?;
        Ok(proof)
    }

    /// Speichert Proof als Base64-kodierte Datei (proof.dat Format)
    ///
    /// # Argumente
    /// * `path` - Zielpfad für .dat Datei
    ///
    /// # Rückgabe
    /// Result
    pub fn save_as_dat<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn Error>> {
        use base64::{engine::general_purpose, Engine as _};

        let json = serde_json::to_string(self)?;
        let encoded = general_purpose::STANDARD.encode(json.as_bytes());

        let mut file = File::create(path)?;
        file.write_all(encoded.as_bytes())?;
        Ok(())
    }

    /// Lädt Proof aus Base64-kodierter .dat Datei
    ///
    /// # Argumente
    /// * `path` - Pfad zur .dat Datei
    ///
    /// # Rückgabe
    /// Proof-Objekt
    pub fn load_from_dat<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        use base64::{engine::general_purpose, Engine as _};

        let encoded = fs::read_to_string(path)?;
        let decoded = general_purpose::STANDARD.decode(encoded.trim())?;
        let json = String::from_utf8(decoded)?;
        let proof: Proof = serde_json::from_str(&json)?;
        Ok(proof)
    }
}

/// Exportiert ein vollständiges Proof-Paket
///
/// # Argumente
/// * `manifest` - Das Manifest
/// * `proof` - Der Proof
/// * `signature_path` - Optional: Pfad zur Signatur-Datei
/// * `output_dir` - Zielverzeichnis für das Paket
///
/// # Rückgabe
/// Result
#[allow(dead_code)]
pub fn export_proof_package<P: AsRef<Path>>(
    manifest: &Manifest,
    proof: &Proof,
    signature_path: Option<&str>,
    output_dir: P,
) -> Result<(), Box<dyn Error>> {
    let dir = output_dir.as_ref();

    // Erstelle Verzeichnis
    fs::create_dir_all(dir)?;

    // Kopiere/Erstelle manifest.json
    let manifest_path = dir.join("manifest.json");
    manifest.save(&manifest_path)?;

    // Erstelle proof.dat (Base64)
    let proof_path = dir.join("proof.dat");
    proof.save_as_dat(&proof_path)?;

    // Kopiere signature.json falls vorhanden
    if let Some(sig_path) = signature_path {
        let sig_dest = dir.join("signature.json");
        fs::copy(sig_path, sig_dest)?;
    }

    // Erstelle README.txt
    let readme_path = dir.join("README.txt");
    let mut readme = File::create(readme_path)?;
    writeln!(readme, "LkSG Proof Package")?;
    writeln!(readme, "==================")?;
    writeln!(readme)?;
    writeln!(readme, "Generated: {}", Utc::now().to_rfc3339())?;
    writeln!(readme, "Proof Version: {}", proof.version)?;
    writeln!(readme, "Proof Type: {}", proof.proof_type)?;
    writeln!(readme, "Status: {}", proof.status)?;
    writeln!(readme)?;
    writeln!(readme, "Files:")?;
    writeln!(readme, "  - manifest.json: Compliance manifest")?;
    writeln!(readme, "  - proof.dat: Base64-encoded proof")?;
    if signature_path.is_some() {
        writeln!(readme, "  - signature.json: Ed25519 signature")?;
    }
    writeln!(readme)?;
    writeln!(readme, "Verification:")?;
    writeln!(
        readme,
        "  cargo run -- verifier run --package {}",
        dir.display()
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::{AuditInfo, Manifest, ProofInfo};
    use crate::policy::{Policy, PolicyConstraints, PolicyInfo};

    #[test]
    fn test_proof_build_success() {
        let policy = Policy {
            version: "lksg.v1".to_string(),
            name: "Test".to_string(),
            created_at: "2025-10-25T09:00:00Z".to_string(),
            constraints: PolicyConstraints {
                require_at_least_one_ubo: true,
                supplier_count_max: 10,
                ubo_count_min: None,
                require_statement_roots: None,
            },
            notes: "".to_string(),
        };

        let manifest = Manifest {
            version: "manifest.v1.0".to_string(),
            created_at: "2025-10-25T10:00:00Z".to_string(),
            supplier_root: "0xabc".to_string(),
            ubo_root: "0xdef".to_string(),
            company_commitment_root: "0x123".to_string(),
            policy: PolicyInfo {
                name: "Test".to_string(),
                version: "lksg.v1".to_string(),
                hash: "0xpolicy".to_string(),
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

        let proof = Proof::build(&policy, &manifest, 5, 2).unwrap();
        assert_eq!(proof.version, "proof.v0");
        assert_eq!(proof.proof_type, "mock");
        assert_eq!(proof.status, "ok");
        assert_eq!(proof.statement, "policy:lksg.v1");
    }

    #[test]
    fn test_proof_verify() {
        let manifest = Manifest {
            version: "manifest.v1.0".to_string(),
            created_at: "2025-10-25T10:00:00Z".to_string(),
            supplier_root: "0xabc".to_string(),
            ubo_root: "0xdef".to_string(),
            company_commitment_root: "0x123".to_string(),
            policy: PolicyInfo {
                name: "Test".to_string(),
                version: "lksg.v1".to_string(),
                hash: "0xpolicy".to_string(),
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

        let manifest_hash = Proof::compute_manifest_hash(&manifest).unwrap();

        let proof = Proof {
            version: "proof.v0".to_string(),
            proof_type: "mock".to_string(),
            statement: "policy:lksg.v1".to_string(),
            manifest_hash,
            policy_hash: "0xpolicy".to_string(),
            proof_data: ProofData {
                checked_constraints: vec![ConstraintCheck {
                    name: "test".to_string(),
                    ok: true,
                }],
            },
            status: "ok".to_string(),
        };

        assert!(proof.verify(&manifest).is_ok());
    }

    #[test]
    fn test_proof_dat_serialization() {
        let proof = Proof {
            version: "proof.v0".to_string(),
            proof_type: "mock".to_string(),
            statement: "policy:lksg.v1".to_string(),
            manifest_hash: "0xabc".to_string(),
            policy_hash: "0xdef".to_string(),
            proof_data: ProofData {
                checked_constraints: vec![],
            },
            status: "ok".to_string(),
        };

        let temp_path = "/tmp/test_proof.dat";
        proof.save_as_dat(temp_path).unwrap();

        let loaded = Proof::load_from_dat(temp_path).unwrap();
        assert_eq!(proof.version, loaded.version);
        assert_eq!(proof.manifest_hash, loaded.manifest_hash);

        std::fs::remove_file(temp_path).ok();
    }

    // ====================================================================================
    // Neue Tests für Coverage-Erweiterung (49% -> 65%+)
    // ====================================================================================

    // --- build() Edge Cases ---

    #[test]
    fn test_proof_build_failed_status() {
        let policy = Policy {
            version: "lksg.v1".to_string(),
            name: "Test".to_string(),
            created_at: "2025-10-25T09:00:00Z".to_string(),
            constraints: PolicyConstraints {
                require_at_least_one_ubo: true,
                supplier_count_max: 5,
                ubo_count_min: None,
                require_statement_roots: None,
            },
            notes: "".to_string(),
        };

        let manifest = create_test_manifest();

        // supplier_count = 10 > supplier_count_max = 5 → failed
        let proof = Proof::build(&policy, &manifest, 10, 2).unwrap();

        assert_eq!(proof.status, "failed");
        assert_eq!(proof.proof_data.checked_constraints.len(), 2);

        // Check 1: UBO check should pass (ubo_count=2 >= 1)
        assert!(proof.proof_data.checked_constraints[0].ok);

        // Check 2: Supplier check should fail (10 > 5)
        assert!(!proof.proof_data.checked_constraints[1].ok);
    }

    #[test]
    fn test_proof_build_ubo_check_disabled() {
        let policy = Policy {
            version: "lksg.v1".to_string(),
            name: "Test".to_string(),
            created_at: "2025-10-25T09:00:00Z".to_string(),
            constraints: PolicyConstraints {
                require_at_least_one_ubo: false, // Disabled
                supplier_count_max: 10,
                ubo_count_min: None,
                require_statement_roots: None,
            },
            notes: "".to_string(),
        };

        let manifest = create_test_manifest();

        // ubo_count = 0 but check is disabled
        let proof = Proof::build(&policy, &manifest, 5, 0).unwrap();

        assert_eq!(proof.status, "ok");
        // Only supplier check should exist
        assert_eq!(proof.proof_data.checked_constraints.len(), 1);
        assert_eq!(
            proof.proof_data.checked_constraints[0].name,
            "supplier_count_max_10"
        );
    }

    #[test]
    fn test_proof_build_ubo_check_failed() {
        let policy = Policy {
            version: "lksg.v1".to_string(),
            name: "Test".to_string(),
            created_at: "2025-10-25T09:00:00Z".to_string(),
            constraints: PolicyConstraints {
                require_at_least_one_ubo: true,
                supplier_count_max: 10,
                ubo_count_min: None,
                require_statement_roots: None,
            },
            notes: "".to_string(),
        };

        let manifest = create_test_manifest();

        // ubo_count = 0 < 1 → failed
        let proof = Proof::build(&policy, &manifest, 5, 0).unwrap();

        assert_eq!(proof.status, "failed");
        assert!(!proof.proof_data.checked_constraints[0].ok);
        assert_eq!(
            proof.proof_data.checked_constraints[0].name,
            "require_at_least_one_ubo"
        );
    }

    // --- verify() Error Cases ---

    #[test]
    fn test_verify_manifest_hash_mismatch() {
        let manifest = create_test_manifest();

        let proof = Proof {
            version: "proof.v0".to_string(),
            proof_type: "mock".to_string(),
            statement: "policy:lksg.v1".to_string(),
            manifest_hash: "0xWRONG".to_string(), // Wrong hash
            policy_hash: "0xpolicy".to_string(),
            proof_data: ProofData {
                checked_constraints: vec![],
            },
            status: "ok".to_string(),
        };

        let result = proof.verify(&manifest);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Manifest-Hash"));
    }

    #[test]
    fn test_verify_policy_hash_mismatch() {
        let manifest = create_test_manifest();
        let manifest_hash = Proof::compute_manifest_hash(&manifest).unwrap();

        let proof = Proof {
            version: "proof.v0".to_string(),
            proof_type: "mock".to_string(),
            statement: "policy:lksg.v1".to_string(),
            manifest_hash,
            policy_hash: "0xWRONG".to_string(), // Wrong hash
            proof_data: ProofData {
                checked_constraints: vec![],
            },
            status: "ok".to_string(),
        };

        let result = proof.verify(&manifest);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Policy-Hash"));
    }

    #[test]
    fn test_verify_status_not_ok() {
        let manifest = create_test_manifest();
        let manifest_hash = Proof::compute_manifest_hash(&manifest).unwrap();

        let proof = Proof {
            version: "proof.v0".to_string(),
            proof_type: "mock".to_string(),
            statement: "policy:lksg.v1".to_string(),
            manifest_hash,
            policy_hash: "0xpolicy".to_string(),
            proof_data: ProofData {
                checked_constraints: vec![],
            },
            status: "failed".to_string(), // Not OK
        };

        let result = proof.verify(&manifest);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Status ist nicht OK"));
    }

    #[test]
    fn test_verify_constraint_failed() {
        let manifest = create_test_manifest();
        let manifest_hash = Proof::compute_manifest_hash(&manifest).unwrap();

        let proof = Proof {
            version: "proof.v0".to_string(),
            proof_type: "mock".to_string(),
            statement: "policy:lksg.v1".to_string(),
            manifest_hash,
            policy_hash: "0xpolicy".to_string(),
            proof_data: ProofData {
                checked_constraints: vec![ConstraintCheck {
                    name: "test_check".to_string(),
                    ok: false, // Failed constraint
                }],
            },
            status: "ok".to_string(),
        };

        let result = proof.verify(&manifest);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("fehlgeschlagen"));
    }

    // --- save() / load() Roundtrip ---

    #[test]
    fn test_proof_json_save_load_roundtrip() {
        let proof = Proof {
            version: "proof.v0".to_string(),
            proof_type: "mock".to_string(),
            statement: "policy:lksg.v1".to_string(),
            manifest_hash: "0xabc123".to_string(),
            policy_hash: "0xdef456".to_string(),
            proof_data: ProofData {
                checked_constraints: vec![ConstraintCheck {
                    name: "test".to_string(),
                    ok: true,
                }],
            },
            status: "ok".to_string(),
        };

        let temp_path = "/tmp/test_proof.json";
        proof.save(temp_path).unwrap();

        let loaded = Proof::load(temp_path).unwrap();
        assert_eq!(proof.version, loaded.version);
        assert_eq!(proof.manifest_hash, loaded.manifest_hash);
        assert_eq!(proof.policy_hash, loaded.policy_hash);
        assert_eq!(proof.status, loaded.status);

        std::fs::remove_file(temp_path).ok();
    }

    #[test]
    fn test_load_nonexistent_file() {
        let result = Proof::load("/tmp/nonexistent_proof.json");
        assert!(result.is_err());
    }

    // --- compute_manifest_hash() ---

    #[test]
    fn test_compute_manifest_hash_deterministic() {
        let manifest = create_test_manifest();

        let hash1 = Proof::compute_manifest_hash(&manifest).unwrap();
        let hash2 = Proof::compute_manifest_hash(&manifest).unwrap();

        assert_eq!(hash1, hash2);
        assert!(hash1.starts_with("0x"));
        assert_eq!(hash1.len(), 66); // "0x" + 64 hex chars
    }

    // --- Helper ---

    fn create_test_manifest() -> Manifest {
        Manifest {
            version: "manifest.v1.0".to_string(),
            created_at: "2025-10-25T10:00:00Z".to_string(),
            supplier_root: "0xabc".to_string(),
            ubo_root: "0xdef".to_string(),
            company_commitment_root: "0x123".to_string(),
            policy: PolicyInfo {
                name: "Test".to_string(),
                version: "lksg.v1".to_string(),
                hash: "0xpolicy".to_string(),
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
