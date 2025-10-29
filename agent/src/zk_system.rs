use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};
use std::error::Error;

/// Statement für Zero-Knowledge-Proof
/// Enthält öffentliche Informationen, die verifiziert werden
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Statement {
    /// Policy-Hash (öffentlich bekannt)
    pub policy_hash: String,
    /// Company Commitment Root (öffentlich bekannt)
    pub company_commitment_root: String,
    /// Constraint-Liste (öffentlich)
    pub constraints: Vec<String>,
    /// Optionale Sanctions-Root (für externe Listen)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sanctions_root: Option<String>,
    /// Optionale Jurisdiction-Root (für externe Listen)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jurisdiction_root: Option<String>,
}

/// Witness für Zero-Knowledge-Proof
/// Enthält private Daten, die NICHT offengelegt werden
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Witness {
    /// Gehashte Supplier-Daten (privat)
    pub suppliers: Vec<String>,
    /// Gehashte UBO-Daten (privat)
    pub ubos: Vec<String>,
    /// Supplier-Count (privat)
    pub supplier_count: usize,
    /// UBO-Count (privat)
    pub ubo_count: usize,
}

/// ZK-Proof-Struktur
/// Enthält den Zero-Knowledge-Beweis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZkProof {
    /// Version des Proof-Systems
    pub version: String,
    /// ZK-System-Typ ("simplified", "halo2", "spartan", "risc0")
    pub system: String,
    /// Serialisierte Beweis-Daten (Base64 oder Binär)
    pub proof_data: Vec<u8>,
    /// Öffentliche Inputs (Statement)
    pub public_inputs: Statement,
    /// Status ("ok" oder "failed")
    pub status: String,
    /// Zeitstempel der Proof-Erstellung
    pub created_at: String,
}

/// ProofSystem-Trait
/// Definiert die Schnittstelle für alle ZK-Proof-Systeme
pub trait ProofSystem {
    /// Erstellt einen Zero-Knowledge-Proof
    ///
    /// # Argumente
    /// * `statement` - Öffentliche Statement-Daten
    /// * `witness` - Private Witness-Daten
    ///
    /// # Rückgabe
    /// ZkProof oder Fehler
    fn prove(&self, statement: &Statement, witness: &Witness) -> Result<ZkProof, Box<dyn Error>>;

    /// Verifiziert einen Zero-Knowledge-Proof
    ///
    /// # Argumente
    /// * `proof` - Der zu verifizierende Proof
    ///
    /// # Rückgabe
    /// true wenn gültig, false oder Fehler sonst
    fn verify(&self, proof: &ZkProof) -> Result<bool, Box<dyn Error>>;

    /// Gibt den Namen des Proof-Systems zurück
    #[allow(dead_code)]
    fn name(&self) -> &str;
}

/// Simplified ZK-Backend
/// Mock-Implementierung für MVP, kann später durch echte ZK-Library ersetzt werden
pub struct SimplifiedZK {
    name: String,
}

impl SimplifiedZK {
    /// Erstellt eine neue SimplifiedZK-Instanz
    pub fn new() -> Self {
        SimplifiedZK {
            name: "simplified".to_string(),
        }
    }

    /// Prüft Constraints gegen Witness
    fn check_constraints(
        &self,
        statement: &Statement,
        witness: &Witness,
    ) -> Result<Vec<(String, bool)>, Box<dyn Error>> {
        let mut results = Vec::new();

        for constraint in &statement.constraints {
            let passed = match constraint.as_str() {
                "require_at_least_one_ubo" => witness.ubo_count >= 1,
                c if c.starts_with("supplier_count_max_") => {
                    let max_str = c.replace("supplier_count_max_", "");
                    let max: usize = max_str.parse()?;
                    witness.supplier_count <= max
                }
                _ => {
                    return Err(format!("Unbekannte Constraint: {}", constraint).into());
                }
            };
            results.push((constraint.clone(), passed));
        }

        Ok(results)
    }

    /// Berechnet Proof-Hash aus Statement + Witness
    fn compute_proof_hash(
        &self,
        statement: &Statement,
        witness: &Witness,
    ) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut hasher = Sha3_256::new();

        // Hash Statement
        let statement_json = serde_json::to_string(statement)?;
        hasher.update(statement_json.as_bytes());

        // Hash Witness (ohne Offenlegung)
        let witness_json = serde_json::to_string(witness)?;
        hasher.update(witness_json.as_bytes());

        // Hash Constraints-Ergebnisse
        let checks = self.check_constraints(statement, witness)?;
        let checks_json = serde_json::to_string(&checks)?;
        hasher.update(checks_json.as_bytes());

        Ok(hasher.finalize().to_vec())
    }
}

impl Default for SimplifiedZK {
    fn default() -> Self {
        Self::new()
    }
}

impl ProofSystem for SimplifiedZK {
    fn prove(&self, statement: &Statement, witness: &Witness) -> Result<ZkProof, Box<dyn Error>> {
        // Prüfe Constraints
        let checks = self.check_constraints(statement, witness)?;
        let all_passed = checks.iter().all(|(_, ok)| *ok);

        // Berechne Proof-Hash
        let proof_hash = self.compute_proof_hash(statement, witness)?;

        // Erstelle Proof-Daten (Simplified: nur Hash + Checks)
        #[derive(Serialize)]
        struct SimplifiedProofData {
            #[allow(dead_code)]
            proof_hash: String,
            checks: Vec<(String, bool)>,
            #[allow(dead_code)]
            witness_commitment: String, // Hash des Witness (ohne Daten)
        }

        // Witness-Commitment (Hash ohne Offenlegung)
        let witness_json = serde_json::to_string(witness)?;
        let mut witness_hasher = Sha3_256::new();
        witness_hasher.update(witness_json.as_bytes());
        let witness_commitment = format!("0x{}", hex::encode(witness_hasher.finalize()));

        let proof_data = SimplifiedProofData {
            proof_hash: format!("0x{}", hex::encode(&proof_hash)),
            checks,
            witness_commitment,
        };

        let proof_bytes = serde_json::to_vec(&proof_data)?;

        Ok(ZkProof {
            version: "zk.v1".to_string(),
            system: self.name.clone(),
            proof_data: proof_bytes,
            public_inputs: statement.clone(),
            status: if all_passed { "ok".to_string() } else { "failed".to_string() },
            created_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    fn verify(&self, proof: &ZkProof) -> Result<bool, Box<dyn Error>> {
        // Prüfe System-Typ
        if proof.system != self.name {
            return Err(format!(
                "Proof-System-Mismatch: erwartet {}, gefunden {}",
                self.name, proof.system
            )
            .into());
        }

        // Prüfe Status
        if proof.status != "ok" {
            return Ok(false);
        }

        // Dekodiere Proof-Daten
        #[derive(Deserialize)]
        struct SimplifiedProofData {
            #[allow(dead_code)]
            proof_hash: String,
            checks: Vec<(String, bool)>,
            #[allow(dead_code)]
            witness_commitment: String,
        }

        let proof_data: SimplifiedProofData = serde_json::from_slice(&proof.proof_data)?;

        // Prüfe alle Checks
        let all_passed = proof_data.checks.iter().all(|(_, ok)| *ok);
        if !all_passed {
            return Ok(false);
        }

        // Prüfe Statement-Konsistenz
        if proof_data.checks.len() != proof.public_inputs.constraints.len() {
            return Ok(false);
        }

        Ok(true)
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// Speichert ZkProof als JSON-Datei
///
/// # Argumente
/// * `proof` - Der ZkProof
/// * `path` - Zielpfad
pub fn save_zk_proof_json<P: AsRef<std::path::Path>>(
    proof: &ZkProof,
    path: P,
) -> Result<(), Box<dyn Error>> {
    let json = serde_json::to_string_pretty(proof)?;
    std::fs::write(path, json)?;
    Ok(())
}

/// Lädt ZkProof aus JSON-Datei
///
/// # Argumente
/// * `path` - Pfad zur Datei
pub fn load_zk_proof_json<P: AsRef<std::path::Path>>(
    path: P,
) -> Result<ZkProof, Box<dyn Error>> {
    let json = std::fs::read_to_string(path)?;
    let proof: ZkProof = serde_json::from_str(&json)?;
    Ok(proof)
}

/// Speichert ZkProof als Base64-kodierte .dat Datei
///
/// # Argumente
/// * `proof` - Der ZkProof
/// * `path` - Zielpfad
pub fn save_zk_proof_dat<P: AsRef<std::path::Path>>(
    proof: &ZkProof,
    path: P,
) -> Result<(), Box<dyn Error>> {
    use base64::{engine::general_purpose, Engine as _};

    let json = serde_json::to_string(proof)?;
    let encoded = general_purpose::STANDARD.encode(json.as_bytes());
    std::fs::write(path, encoded)?;
    Ok(())
}

/// Lädt ZkProof aus Base64-kodierter .dat Datei
///
/// # Argumente
/// * `path` - Pfad zur Datei
pub fn load_zk_proof_dat<P: AsRef<std::path::Path>>(
    path: P,
) -> Result<ZkProof, Box<dyn Error>> {
    use base64::{engine::general_purpose, Engine as _};

    let encoded = std::fs::read_to_string(path)?;
    let decoded = general_purpose::STANDARD.decode(encoded.trim())?;
    let json = String::from_utf8(decoded)?;
    let proof: ZkProof = serde_json::from_str(&json)?;
    Ok(proof)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simplified_zk_prove_success() {
        let zk = SimplifiedZK::new();

        let statement = Statement {
            policy_hash: "0xtest".to_string(),
            company_commitment_root: "0xroot".to_string(),
            constraints: vec![
                "require_at_least_one_ubo".to_string(),
                "supplier_count_max_10".to_string(),
            ],
            sanctions_root: None,
            jurisdiction_root: None,
        };

        let witness = Witness {
            suppliers: vec!["0xsup1".to_string(), "0xsup2".to_string()],
            ubos: vec!["0xubo1".to_string()],
            supplier_count: 2,
            ubo_count: 1,
        };

        let proof = zk.prove(&statement, &witness).unwrap();
        assert_eq!(proof.system, "simplified");
        assert_eq!(proof.status, "ok");
    }

    #[test]
    fn test_simplified_zk_verify_success() {
        let zk = SimplifiedZK::new();

        let statement = Statement {
            policy_hash: "0xtest".to_string(),
            company_commitment_root: "0xroot".to_string(),
            constraints: vec![
                "require_at_least_one_ubo".to_string(),
                "supplier_count_max_10".to_string(),
            ],
            sanctions_root: None,
            jurisdiction_root: None,
        };

        let witness = Witness {
            suppliers: vec!["0xsup1".to_string()],
            ubos: vec!["0xubo1".to_string()],
            supplier_count: 1,
            ubo_count: 1,
        };

        let proof = zk.prove(&statement, &witness).unwrap();
        let is_valid = zk.verify(&proof).unwrap();
        assert!(is_valid);
    }

    #[test]
    fn test_simplified_zk_prove_failure() {
        let zk = SimplifiedZK::new();

        let statement = Statement {
            policy_hash: "0xtest".to_string(),
            company_commitment_root: "0xroot".to_string(),
            constraints: vec!["require_at_least_one_ubo".to_string()],
            sanctions_root: None,
            jurisdiction_root: None,
        };

        let witness = Witness {
            suppliers: vec![],
            ubos: vec![], // Kein UBO -> sollte fehlschlagen
            supplier_count: 0,
            ubo_count: 0,
        };

        let proof = zk.prove(&statement, &witness).unwrap();
        assert_eq!(proof.status, "failed");
    }

    #[test]
    fn test_zk_proof_serialization() {
        let zk = SimplifiedZK::new();

        let statement = Statement {
            policy_hash: "0xtest".to_string(),
            company_commitment_root: "0xroot".to_string(),
            constraints: vec!["require_at_least_one_ubo".to_string()],
            sanctions_root: None,
            jurisdiction_root: None,
        };

        let witness = Witness {
            suppliers: vec![],
            ubos: vec!["0xubo1".to_string()],
            supplier_count: 0,
            ubo_count: 1,
        };

        let proof = zk.prove(&statement, &witness).unwrap();

        // Test JSON serialization
        let temp_json = "/tmp/test_zk_proof.json";
        save_zk_proof_json(&proof, temp_json).unwrap();
        let loaded_json = load_zk_proof_json(temp_json).unwrap();
        assert_eq!(proof.system, loaded_json.system);
        std::fs::remove_file(temp_json).ok();

        // Test DAT serialization
        let temp_dat = "/tmp/test_zk_proof.dat";
        save_zk_proof_dat(&proof, temp_dat).unwrap();
        let loaded_dat = load_zk_proof_dat(temp_dat).unwrap();
        assert_eq!(proof.system, loaded_dat.system);
        std::fs::remove_file(temp_dat).ok();
    }

    #[test]
    fn statement_optional_roots_serialization() {
        // Test mit optionalen Roots
        let statement_with_roots = Statement {
            policy_hash: "0xpolicy".to_string(),
            company_commitment_root: "0xcompany".to_string(),
            constraints: vec!["require_at_least_one_ubo".to_string()],
            sanctions_root: Some("0x3a1f02bb".to_string()),
            jurisdiction_root: Some("0x0c3f99aa".to_string()),
        };

        // Serialisiere zu JSON
        let json_with_roots = serde_json::to_string(&statement_with_roots).unwrap();
        assert!(json_with_roots.contains("sanctions_root"));
        assert!(json_with_roots.contains("jurisdiction_root"));

        // Deserialize zurück
        let deserialized: Statement = serde_json::from_str(&json_with_roots).unwrap();
        assert_eq!(deserialized.sanctions_root, Some("0x3a1f02bb".to_string()));
        assert_eq!(deserialized.jurisdiction_root, Some("0x0c3f99aa".to_string()));

        // Test ohne optionale Roots
        let statement_no_roots = Statement {
            policy_hash: "0xpolicy".to_string(),
            company_commitment_root: "0xcompany".to_string(),
            constraints: vec!["require_at_least_one_ubo".to_string()],
            sanctions_root: None,
            jurisdiction_root: None,
        };

        // Serialisiere zu JSON (skip_serializing_if sollte greifen)
        let json_no_roots = serde_json::to_string(&statement_no_roots).unwrap();
        assert!(!json_no_roots.contains("sanctions_root"));
        assert!(!json_no_roots.contains("jurisdiction_root"));
    }
}
