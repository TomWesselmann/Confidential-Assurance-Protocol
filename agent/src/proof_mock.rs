use crate::io::JsonPersistent;
use crate::manifest::Manifest;
use crate::policy::Policy;
use serde::{Deserialize, Serialize};
use std::error::Error;

/// Mock-Proof Check-Result
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CheckResult {
    pub name: String,
    pub ok: bool,
}

/// Mock-Proof Details
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MockProofDetails {
    pub checks: Vec<CheckResult>,
}

/// Mock-Proof-Datenstruktur
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MockProof {
    #[serde(rename = "type")]
    pub proof_type: String,
    pub policy_hash: String,
    pub company_commitment_root: String,
    pub status: String,
    pub details: MockProofDetails,
}

impl MockProof {
    /// Generiert einen Mock-Proof basierend auf Policy und Manifest
    ///
    /// Dies ist KEIN echter Zero-Knowledge-Proof, sondern nur eine strukturierte
    /// Darstellung von Policy-Checks für Demonstrationszwecke.
    ///
    /// # Argumente
    /// * `policy` - Die Policy-Regeln
    /// * `manifest` - Das zu prüfende Manifest
    /// * `supplier_count` - Anzahl der Suppliers (für Constraint-Check)
    /// * `ubo_count` - Anzahl der UBOs (für Constraint-Check)
    ///
    /// # Rückgabe
    /// MockProof-Objekt
    pub fn generate(
        policy: &Policy,
        manifest: &Manifest,
        supplier_count: usize,
        ubo_count: usize,
    ) -> Result<Self, Box<dyn Error>> {
        let mut checks = Vec::new();

        // Check 1: Mindestens ein UBO erforderlich
        if policy.constraints.require_at_least_one_ubo {
            checks.push(CheckResult {
                name: "require_at_least_one_ubo".to_string(),
                ok: ubo_count >= 1,
            });
        }

        // Check 2: Supplier-Anzahl innerhalb des Limits
        let supplier_check_ok = (supplier_count as u32) <= policy.constraints.supplier_count_max;
        checks.push(CheckResult {
            name: format!(
                "supplier_count_max_{}",
                policy.constraints.supplier_count_max
            ),
            ok: supplier_check_ok,
        });

        // Gesamtstatus: alle Checks müssen OK sein
        let all_ok = checks.iter().all(|c| c.ok);

        Ok(MockProof {
            proof_type: "mock".to_string(),
            policy_hash: manifest.policy.hash.clone(),
            company_commitment_root: manifest.company_commitment_root.clone(),
            status: if all_ok {
                "ok".to_string()
            } else {
                "failed".to_string()
            },
            details: MockProofDetails { checks },
        })
    }

    /// Verifiziert einen Mock-Proof (prüft Hash-Konsistenz und Status)
    ///
    /// # Argumente
    /// * `manifest` - Das Manifest, gegen das geprüft werden soll
    ///
    /// # Rückgabe
    /// Result mit () bei Erfolg oder Fehler
    #[allow(dead_code)]
    pub fn verify(&self, manifest: &Manifest) -> Result<(), Box<dyn Error>> {
        // Prüfe, ob Policy-Hash übereinstimmt
        if self.policy_hash != manifest.policy.hash {
            return Err(format!(
                "Policy-Hash-Mismatch: erwartet {}, gefunden {}",
                manifest.policy.hash, self.policy_hash
            )
            .into());
        }

        // Prüfe, ob Company-Root übereinstimmt
        if self.company_commitment_root != manifest.company_commitment_root {
            return Err(format!(
                "Company-Root-Mismatch: erwartet {}, gefunden {}",
                manifest.company_commitment_root, self.company_commitment_root
            )
            .into());
        }

        // Prüfe Status
        if self.status != "ok" {
            return Err(format!("Proof-Status ist nicht OK: {}", self.status).into());
        }

        // Prüfe, ob alle Checks OK sind
        for check in &self.details.checks {
            if !check.ok {
                return Err(format!("Check '{}' ist fehlgeschlagen", check.name).into());
            }
        }

        Ok(())
    }

}

/// JsonPersistent Trait für MockProof - ermöglicht load()/save() via Trait
impl JsonPersistent for MockProof {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::{AuditInfo, Manifest, ProofInfo};
    use crate::policy::{Policy, PolicyConstraints, PolicyInfo};

    #[test]
    fn test_mock_proof_generation_success() {
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

        let proof = MockProof::generate(&policy, &manifest, 5, 2).unwrap();
        assert_eq!(proof.status, "ok");
        assert_eq!(proof.proof_type, "mock");
        assert!(proof.details.checks.len() >= 2);
    }

    #[test]
    fn test_mock_proof_generation_failure() {
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

        // 10 Suppliers, aber max ist 5
        let proof = MockProof::generate(&policy, &manifest, 10, 2).unwrap();
        assert_eq!(proof.status, "failed");
    }

    #[test]
    fn test_mock_proof_verify() {
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

        let proof = MockProof {
            proof_type: "mock".to_string(),
            policy_hash: "0xpolicy".to_string(),
            company_commitment_root: "0x123".to_string(),
            status: "ok".to_string(),
            details: MockProofDetails {
                checks: vec![CheckResult {
                    name: "test".to_string(),
                    ok: true,
                }],
            },
        };

        assert!(proof.verify(&manifest).is_ok());
    }
}
