use crate::manifest::Manifest;
use crate::proof_engine::Proof;
use std::error::Error;
use std::path::{Path, PathBuf};

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
    /// Prüft:
    /// - Manifest existiert und ist gültig
    /// - Proof.dat existiert und ist dekodierbar
    /// - Proof verifiziert gegen Manifest
    /// - Optional: Signatur-Verifikation
    ///
    /// # Rückgabe
    /// VerificationResult oder Fehler
    pub fn verify(&self) -> Result<VerificationResult, Box<dyn Error>> {
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
    summary.push_str(&format!("  Company Root: {}\n", manifest.company_commitment_root));
    summary.push_str(&format!("  Policy: {} ({})\n", manifest.policy.name, manifest.policy.version));
    summary.push_str(&format!("  Policy Hash: {}\n", manifest.policy.hash));
    summary.push_str(&format!("  Audit Events: {}\n", manifest.audit.events_count));
    summary.push_str(&format!("  Audit Tail: {}\n\n", manifest.audit.tail_digest));

    summary.push_str("Proof:\n");
    summary.push_str(&format!("  Version: {}\n", proof.version));
    summary.push_str(&format!("  Typ: {}\n", proof.proof_type));
    summary.push_str(&format!("  Statement: {}\n", proof.statement));
    summary.push_str(&format!("  Status: {}\n", proof.status));
    summary.push_str(&format!("  Checks: {}/{}\n",
        proof.proof_data.checked_constraints.iter().filter(|c| c.ok).count(),
        proof.proof_data.checked_constraints.len()));

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
            version: "manifest.v0".to_string(),
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
        proof.save_as_dat(format!("{}/proof.dat", test_dir)).unwrap();

        let summary = show_package_summary(test_dir).unwrap();
        assert!(summary.contains("PROOF-PAKET"));
        assert!(summary.contains("Test Policy"));

        fs::remove_dir_all(test_dir).ok();
    }
}
