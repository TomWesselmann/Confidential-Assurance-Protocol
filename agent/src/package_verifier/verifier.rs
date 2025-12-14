//! Verifier - Main package verification logic
//!
//! Provides the Verifier struct for verifying proof packages,
//! supporting both modern (cap-bundle.v1) and legacy formats.

use crate::manifest::Manifest;
use crate::proof_engine::Proof;
use crate::bundle::meta::{check_dependency_cycles, load_bundle_meta, BundleMeta, ProofUnitMeta};
use crate::verifier::core::{
    extract_statement_from_manifest, verify as core_verify, VerifyOptions,
};
use std::error::Error;
use std::path::{Path, PathBuf};

use super::types::{aggregate_status, BundleType, BundleVerifyResult, VerificationResult};
use super::validation::load_and_validate_bundle;

/// Detects bundle type based on _meta.json presence
pub fn detect_bundle_type(package_dir: &Path) -> BundleType {
    let meta_path = package_dir.join("_meta.json");
    if meta_path.exists() {
        BundleType::Modern
    } else {
        BundleType::Legacy
    }
}

/// Verifier für Proof-Pakete
pub struct Verifier {
    pub(crate) package_dir: PathBuf,
}

impl Verifier {
    /// Erstellt einen neuen Verifier für ein Proof-Paket
    pub fn new<P: AsRef<Path>>(package_dir: P) -> Self {
        Verifier {
            package_dir: package_dir.as_ref().to_path_buf(),
        }
    }

    /// Verifiziert ein vollständiges Proof-Paket
    ///
    /// Supports both Modern (cap-bundle.v1 with _meta.json) and Legacy bundles.
    pub fn verify(&self) -> Result<VerificationResult, Box<dyn Error>> {
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
            checks_passed: 0,
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
    pub fn extract_manifest(&self) -> Result<Manifest, Box<dyn Error>> {
        let manifest_path = self.package_dir.join("manifest.json");
        Manifest::load(&manifest_path)
    }

    /// Extrahiert Proof-Informationen
    pub fn extract_proof(&self) -> Result<Proof, Box<dyn Error>> {
        let proof_path = self.package_dir.join("proof.dat");
        Proof::load_from_dat(&proof_path)
    }

    /// Zeigt Audit-Trail an
    pub fn show_audit_trail(&self) -> Result<(String, u64), Box<dyn Error>> {
        let manifest = self.extract_manifest()?;
        Ok((
            manifest.audit.tail_digest.clone(),
            manifest.audit.events_count,
        ))
    }

    /// Prüft ob alle erforderlichen Dateien vorhanden sind
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

    /// Verifiziert ein cap-bundle.v1 Paket mit mehreren Proof Units
    pub fn verify_bundle(&self) -> anyhow::Result<BundleVerifyResult> {
        // 1. Lade Bundle-Metadaten
        let meta = load_bundle_meta(&self.package_dir)?;

        // 2. Validiere Dependency-Zyklen
        check_dependency_cycles(&meta.proof_units)?;

        // 3. Verifiziere jede Proof Unit
        let mut unit_results = Vec::new();

        for unit in &meta.proof_units {
            let result = self.verify_proof_unit_internal(unit, &meta)?;
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

    /// Interne Methode: Verifiziert eine einzelne Proof Unit
    fn verify_proof_unit_internal(
        &self,
        unit: &ProofUnitMeta,
        meta: &BundleMeta,
    ) -> anyhow::Result<crate::verifier::core::VerifyReport> {
        // 1. Lade Manifest
        let manifest_path = self.package_dir.join(&unit.manifest_file);
        let manifest_bytes = std::fs::read(&manifest_path)?;

        // 2. Lade Proof
        let proof_path = self.package_dir.join(&unit.proof_file);
        let proof_bytes = std::fs::read(&proof_path)?;

        // 3. Validiere Hashes gegen _meta.json
        let manifest_file_meta = meta.files.get(&unit.manifest_file).ok_or_else(|| {
            anyhow::anyhow!(
                "Manifest file not found in _meta.json: {}",
                unit.manifest_file
            )
        })?;

        let proof_file_meta = meta
            .files
            .get(&unit.proof_file)
            .ok_or_else(|| anyhow::anyhow!("Proof file not found in _meta.json: {}", unit.proof_file))?;

        let computed_manifest_hash =
            crate::crypto::hex_lower_prefixed32(crate::crypto::sha3_256(&manifest_bytes));
        let computed_proof_hash =
            crate::crypto::hex_lower_prefixed32(crate::crypto::sha3_256(&proof_bytes));

        if computed_manifest_hash != manifest_file_meta.hash {
            return Err(anyhow::anyhow!(
                "Manifest hash mismatch: expected {}, got {}",
                manifest_file_meta.hash,
                computed_manifest_hash
            ));
        }

        if computed_proof_hash != proof_file_meta.hash {
            return Err(anyhow::anyhow!(
                "Proof hash mismatch: expected {}, got {}",
                proof_file_meta.hash,
                computed_proof_hash
            ));
        }

        // 4. Parse manifest and extract statement
        let manifest_json: serde_json::Value = serde_json::from_slice(&manifest_bytes)?;
        let stmt = extract_statement_from_manifest(&manifest_json)?;

        // 5. Create verify options
        let opts = VerifyOptions {
            check_timestamp: false,
            check_registry: false,
        };

        // 6. Call Core-Verify API
        let report = core_verify(&manifest_json, &proof_bytes, &stmt, &opts)?;
        Ok(report)
    }
}

/// Type-Alias für API-Kompatibilität
///
/// `BundleVerifier` wurde in `Verifier` konsolidiert (Session 25).
pub type BundleVerifier = Verifier;
