/// Package Verifier – I/O-based Proof Package Verification
///
/// This module provides package-level verification that loads files from disk.
/// For pure, portable verification logic, see `cap_agent::verifier::core` in the library.
use crate::manifest::Manifest;
use crate::proof_engine::Proof;
use crate::verifier::VerifyStatus;
use cap_agent::bundle::meta::{
    check_dependency_cycles, load_bundle_meta, BundleMeta, ProofUnitMeta,
};
use cap_agent::crypto::{hex_lower_prefixed32, sha3_256};
use cap_agent::verifier::core::{
    extract_statement_from_manifest, verify as core_verify, VerifyOptions,
};
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

/// Bundle Type Detection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BundleType {
    /// Modern bundle with _meta.json (cap-bundle.v1)
    Modern,
    /// Legacy bundle without _meta.json
    Legacy,
}

/// Detects bundle type based on _meta.json presence
fn detect_bundle_type(package_dir: &Path) -> BundleType {
    let meta_path = package_dir.join("_meta.json");
    if meta_path.exists() {
        BundleType::Modern
    } else {
        BundleType::Legacy
    }
}

/// 🔒 FILE SIZE LIMIT: 100 MB (DoS Prevention)
const MAX_FILE_SIZE: u64 = 100 * 1024 * 1024;

/// Validates file hash with Load-Once-Pattern (TOCTOU mitigation)
///
/// 1. Reads file into memory (single operation)
/// 2. Validates file size (DoS prevention)
/// 3. Computes SHA3-256 hash of memory bytes
/// 4. Returns memory bytes if hash matches
///
/// # Security
/// - TOCTOU mitigation: File read only once
/// - DoS prevention: File size limit (100 MB)
/// - Memory-safe: Hash computed on Vec<u8>
fn validate_file_hash(file_path: &Path, expected_hash: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    // 1. Load file into memory (single read)
    let bytes = fs::read(file_path)?;

    // 2. Check file size limit (DoS prevention)
    if bytes.len() as u64 > MAX_FILE_SIZE {
        return Err(format!(
            "File size exceeds limit: {} > {} MB",
            bytes.len() / 1024 / 1024,
            MAX_FILE_SIZE / 1024 / 1024
        )
        .into());
    }

    // 3. Compute hash from memory (not from disk!)
    let computed_hash = hex_lower_prefixed32(sha3_256(&bytes));

    // 4. Validate hash
    if computed_hash != expected_hash {
        return Err(format!(
            "Hash mismatch for {}: expected {}, got {}",
            file_path.display(),
            expected_hash,
            computed_hash
        )
        .into());
    }

    // 5. Return memory bytes (Load-Once-Pattern)
    Ok(bytes)
}

/// Loads and validates all files from bundle metadata
///
/// Returns HashMap: filename → validated bytes
fn load_and_validate_bundle(
    meta: &BundleMeta,
    bundle_dir: &Path,
) -> Result<HashMap<String, Vec<u8>>, Box<dyn Error>> {
    let mut validated_files = HashMap::new();

    for (filename, file_meta) in &meta.files {
        // Skip optional files that don't exist
        if file_meta.optional {
            let file_path = bundle_dir.join(filename);
            if !file_path.exists() {
                println!("   ⊘ {} (optional, nicht vorhanden)", filename);
                continue;
            }
        }

        // Validate and load file
        let file_path = bundle_dir.join(filename);
        println!("   🔐 Validiere Hash: {}", filename);
        let bytes = validate_file_hash(&file_path, &file_meta.hash)?;
        validated_files.insert(filename.clone(), bytes);
    }

    Ok(validated_files)
}

/// Verifier-Ergebnis
#[derive(Debug)]
pub struct VerificationResult {
    pub success: bool,
    pub manifest_hash: String,
    pub policy_hash: String,
    pub proof_status: String,
    pub checks_passed: usize,
    pub checks_total: usize,
}

/// Verifier für Proof-Pakete
pub struct Verifier {
    package_dir: PathBuf,
}

impl Verifier {
    /// Erstellt einen neuen Verifier für ein Proof-Paket
    ///
    /// # Argumente
    /// * `package_dir` - Pfad zum Proof-Paket-Verzeichnis
    ///
    /// # Rückgabe
    /// Neuer Verifier
    pub fn new<P: AsRef<Path>>(package_dir: P) -> Self {
        Verifier {
            package_dir: package_dir.as_ref().to_path_buf(),
        }
    }

    /// Verifiziert ein vollständiges Proof-Paket
    ///
    /// Supports both Modern (cap-bundle.v1 with _meta.json) and Legacy bundles.
    ///
    /// # Modern Bundle Verification (cap-bundle.v1)
    /// - Loads and validates _meta.json
    /// - Validates file hashes with Load-Once-Pattern (TOCTOU mitigation)
    /// - Checks dependency cycles in proof_units
    /// - Uses Core-Verify API for portable verification
    ///
    /// # Legacy Bundle Verification (Fallback)
    /// - Direct manifest.json + proof.dat loading
    /// - Uses old proof_engine verification
    ///
    /// # Rückgabe
    /// VerificationResult oder Fehler
    pub fn verify(&self) -> Result<VerificationResult, Box<dyn Error>> {
        // Detect bundle type
        let bundle_type = detect_bundle_type(&self.package_dir);

        match bundle_type {
            BundleType::Modern => {
                println!("🔍 Erkanntes Bundle-Format: cap-bundle.v1 (Modern)");
                self.verify_modern_bundle()
            }
            BundleType::Legacy => {
                println!("🔍 Erkanntes Bundle-Format: Legacy (ohne _meta.json)");
                self.verify_legacy_bundle()
            }
        }
    }

    /// Modern bundle verification with cap-bundle.v1 format
    fn verify_modern_bundle(&self) -> Result<VerificationResult, Box<dyn Error>> {
        // 1. Load and validate _meta.json
        println!("📋 Lade Bundle-Metadaten...");
        let meta = load_bundle_meta(&self.package_dir)?;

        // 2. Check dependency cycles
        println!("🔄 Prüfe Proof-Unit-Abhängigkeiten...");
        check_dependency_cycles(&meta.proof_units)?;
        println!("   ✅ Keine zirkulären Abhängigkeiten");

        // 3. Load and validate all files with Load-Once-Pattern
        println!("🔐 Validiere Datei-Hashes (Load-Once-Pattern)...");
        let validated_files = load_and_validate_bundle(&meta, &self.package_dir)?;
        println!("   ✅ Alle Datei-Hashes validiert");

        // 4. Extract manifest and proof from validated bytes
        let manifest_bytes = validated_files
            .get("manifest.json")
            .ok_or("manifest.json missing in validated files")?;
        let proof_bytes = validated_files
            .get("proof.dat")
            .ok_or("proof.dat missing in validated files")?;

        // 5. Parse manifest JSON
        let manifest_json: serde_json::Value = serde_json::from_slice(manifest_bytes)?;

        // 6. Extract statement from manifest
        println!("📝 Extrahiere Proof-Statement...");
        let stmt = extract_statement_from_manifest(&manifest_json)?;

        // 7. Verify with Core-Verify API
        println!("🔍 Führe Kern-Verifikation aus...");
        let opts = VerifyOptions {
            check_timestamp: false,
            check_registry: false,
        };
        let report = core_verify(&manifest_json, proof_bytes, &stmt, &opts)?;

        // 8. Build result
        Ok(VerificationResult {
            success: report.status == "ok",
            manifest_hash: report.manifest_hash,
            policy_hash: stmt.policy_hash,
            proof_status: report.status,
            checks_passed: 0, // Core-Verify doesn't expose constraint counts
            checks_total: 0,
        })
    }

    /// Legacy bundle verification (fallback for old bundles)
    fn verify_legacy_bundle(&self) -> Result<VerificationResult, Box<dyn Error>> {
        // 1. Lade Manifest
        let manifest_path = self.package_dir.join("manifest.json");
        if !manifest_path.exists() {
            return Err("manifest.json nicht gefunden im Proof-Paket".into());
        }
        let manifest = Manifest::load(&manifest_path)?;

        // 2. Lade Proof
        let proof_path = self.package_dir.join("proof.dat");
        if !proof_path.exists() {
            return Err("proof.dat nicht gefunden im Proof-Paket".into());
        }
        let proof = Proof::load_from_dat(&proof_path)?;

        // 3. Verifiziere Proof gegen Manifest
        proof.verify(&manifest)?;

        // 4. Zähle Checks
        let checks_total = proof.proof_data.checked_constraints.len();
        let checks_passed = proof
            .proof_data
            .checked_constraints
            .iter()
            .filter(|c| c.ok)
            .count();

        Ok(VerificationResult {
            success: true,
            manifest_hash: proof.manifest_hash.clone(),
            policy_hash: proof.policy_hash.clone(),
            proof_status: proof.status.clone(),
            checks_passed,
            checks_total,
        })
    }

    /// Extrahiert Manifest-Informationen
    ///
    /// # Rückgabe
    /// Manifest-Objekt oder Fehler
    pub fn extract_manifest(&self) -> Result<Manifest, Box<dyn Error>> {
        let manifest_path = self.package_dir.join("manifest.json");
        Manifest::load(&manifest_path)
    }

    /// Extrahiert Proof-Informationen
    ///
    /// # Rückgabe
    /// Proof-Objekt oder Fehler
    pub fn extract_proof(&self) -> Result<Proof, Box<dyn Error>> {
        let proof_path = self.package_dir.join("proof.dat");
        Proof::load_from_dat(&proof_path)
    }

    /// Zeigt Audit-Trail an
    ///
    /// # Rückgabe
    /// Tuple (tail_digest, events_count) oder Fehler
    pub fn show_audit_trail(&self) -> Result<(String, u64), Box<dyn Error>> {
        let manifest = self.extract_manifest()?;
        Ok((
            manifest.audit.tail_digest.clone(),
            manifest.audit.events_count,
        ))
    }

    /// Prüft ob alle erforderlichen Dateien vorhanden sind
    ///
    /// # Rückgabe
    /// Result mit Statusmeldung
    pub fn check_package_integrity(&self) -> Result<String, Box<dyn Error>> {
        let mut missing = Vec::new();

        if !self.package_dir.join("manifest.json").exists() {
            missing.push("manifest.json");
        }
        if !self.package_dir.join("proof.dat").exists() {
            missing.push("proof.dat");
        }

        if !missing.is_empty() {
            return Err(format!("Fehlende Dateien: {}", missing.join(", ")).into());
        }

        let has_signature = self.package_dir.join("signature.json").exists();

        Ok(format!(
            "Proof-Paket vollständig (Signatur: {})",
            if has_signature { "ja" } else { "nein" }
        ))
    }
}

/// Zeigt eine formatierte Zusammenfassung eines Proof-Pakets
///
/// # Argumente
/// * `package_dir` - Pfad zum Proof-Paket
///
/// # Rückgabe
/// Formatierter String oder Fehler
pub fn show_package_summary<P: AsRef<Path>>(package_dir: P) -> Result<String, Box<dyn Error>> {
    let verifier = Verifier::new(package_dir);

    let manifest = verifier.extract_manifest()?;
    let proof = verifier.extract_proof()?;

    let mut summary = String::new();
    summary.push_str("=== PROOF-PAKET ZUSAMMENFASSUNG ===\n\n");

    summary.push_str("Manifest:\n");
    summary.push_str(&format!("  Version: {}\n", manifest.version));
    summary.push_str(&format!("  Erstellt: {}\n", manifest.created_at));
    summary.push_str(&format!(
        "  Company Root: {}\n",
        manifest.company_commitment_root
    ));
    summary.push_str(&format!(
        "  Policy: {} ({})\n",
        manifest.policy.name, manifest.policy.version
    ));
    summary.push_str(&format!("  Policy Hash: {}\n", manifest.policy.hash));
    summary.push_str(&format!(
        "  Audit Events: {}\n",
        manifest.audit.events_count
    ));
    summary.push_str(&format!("  Audit Tail: {}\n\n", manifest.audit.tail_digest));

    summary.push_str("Proof:\n");
    summary.push_str(&format!("  Version: {}\n", proof.version));
    summary.push_str(&format!("  Typ: {}\n", proof.proof_type));
    summary.push_str(&format!("  Statement: {}\n", proof.statement));
    summary.push_str(&format!("  Status: {}\n", proof.status));
    summary.push_str(&format!(
        "  Checks: {}/{}\n",
        proof
            .proof_data
            .checked_constraints
            .iter()
            .filter(|c| c.ok)
            .count(),
        proof.proof_data.checked_constraints.len()
    ));

    summary.push_str("\nConstraint-Checks:\n");
    for check in &proof.proof_data.checked_constraints {
        summary.push_str(&format!(
            "  {} {}\n",
            if check.ok { "✅" } else { "❌" },
            check.name
        ));
    }

    Ok(summary)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::{AuditInfo, Manifest, ProofInfo};
    use crate::policy::PolicyInfo;
    use crate::proof_engine::{ConstraintCheck, Proof, ProofData};
    use std::fs;

    #[test]
    fn test_verifier_check_integrity() {
        let test_dir = "/tmp/test_proof_package";
        let _ = fs::remove_dir_all(test_dir);
        fs::create_dir_all(test_dir).unwrap();

        // Erstelle Dummy-Dateien
        fs::write(format!("{}/manifest.json", test_dir), "{}").unwrap();
        fs::write(format!("{}/proof.dat", test_dir), "dummy").unwrap();

        let verifier = Verifier::new(test_dir);
        let result = verifier.check_package_integrity().unwrap();
        assert!(result.contains("vollständig"));

        fs::remove_dir_all(test_dir).ok();
    }

    #[test]
    fn test_verifier_missing_files() {
        let test_dir = "/tmp/test_proof_package_empty";
        let _ = fs::remove_dir_all(test_dir);
        fs::create_dir_all(test_dir).unwrap();

        let verifier = Verifier::new(test_dir);
        let result = verifier.check_package_integrity();
        assert!(result.is_err());

        fs::remove_dir_all(test_dir).ok();
    }

    #[test]
    fn test_package_summary() {
        let test_dir = "/tmp/test_proof_summary";
        let _ = fs::remove_dir_all(test_dir);
        fs::create_dir_all(test_dir).unwrap();

        // Erstelle Test-Manifest
        let manifest = Manifest {
            version: "manifest.v1.0".to_string(),
            created_at: "2025-10-25T10:00:00Z".to_string(),
            supplier_root: "0xabc".to_string(),
            ubo_root: "0xdef".to_string(),
            company_commitment_root: "0x123".to_string(),
            policy: PolicyInfo {
                name: "Test Policy".to_string(),
                version: "lksg.v1".to_string(),
                hash: "0xpolicy".to_string(),
            },
            audit: AuditInfo {
                tail_digest: "0xtail".to_string(),
                events_count: 10,
            },
            proof: ProofInfo {
                proof_type: "mock".to_string(),
                status: "ok".to_string(),
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
                    name: "test_check".to_string(),
                    ok: true,
                }],
            },
            status: "ok".to_string(),
        };

        manifest
            .save(format!("{}/manifest.json", test_dir))
            .unwrap();
        proof
            .save_as_dat(format!("{}/proof.dat", test_dir))
            .unwrap();

        let summary = show_package_summary(test_dir).unwrap();
        assert!(summary.contains("PROOF-PAKET"));
        assert!(summary.contains("Test Policy"));

        fs::remove_dir_all(test_dir).ok();
    }

    // ====================================================================================
    // Neue Tests für Coverage-Erweiterung (68% -> 80%+)
    // ====================================================================================

    // --- verify() Tests ---

    #[test]
    fn test_verify_success() {
        let test_dir = "/tmp/test_verify_success";
        let _ = fs::remove_dir_all(test_dir);
        fs::create_dir_all(test_dir).unwrap();

        let manifest = create_test_manifest();
        let manifest_hash = Proof::compute_manifest_hash(&manifest).unwrap();

        let proof = Proof {
            version: "proof.v0".to_string(),
            proof_type: "mock".to_string(),
            statement: "policy:lksg.v1".to_string(),
            manifest_hash: manifest_hash.clone(),
            policy_hash: "0xpolicy".to_string(),
            proof_data: ProofData {
                checked_constraints: vec![
                    ConstraintCheck {
                        name: "check1".to_string(),
                        ok: true,
                    },
                    ConstraintCheck {
                        name: "check2".to_string(),
                        ok: true,
                    },
                ],
            },
            status: "ok".to_string(),
        };

        manifest
            .save(format!("{}/manifest.json", test_dir))
            .unwrap();
        proof
            .save_as_dat(format!("{}/proof.dat", test_dir))
            .unwrap();

        let verifier = Verifier::new(test_dir);
        let result = verifier.verify().unwrap();

        assert!(result.success);
        assert_eq!(result.manifest_hash, manifest_hash);
        assert_eq!(result.policy_hash, "0xpolicy");
        assert_eq!(result.proof_status, "ok");
        assert_eq!(result.checks_passed, 2);
        assert_eq!(result.checks_total, 2);

        fs::remove_dir_all(test_dir).ok();
    }

    #[test]
    fn test_verify_manifest_not_found() {
        let test_dir = "/tmp/test_verify_no_manifest";
        let _ = fs::remove_dir_all(test_dir);
        fs::create_dir_all(test_dir).unwrap();

        let verifier = Verifier::new(test_dir);
        let result = verifier.verify();

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("nicht gefunden"));

        fs::remove_dir_all(test_dir).ok();
    }

    #[test]
    fn test_verify_proof_not_found() {
        let test_dir = "/tmp/test_verify_no_proof";
        let _ = fs::remove_dir_all(test_dir);
        fs::create_dir_all(test_dir).unwrap();

        let manifest = create_test_manifest();
        manifest
            .save(format!("{}/manifest.json", test_dir))
            .unwrap();

        let verifier = Verifier::new(test_dir);
        let result = verifier.verify();

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("proof.dat"));

        fs::remove_dir_all(test_dir).ok();
    }

    #[test]
    fn test_verify_fails_with_failed_constraint() {
        let test_dir = "/tmp/test_verify_partial";
        let _ = fs::remove_dir_all(test_dir);
        fs::create_dir_all(test_dir).unwrap();

        let manifest = create_test_manifest();
        let manifest_hash = Proof::compute_manifest_hash(&manifest).unwrap();

        let proof = Proof {
            version: "proof.v0".to_string(),
            proof_type: "mock".to_string(),
            statement: "policy:lksg.v1".to_string(),
            manifest_hash,
            policy_hash: "0xpolicy".to_string(),
            proof_data: ProofData {
                checked_constraints: vec![
                    ConstraintCheck {
                        name: "check1".to_string(),
                        ok: true,
                    },
                    ConstraintCheck {
                        name: "check2".to_string(),
                        ok: false,
                    },
                    ConstraintCheck {
                        name: "check3".to_string(),
                        ok: true,
                    },
                ],
            },
            status: "ok".to_string(),
        };

        manifest
            .save(format!("{}/manifest.json", test_dir))
            .unwrap();
        proof
            .save_as_dat(format!("{}/proof.dat", test_dir))
            .unwrap();

        let verifier = Verifier::new(test_dir);
        let result = verifier.verify();

        // verify() should fail because constraint check2 failed
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("ist fehlgeschlagen"));

        fs::remove_dir_all(test_dir).ok();
    }

    // --- extract_manifest() Tests ---

    #[test]
    fn test_extract_manifest_success() {
        let test_dir = "/tmp/test_extract_manifest";
        let _ = fs::remove_dir_all(test_dir);
        fs::create_dir_all(test_dir).unwrap();

        let manifest = create_test_manifest();
        manifest
            .save(format!("{}/manifest.json", test_dir))
            .unwrap();

        let verifier = Verifier::new(test_dir);
        let extracted = verifier.extract_manifest().unwrap();

        assert_eq!(extracted.version, "manifest.v1.0");
        assert_eq!(extracted.policy.name, "Test Policy");

        fs::remove_dir_all(test_dir).ok();
    }

    #[test]
    fn test_extract_manifest_not_found() {
        let test_dir = "/tmp/test_extract_manifest_missing";
        let _ = fs::remove_dir_all(test_dir);
        fs::create_dir_all(test_dir).unwrap();

        let verifier = Verifier::new(test_dir);
        let result = verifier.extract_manifest();

        assert!(result.is_err());

        fs::remove_dir_all(test_dir).ok();
    }

    // --- extract_proof() Tests ---

    #[test]
    fn test_extract_proof_success() {
        let test_dir = "/tmp/test_extract_proof";
        let _ = fs::remove_dir_all(test_dir);
        fs::create_dir_all(test_dir).unwrap();

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
            status: "ok".to_string(),
        };

        proof
            .save_as_dat(format!("{}/proof.dat", test_dir))
            .unwrap();

        let verifier = Verifier::new(test_dir);
        let extracted = verifier.extract_proof().unwrap();

        assert_eq!(extracted.version, "proof.v0");
        assert_eq!(extracted.proof_type, "mock");

        fs::remove_dir_all(test_dir).ok();
    }

    #[test]
    fn test_extract_proof_not_found() {
        let test_dir = "/tmp/test_extract_proof_missing";
        let _ = fs::remove_dir_all(test_dir);
        fs::create_dir_all(test_dir).unwrap();

        let verifier = Verifier::new(test_dir);
        let result = verifier.extract_proof();

        assert!(result.is_err());

        fs::remove_dir_all(test_dir).ok();
    }

    // --- show_audit_trail() Tests ---

    #[test]
    fn test_show_audit_trail_success() {
        let test_dir = "/tmp/test_audit_trail";
        let _ = fs::remove_dir_all(test_dir);
        fs::create_dir_all(test_dir).unwrap();

        let manifest = create_test_manifest();
        manifest
            .save(format!("{}/manifest.json", test_dir))
            .unwrap();

        let verifier = Verifier::new(test_dir);
        let (tail_digest, events_count) = verifier.show_audit_trail().unwrap();

        assert_eq!(tail_digest, "0xtail");
        assert_eq!(events_count, 10);

        fs::remove_dir_all(test_dir).ok();
    }

    #[test]
    fn test_show_audit_trail_manifest_not_found() {
        let test_dir = "/tmp/test_audit_trail_missing";
        let _ = fs::remove_dir_all(test_dir);
        fs::create_dir_all(test_dir).unwrap();

        let verifier = Verifier::new(test_dir);
        let result = verifier.show_audit_trail();

        assert!(result.is_err());

        fs::remove_dir_all(test_dir).ok();
    }

    // --- check_package_integrity() Edge Cases ---

    #[test]
    fn test_integrity_missing_manifest_only() {
        let test_dir = "/tmp/test_integrity_no_manifest";
        let _ = fs::remove_dir_all(test_dir);
        fs::create_dir_all(test_dir).unwrap();

        fs::write(format!("{}/proof.dat", test_dir), "dummy").unwrap();

        let verifier = Verifier::new(test_dir);
        let result = verifier.check_package_integrity();

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("manifest.json"));

        fs::remove_dir_all(test_dir).ok();
    }

    #[test]
    fn test_integrity_missing_proof_only() {
        let test_dir = "/tmp/test_integrity_no_proof";
        let _ = fs::remove_dir_all(test_dir);
        fs::create_dir_all(test_dir).unwrap();

        fs::write(format!("{}/manifest.json", test_dir), "{}").unwrap();

        let verifier = Verifier::new(test_dir);
        let result = verifier.check_package_integrity();

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("proof.dat"));

        fs::remove_dir_all(test_dir).ok();
    }

    #[test]
    fn test_integrity_with_signature() {
        let test_dir = "/tmp/test_integrity_with_sig";
        let _ = fs::remove_dir_all(test_dir);
        fs::create_dir_all(test_dir).unwrap();

        fs::write(format!("{}/manifest.json", test_dir), "{}").unwrap();
        fs::write(format!("{}/proof.dat", test_dir), "dummy").unwrap();
        fs::write(format!("{}/signature.json", test_dir), "{}").unwrap();

        let verifier = Verifier::new(test_dir);
        let result = verifier.check_package_integrity().unwrap();

        assert!(result.contains("vollständig"));
        assert!(result.contains("ja"));

        fs::remove_dir_all(test_dir).ok();
    }

    // --- show_package_summary() Error Cases ---

    #[test]
    fn test_summary_manifest_missing() {
        let test_dir = "/tmp/test_summary_no_manifest";
        let _ = fs::remove_dir_all(test_dir);
        fs::create_dir_all(test_dir).unwrap();

        let result = show_package_summary(test_dir);
        assert!(result.is_err());

        fs::remove_dir_all(test_dir).ok();
    }

    #[test]
    fn test_summary_multiple_constraints() {
        let test_dir = "/tmp/test_summary_multi";
        let _ = fs::remove_dir_all(test_dir);
        fs::create_dir_all(test_dir).unwrap();

        let manifest = create_test_manifest();
        let manifest_hash = Proof::compute_manifest_hash(&manifest).unwrap();

        let proof = Proof {
            version: "proof.v0".to_string(),
            proof_type: "mock".to_string(),
            statement: "policy:lksg.v1".to_string(),
            manifest_hash,
            policy_hash: "0xpolicy".to_string(),
            proof_data: ProofData {
                checked_constraints: vec![
                    ConstraintCheck {
                        name: "check1".to_string(),
                        ok: true,
                    },
                    ConstraintCheck {
                        name: "check2".to_string(),
                        ok: false,
                    },
                    ConstraintCheck {
                        name: "check3".to_string(),
                        ok: true,
                    },
                ],
            },
            status: "ok".to_string(),
        };

        manifest
            .save(format!("{}/manifest.json", test_dir))
            .unwrap();
        proof
            .save_as_dat(format!("{}/proof.dat", test_dir))
            .unwrap();

        let summary = show_package_summary(test_dir).unwrap();
        assert!(summary.contains("Checks: 2/3"));
        assert!(summary.contains("✅ check1"));
        assert!(summary.contains("❌ check2"));
        assert!(summary.contains("✅ check3"));

        fs::remove_dir_all(test_dir).ok();
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
                name: "Test Policy".to_string(),
                version: "lksg.v1".to_string(),
                hash: "0xpolicy".to_string(),
            },
            audit: AuditInfo {
                tail_digest: "0xtail".to_string(),
                events_count: 10,
            },
            proof: ProofInfo {
                proof_type: "mock".to_string(),
                status: "ok".to_string(),
            },
            signatures: Vec::new(),
            time_anchor: None,
        }
    }
}

// ====================================================================================
// Bundle Verifier – cap-bundle.v1 Support
// ====================================================================================

use anyhow::{anyhow, Result};

/// Bundle-Verifier für cap-bundle.v1 Pakete
pub struct BundleVerifier {
    bundle_dir: PathBuf,
}

impl BundleVerifier {
    /// Erstellt einen neuen BundleVerifier
    ///
    /// # Arguments
    /// * `bundle_dir` - Pfad zum Bundle-Verzeichnis
    ///
    /// # Returns
    /// Neuer BundleVerifier
    pub fn new<P: AsRef<Path>>(bundle_dir: P) -> Self {
        BundleVerifier {
            bundle_dir: bundle_dir.as_ref().to_path_buf(),
        }
    }

    /// Verifiziert ein vollständiges cap-bundle.v1 Paket
    ///
    /// # Verifikationsschritte
    /// 1. Lade _meta.json
    /// 2. Validiere Bundle-Schema
    /// 3. Prüfe Dependency-Zyklen
    /// 4. Verifiziere jede Proof Unit mit Core-Verify API
    /// 5. Validiere File-Hashes gegen _meta.json
    ///
    /// # Returns
    /// BundleVerifyResult oder Fehler
    pub fn verify_bundle(&self) -> Result<BundleVerifyResult> {
        // 1. Lade Bundle-Metadaten
        eprintln!("DEBUG: Loading bundle meta from {:?}", self.bundle_dir);
        let meta = load_bundle_meta(&self.bundle_dir)?;
        eprintln!(
            "DEBUG: Loaded meta, found {} proof units",
            meta.proof_units.len()
        );

        // 2. Validiere Dependency-Zyklen
        check_dependency_cycles(&meta.proof_units)?;
        eprintln!("DEBUG: Dependency cycles checked");

        // 3. Verifiziere jede Proof Unit
        let mut unit_results = Vec::new();

        for unit in &meta.proof_units {
            eprintln!("DEBUG: Verifying unit: {}", unit.id);
            let result = self.verify_proof_unit(unit, &meta)?;
            unit_results.push((unit.id.clone(), result));
        }

        // 4. Aggregiere Gesamtstatus
        let overall_status = aggregate_status(&unit_results);

        Ok(BundleVerifyResult {
            bundle_id: meta.bundle_id,
            schema: meta.schema,
            created_at: meta.created_at,
            status: overall_status,
            unit_results,
        })
    }

    /// Verifiziert eine einzelne Proof Unit
    fn verify_proof_unit(
        &self,
        unit: &ProofUnitMeta,
        meta: &BundleMeta,
    ) -> Result<cap_agent::verifier::core::VerifyReport> {
        // 1. Lade Manifest
        let manifest_path = self.bundle_dir.join(&unit.manifest_file);
        eprintln!("DEBUG: Loading manifest: {:?}", manifest_path);
        let manifest_bytes = std::fs::read(&manifest_path)?;
        eprintln!("DEBUG: Loaded {} bytes", manifest_bytes.len());

        // 2. Lade Proof
        let proof_path = self.bundle_dir.join(&unit.proof_file);
        eprintln!("DEBUG: Loading proof: {:?}", proof_path);
        let proof_bytes = std::fs::read(&proof_path)?;
        eprintln!("DEBUG: Loaded {} bytes", proof_bytes.len());

        // 3. Validiere Hashes gegen _meta.json
        let manifest_file_meta = meta.files.get(&unit.manifest_file).ok_or_else(|| {
            anyhow!(
                "Manifest file not found in _meta.json: {}",
                unit.manifest_file
            )
        })?;

        let proof_file_meta = meta
            .files
            .get(&unit.proof_file)
            .ok_or_else(|| anyhow!("Proof file not found in _meta.json: {}", unit.proof_file))?;

        let computed_manifest_hash =
            crate::crypto::hex_lower_prefixed32(crate::crypto::sha3_256(&manifest_bytes));
        let computed_proof_hash =
            crate::crypto::hex_lower_prefixed32(crate::crypto::sha3_256(&proof_bytes));

        if computed_manifest_hash != manifest_file_meta.hash {
            return Err(anyhow!(
                "Manifest hash mismatch: expected {}, got {}",
                manifest_file_meta.hash,
                computed_manifest_hash
            ));
        }

        if computed_proof_hash != proof_file_meta.hash {
            return Err(anyhow!(
                "Proof hash mismatch: expected {}, got {}",
                proof_file_meta.hash,
                computed_proof_hash
            ));
        }

        // 4. Parse manifest and extract statement
        eprintln!("DEBUG: Parsing manifest JSON");
        let manifest_json: serde_json::Value = serde_json::from_slice(&manifest_bytes)?;
        let stmt = extract_statement_from_manifest(&manifest_json)?;

        // 5. Create verify options
        let opts = VerifyOptions {
            check_timestamp: false,
            check_registry: false,
        };

        // 6. Call Core-Verify API
        eprintln!(
            "DEBUG: Calling core_verify() with manifest_hash={}, proof_hash={}",
            computed_manifest_hash, computed_proof_hash
        );
        let report = core_verify(&manifest_json, &proof_bytes, &stmt, &opts)?;
        eprintln!("DEBUG: core_verify() returned status={}", report.status);
        Ok(report)
    }
}

/// Bundle-Verifikations-Ergebnis
#[derive(Debug)]
pub struct BundleVerifyResult {
    pub bundle_id: String,
    pub schema: String,
    #[allow(dead_code)] // Reserved for future reporting/serialization
    pub created_at: String,
    pub status: VerifyStatus,
    pub unit_results: Vec<(String, cap_agent::verifier::core::VerifyReport)>,
}

/// Aggregiert den Gesamtstatus aus allen Unit-Ergebnissen
fn aggregate_status(
    unit_results: &[(String, cap_agent::verifier::core::VerifyReport)],
) -> VerifyStatus {
    unit_results
        .iter()
        .fold(VerifyStatus::Ok, |acc, (_id, report)| {
            let unit_status = match report.status.as_str() {
                "ok" => VerifyStatus::Ok,
                "fail" => VerifyStatus::Fail,
                _ => VerifyStatus::Warn,
            };
            match (acc, unit_status) {
                (VerifyStatus::Error, _) | (_, VerifyStatus::Error) => VerifyStatus::Error,
                (VerifyStatus::Fail, _) | (_, VerifyStatus::Fail) => VerifyStatus::Fail,
                (VerifyStatus::Warn, _) | (_, VerifyStatus::Warn) => VerifyStatus::Warn,
                _ => VerifyStatus::Ok,
            }
        })
}
