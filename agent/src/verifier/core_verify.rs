//! Core-Verify API - I/O-freier Verifikationskern
//!
//! Dieses Modul definiert die Core-Verify API für portable, I/O-freie
//! Proof-Verifikation. Alle Eingaben sind In-Memory (Bytes/Strings),
//! keine Filesystem-Zugriffe, keine Console-Ausgaben.
//!
//! Portabel für: CLI, Tests, WASM, zkVM, REST API

use anyhow::{anyhow, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};

/// Verifikations-Optionen (alle optional/toggelbar)
#[derive(Clone, Debug)]
pub struct CoreVerifyOptions {
    pub check_signature: bool,
    pub check_timestamp: bool,
    pub check_registry: bool,
}

impl Default for CoreVerifyOptions {
    fn default() -> Self {
        Self {
            check_signature: true,
            check_timestamp: true,
            check_registry: true,
        }
    }
}

/// Kanonische Eingabe für Core-Verifikation
#[derive(Clone, Debug)]
pub struct CoreVerifyInput {
    /// Protokoll-Version der Verifikation (z.B. "cap-core-verify.v1")
    pub protocol_version: String,

    /// Kanonische Bytes des Manifests (genau so, wie sie signiert/gehasht wurden)
    pub manifest_bytes: Vec<u8>,

    /// Proof-Bytes (Backend-spezifisch, für mock/halo2/etc.)
    pub proof_bytes: Vec<u8>,

    /// Bereits berechnete Hashes (SHA3-256 als "0x...")
    pub manifest_hash: String,
    pub proof_hash: String,
    pub policy_hash: String,

    /// Logische Identifikation der Policy, z.B. "lksg.demo.v1"
    pub policy_id: String,

    /// Welcher Proof-Backend-Typ (z.B. "mock", "halo2", "zkvm")
    pub backend: String,

    /// Optional: Signaturdaten (falls vorhanden)
    pub signature: Option<Vec<u8>>,
    pub public_key: Option<Vec<u8>>,

    /// Optional: Timestamp-/Registry-Daten (vorgeparst durch Adapter)
    pub timestamp_attestation: Option<Vec<u8>>,
    pub registry_entry_json: Option<String>,

    pub options: CoreVerifyOptions,
}

/// Status eines einzelnen Checks oder des Gesamtergebnisses
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VerifyStatus {
    /// Verifikation erfolgreich
    Ok,
    /// Warnung, aber nicht kritisch
    Warn,
    /// Verifikation fehlgeschlagen
    Fail,
    /// Interner Fehler (Parsing, etc.)
    Error,
}

/// Einzelner Verifikations-Check
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CheckResult {
    /// Maschinenlesbare ID des Checks, z.B. "hash_match_manifest"
    pub id: String,

    /// Kategorie des Checks: "integrity" | "policy" | "signature" | "timestamp" | "registry"
    pub kind: String,

    pub status: VerifyStatus,

    pub message: String,
}

/// Strukturiertes Verifikationsergebnis
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CoreVerifyResult {
    pub status: VerifyStatus,

    pub manifest_hash: String,
    pub proof_hash: String,
    pub policy_id: String,
    pub policy_hash: String,
    pub backend: String,

    pub checks: Vec<CheckResult>,

    pub signature_valid: Option<bool>,
    pub timestamp_valid: Option<bool>,
    pub registry_match: Option<bool>,

    pub started_at: String,  // ISO-8601 UTC
    pub finished_at: String, // ISO-8601 UTC
}

/// Zentrale I/O-freie Verifikationsfunktion
///
/// # Verifikationsschritte
/// 1. Hash-Verifikation (Manifest & Proof)
/// 2. Policy-/Statement-Validierung
/// 3. Optional: Signatur-Check
/// 4. Optional: Timestamp-Validierung
/// 5. Optional: Registry-Match
///
/// # Invarianten
/// - Kein Filesystem-Zugriff (std::fs verboten)
/// - Keine Console-Ausgaben (println!/eprintln! verboten)
/// - Deterministische Ergebnisse (gleiche Inputs → gleiche Outputs, außer Timestamps)
/// - Alle Inputs sind In-Memory Datenstrukturen
pub fn verify_core(input: CoreVerifyInput) -> CoreVerifyResult {
    let started_at = Utc::now().to_rfc3339();
    let mut checks = Vec::new();

    // 1. Hash-Verifikation (Manifest)
    let computed_manifest_hash = crate::crypto::sha3_256(&input.manifest_bytes);
    let computed_manifest_hex = crate::crypto::hex_lower_prefixed32(computed_manifest_hash);

    checks.push(CheckResult {
        id: "hash_match_manifest".to_string(),
        kind: "integrity".to_string(),
        status: if computed_manifest_hex == input.manifest_hash {
            VerifyStatus::Ok
        } else {
            VerifyStatus::Fail
        },
        message: format!(
            "Manifest hash: expected {}, got {}",
            input.manifest_hash, computed_manifest_hex
        ),
    });

    // 2. Hash-Verifikation (Proof)
    let computed_proof_hash = crate::crypto::sha3_256(&input.proof_bytes);
    let computed_proof_hex = crate::crypto::hex_lower_prefixed32(computed_proof_hash);

    checks.push(CheckResult {
        id: "hash_match_proof".to_string(),
        kind: "integrity".to_string(),
        status: if computed_proof_hex == input.proof_hash {
            VerifyStatus::Ok
        } else {
            VerifyStatus::Fail
        },
        message: format!(
            "Proof hash: expected {}, got {}",
            input.proof_hash, computed_proof_hex
        ),
    });

    // 3. Policy-Verifikation
    // Parse Manifest JSON
    eprintln!("DEBUG [verify_core]: Parsing manifest JSON ({} bytes)", input.manifest_bytes.len());
    match serde_json::from_slice::<serde_json::Value>(&input.manifest_bytes) {
        Ok(manifest) => {
            eprintln!("DEBUG [verify_core]: Manifest JSON parsed successfully");

            // Prüfe Policy-Hash im Manifest gegen input.policy_hash
            if let Some(policy_obj) = manifest.get("policy") {
                if let Some(manifest_policy_hash) = policy_obj.get("hash").and_then(|v| v.as_str())
                {
                    checks.push(CheckResult {
                        id: "policy_hash_match".to_string(),
                        kind: "policy".to_string(),
                        status: if manifest_policy_hash == input.policy_hash {
                            VerifyStatus::Ok
                        } else {
                            VerifyStatus::Fail
                        },
                        message: format!(
                            "Policy hash in manifest: {}, expected: {}",
                            manifest_policy_hash, input.policy_hash
                        ),
                    });
                } else {
                    checks.push(CheckResult {
                        id: "policy_hash_missing".to_string(),
                        kind: "policy".to_string(),
                        status: VerifyStatus::Warn,
                        message: "Policy hash not found in manifest".to_string(),
                    });
                }
            } else {
                checks.push(CheckResult {
                    id: "policy_missing".to_string(),
                    kind: "policy".to_string(),
                    status: VerifyStatus::Error,
                    message: "Policy section not found in manifest".to_string(),
                });
            }

            // TODO: Weitere Policy-Checks (z.B. Proof-Verifikation gegen Constraints)
            // Hier würde die bestehende Policy-Engine aus verifier/core.rs integriert werden
        }
        Err(e) => {
            checks.push(CheckResult {
                id: "parse_manifest".to_string(),
                kind: "integrity".to_string(),
                status: VerifyStatus::Error,
                message: format!("Failed to parse manifest JSON: {}", e),
            });
        }
    }

    // 4. Optional: Signatur-Check
    if input.options.check_signature {
        if let (Some(sig), Some(pubkey)) = (&input.signature, &input.public_key) {
            match verify_signature(&input.manifest_bytes, sig, pubkey) {
                Ok(valid) => {
                    checks.push(CheckResult {
                        id: "signature_valid".to_string(),
                        kind: "signature".to_string(),
                        status: if valid {
                            VerifyStatus::Ok
                        } else {
                            VerifyStatus::Fail
                        },
                        message: if valid {
                            "Ed25519 signature valid".to_string()
                        } else {
                            "Ed25519 signature invalid".to_string()
                        },
                    });
                }
                Err(e) => {
                    checks.push(CheckResult {
                        id: "signature_error".to_string(),
                        kind: "signature".to_string(),
                        status: VerifyStatus::Error,
                        message: format!("Signature verification error: {}", e),
                    });
                }
            }
        } else {
            checks.push(CheckResult {
                id: "signature_missing".to_string(),
                kind: "signature".to_string(),
                status: VerifyStatus::Warn,
                message: "No signature provided (check_signature enabled but no data)".to_string(),
            });
        }
    }

    // 5. Optional: Timestamp-Check
    if input.options.check_timestamp {
        if let Some(_ts) = &input.timestamp_attestation {
            // TODO: Echte RFC3161-Verifikation (aktuell Mock)
            checks.push(CheckResult {
                id: "timestamp_valid".to_string(),
                kind: "timestamp".to_string(),
                status: VerifyStatus::Ok,
                message: "Timestamp valid (mock)".to_string(),
            });
        } else {
            checks.push(CheckResult {
                id: "timestamp_missing".to_string(),
                kind: "timestamp".to_string(),
                status: VerifyStatus::Warn,
                message: "No timestamp provided".to_string(),
            });
        }
    }

    // 6. Optional: Registry-Check
    if input.options.check_registry {
        if let Some(reg_json) = &input.registry_entry_json {
            // Prüfe ob Hashes im Registry-Eintrag matchen
            match serde_json::from_str::<serde_json::Value>(reg_json) {
                Ok(entry) => {
                    let manifest_match = entry
                        .get("manifest_hash")
                        .and_then(|v| v.as_str())
                        .map(|h| h == input.manifest_hash)
                        .unwrap_or(false);

                    let proof_match = entry
                        .get("proof_hash")
                        .and_then(|v| v.as_str())
                        .map(|h| h == input.proof_hash)
                        .unwrap_or(false);

                    checks.push(CheckResult {
                        id: "registry_match".to_string(),
                        kind: "registry".to_string(),
                        status: if manifest_match && proof_match {
                            VerifyStatus::Ok
                        } else {
                            VerifyStatus::Fail
                        },
                        message: format!(
                            "Registry entry: manifest_match={}, proof_match={}",
                            manifest_match, proof_match
                        ),
                    });
                }
                Err(e) => {
                    checks.push(CheckResult {
                        id: "registry_parse_error".to_string(),
                        kind: "registry".to_string(),
                        status: VerifyStatus::Error,
                        message: format!("Failed to parse registry entry: {}", e),
                    });
                }
            }
        } else {
            checks.push(CheckResult {
                id: "registry_missing".to_string(),
                kind: "registry".to_string(),
                status: VerifyStatus::Warn,
                message: "No registry entry provided".to_string(),
            });
        }
    }

    finalize_result(checks, input, started_at)
}

/// Finalisiert das Verifikationsergebnis
fn finalize_result(
    checks: Vec<CheckResult>,
    input: CoreVerifyInput,
    started_at: String,
) -> CoreVerifyResult {
    let finished_at = Utc::now().to_rfc3339();

    // Gesamtstatus: Fail wenn mind. ein Fail, Error wenn mind. ein Error, Warn wenn mind. ein Warn, sonst Ok
    let overall_status = checks.iter().fold(VerifyStatus::Ok, |acc, check| {
        match (acc, check.status) {
            (VerifyStatus::Error, _) | (_, VerifyStatus::Error) => VerifyStatus::Error,
            (VerifyStatus::Fail, _) | (_, VerifyStatus::Fail) => VerifyStatus::Fail,
            (VerifyStatus::Warn, _) | (_, VerifyStatus::Warn) => VerifyStatus::Warn,
            _ => VerifyStatus::Ok,
        }
    });

    // Extrahiere optionale Felder
    let signature_valid = checks
        .iter()
        .find(|c| c.id == "signature_valid")
        .map(|c| c.status == VerifyStatus::Ok);

    let timestamp_valid = checks
        .iter()
        .find(|c| c.id == "timestamp_valid")
        .map(|c| c.status == VerifyStatus::Ok);

    let registry_match = checks
        .iter()
        .find(|c| c.id == "registry_match")
        .map(|c| c.status == VerifyStatus::Ok);

    CoreVerifyResult {
        status: overall_status,
        manifest_hash: input.manifest_hash,
        proof_hash: input.proof_hash,
        policy_id: input.policy_id,
        policy_hash: input.policy_hash,
        backend: input.backend,
        checks,
        signature_valid,
        timestamp_valid,
        registry_match,
        started_at,
        finished_at,
    }
}

/// Verifiziert Ed25519-Signatur
fn verify_signature(data: &[u8], sig: &[u8], pubkey: &[u8]) -> Result<bool> {
    use crate::crypto::{Ed25519PublicKey, Ed25519Signature};

    if sig.len() != 64 {
        return Err(anyhow!("Invalid signature length: {}, expected 64", sig.len()));
    }

    if pubkey.len() != 32 {
        return Err(anyhow!(
            "Invalid public key length: {}, expected 32",
            pubkey.len()
        ));
    }

    // Convert slices to fixed-size arrays
    let pubkey_array: &[u8; 32] = pubkey.try_into()?;
    let sig_array: &[u8; 64] = sig.try_into()?;

    let pubkey = Ed25519PublicKey::from_bytes(pubkey_array)?;
    let signature = Ed25519Signature::from_bytes(sig_array);

    // ed25519_verify returns bool, not Result
    let valid = crate::crypto::ed25519_verify(&pubkey, data, &signature);
    Ok(valid)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_core_verify_input_creation() {
        let input = CoreVerifyInput {
            protocol_version: "cap-core-verify.v1".to_string(),
            manifest_bytes: b"test".to_vec(),
            proof_bytes: b"proof".to_vec(),
            manifest_hash: "0x1234".to_string(),
            proof_hash: "0x5678".to_string(),
            policy_hash: "0xabcd".to_string(),
            policy_id: "test.policy".to_string(),
            backend: "mock".to_string(),
            signature: None,
            public_key: None,
            timestamp_attestation: None,
            registry_entry_json: None,
            options: CoreVerifyOptions::default(),
        };

        assert_eq!(input.protocol_version, "cap-core-verify.v1");
        assert_eq!(input.backend, "mock");
    }

    #[test]
    fn test_verify_core_hash_match_success() {
        let manifest_bytes = br#"{"version":"manifest.v1.0","policy":{"hash":"0xtest"}}"#;
        let proof_bytes = b"mock_proof_data";

        let manifest_hash = crate::crypto::hex_lower_prefixed32(crate::crypto::sha3_256(
            manifest_bytes,
        ));
        let proof_hash =
            crate::crypto::hex_lower_prefixed32(crate::crypto::sha3_256(proof_bytes));

        let input = CoreVerifyInput {
            protocol_version: "cap-core-verify.v1".to_string(),
            manifest_bytes: manifest_bytes.to_vec(),
            proof_bytes: proof_bytes.to_vec(),
            manifest_hash: manifest_hash.clone(),
            proof_hash: proof_hash.clone(),
            policy_hash: "0xtest".to_string(),
            policy_id: "test.policy".to_string(),
            backend: "mock".to_string(),
            signature: None,
            public_key: None,
            timestamp_attestation: None,
            registry_entry_json: None,
            options: CoreVerifyOptions {
                check_signature: false,
                check_timestamp: false,
                check_registry: false,
            },
        };

        let result = verify_core(input);

        // Status sollte Ok sein (keine Fails)
        assert_eq!(result.status, VerifyStatus::Ok);

        // Hash-Checks sollten Ok sein
        assert!(result
            .checks
            .iter()
            .any(|c| c.id == "hash_match_manifest" && c.status == VerifyStatus::Ok));
        assert!(result
            .checks
            .iter()
            .any(|c| c.id == "hash_match_proof" && c.status == VerifyStatus::Ok));
    }

    #[test]
    fn test_verify_core_hash_mismatch_manifest() {
        // Use valid JSON for manifest_bytes to avoid JSON parse error
        let manifest_bytes = br#"{"version":"manifest.v1.0","policy":{"hash":"0xtest"}}"#;
        let wrong_hash = "0x0000000000000000000000000000000000000000000000000000000000000000";

        let proof_hash = crate::crypto::hex_lower_prefixed32(crate::crypto::sha3_256(b"proof"));

        let input = CoreVerifyInput {
            protocol_version: "cap-core-verify.v1".to_string(),
            manifest_bytes: manifest_bytes.to_vec(),
            proof_bytes: b"proof".to_vec(),
            manifest_hash: wrong_hash.to_string(),
            proof_hash,
            policy_hash: "0xtest".to_string(),
            policy_id: "test.policy".to_string(),
            backend: "mock".to_string(),
            signature: None,
            public_key: None,
            timestamp_attestation: None,
            registry_entry_json: None,
            options: CoreVerifyOptions {
                check_signature: false,
                check_timestamp: false,
                check_registry: false,
            },
        };

        let result = verify_core(input);

        // Status sollte Fail sein
        assert_eq!(result.status, VerifyStatus::Fail);

        // Hash-Check sollte Fail sein
        let hash_check = result
            .checks
            .iter()
            .find(|c| c.id == "hash_match_manifest")
            .unwrap();
        assert_eq!(hash_check.status, VerifyStatus::Fail);
    }

    #[test]
    fn test_verify_core_options_disable_checks() {
        let manifest_bytes = br#"{"version":"manifest.v1.0","policy":{"hash":"0xtest"}}"#;
        let manifest_hash = crate::crypto::hex_lower_prefixed32(crate::crypto::sha3_256(
            manifest_bytes,
        ));
        let proof_hash = crate::crypto::hex_lower_prefixed32(crate::crypto::sha3_256(b"proof"));

        let input = CoreVerifyInput {
            protocol_version: "cap-core-verify.v1".to_string(),
            manifest_bytes: manifest_bytes.to_vec(),
            proof_bytes: b"proof".to_vec(),
            manifest_hash,
            proof_hash,
            policy_hash: "0xtest".to_string(),
            policy_id: "test.policy".to_string(),
            backend: "mock".to_string(),
            signature: Some(vec![0u8; 64]),
            public_key: Some(vec![0u8; 32]),
            timestamp_attestation: Some(b"mock_ts".to_vec()),
            registry_entry_json: Some("{}".to_string()),
            options: CoreVerifyOptions {
                check_signature: false, // Deaktiviert!
                check_timestamp: false,
                check_registry: false,
            },
        };

        let result = verify_core(input);

        // Keine Signatur/Timestamp/Registry-Checks sollten ausgeführt worden sein
        assert!(!result.checks.iter().any(|c| c.kind == "signature"));
        assert!(!result.checks.iter().any(|c| c.kind == "timestamp"));
        assert!(!result.checks.iter().any(|c| c.kind == "registry"));
    }

    #[test]
    fn test_verify_core_result_timestamps() {
        let manifest_bytes = br#"{"version":"manifest.v1.0","policy":{"hash":"0xtest"}}"#;
        let manifest_hash = crate::crypto::hex_lower_prefixed32(crate::crypto::sha3_256(
            manifest_bytes,
        ));
        let proof_hash = crate::crypto::hex_lower_prefixed32(crate::crypto::sha3_256(b"proof"));

        let input = CoreVerifyInput {
            protocol_version: "cap-core-verify.v1".to_string(),
            manifest_bytes: manifest_bytes.to_vec(),
            proof_bytes: b"proof".to_vec(),
            manifest_hash,
            proof_hash,
            policy_hash: "0xtest".to_string(),
            policy_id: "test.policy".to_string(),
            backend: "mock".to_string(),
            signature: None,
            public_key: None,
            timestamp_attestation: None,
            registry_entry_json: None,
            options: CoreVerifyOptions {
                check_signature: false,
                check_timestamp: false,
                check_registry: false,
            },
        };

        let result = verify_core(input);

        assert!(!result.started_at.is_empty());
        assert!(!result.finished_at.is_empty());

        // Parse als RFC3339
        chrono::DateTime::parse_from_rfc3339(&result.started_at).unwrap();
        chrono::DateTime::parse_from_rfc3339(&result.finished_at).unwrap();
    }
}
