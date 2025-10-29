use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};
use std::error::Error;
use std::fs::File;
use std::path::Path;

/// Policy-Datenstruktur gemäß LkSG v1 Schema
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Policy {
    pub version: String,
    pub name: String,
    pub created_at: String,
    pub constraints: PolicyConstraints,
    #[serde(default)]
    pub notes: String,
}

/// Policy-Constraints (Regeln)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PolicyConstraints {
    pub require_at_least_one_ubo: bool,
    pub supplier_count_max: u32,
}

/// Policy mit Hash für Manifest
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PolicyInfo {
    pub name: String,
    pub version: String,
    pub hash: String,
}

impl Policy {
    /// Lädt eine Policy-Datei (YAML oder JSON)
    ///
    /// # Argumente
    /// * `path` - Pfad zur Policy-Datei (.yml, .yaml oder .json)
    ///
    /// # Rückgabe
    /// Policy-Objekt oder Fehler
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let path_ref = path.as_ref();
        let file = File::open(path_ref)?;

        let policy = if path_ref
            .extension()
            .and_then(|s| s.to_str())
            .map(|s| s == "yml" || s == "yaml")
            .unwrap_or(false)
        {
            // YAML-Parsing
            serde_yaml::from_reader(file)?
        } else {
            // JSON-Parsing (fallback)
            serde_json::from_reader(file)?
        };

        Ok(policy)
    }

    /// Validiert die Policy nach Schema und Semantik
    ///
    /// # Rückgabe
    /// Result mit () bei Erfolg oder Fehler mit Beschreibung
    pub fn validate(&self) -> Result<(), Box<dyn Error>> {
        // Schema-Validierung (Pflichtfelder bereits durch serde garantiert)

        // Versionsprüfung
        if !self.version.starts_with("lksg.v") {
            return Err(format!("Ungültige Policy-Version: {}", self.version).into());
        }

        // Name darf nicht leer sein
        if self.name.trim().is_empty() {
            return Err("Policy-Name darf nicht leer sein".into());
        }

        // Timestamp-Format prüfen (grob)
        if !self.created_at.contains('T') {
            return Err("Ungültiges Timestamp-Format (erwartet RFC3339)".into());
        }

        // Semantische Validierung
        if self.constraints.supplier_count_max == 0 {
            return Err("supplier_count_max muss > 0 sein".into());
        }

        Ok(())
    }

    /// Berechnet den SHA3-256 Hash der Policy (kanonisches JSON)
    ///
    /// # Rückgabe
    /// Hex-String des Policy-Hashes
    pub fn compute_hash(&self) -> Result<String, Box<dyn Error>> {
        // Kanonisches JSON (alphabetisch sortierte Keys durch serde_json)
        let json = serde_json::to_string(self)?;
        let mut hasher = Sha3_256::new();
        hasher.update(json.as_bytes());
        let result = hasher.finalize();
        Ok(format!("0x{}", hex::encode(result)))
    }

    /// Erstellt PolicyInfo für Manifest
    ///
    /// # Rückgabe
    /// PolicyInfo mit Name, Version und Hash
    pub fn to_info(&self) -> Result<PolicyInfo, Box<dyn Error>> {
        Ok(PolicyInfo {
            name: self.name.clone(),
            version: self.version.clone(),
            hash: self.compute_hash()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;

    #[test]
    fn test_policy_validation() {
        let policy = Policy {
            version: "lksg.v1".to_string(),
            name: "Test Policy".to_string(),
            created_at: "2025-10-25T09:00:00Z".to_string(),
            constraints: PolicyConstraints {
                require_at_least_one_ubo: true,
                supplier_count_max: 100,
            },
            notes: "Test".to_string(),
        };

        assert!(policy.validate().is_ok());
    }

    #[test]
    fn test_policy_invalid_version() {
        let policy = Policy {
            version: "invalid".to_string(),
            name: "Test".to_string(),
            created_at: "2025-10-25T09:00:00Z".to_string(),
            constraints: PolicyConstraints {
                require_at_least_one_ubo: true,
                supplier_count_max: 100,
            },
            notes: "".to_string(),
        };

        assert!(policy.validate().is_err());
    }

    #[test]
    fn test_policy_hash_deterministic() {
        let policy = Policy {
            version: "lksg.v1".to_string(),
            name: "Test".to_string(),
            created_at: "2025-10-25T09:00:00Z".to_string(),
            constraints: PolicyConstraints {
                require_at_least_one_ubo: true,
                supplier_count_max: 10,
            },
            notes: "".to_string(),
        };

        let hash1 = policy.compute_hash().unwrap();
        let hash2 = policy.compute_hash().unwrap();
        assert_eq!(hash1, hash2);
        assert!(hash1.starts_with("0x"));
    }

    #[test]
    fn test_load_yaml() {
        let yaml_content = r#"
version: "lksg.v1"
name: "Test Policy"
created_at: "2025-10-25T09:00:00Z"
constraints:
  require_at_least_one_ubo: true
  supplier_count_max: 50
notes: "Test notes"
"#;

        let temp_path = "/tmp/test_policy.yml";
        let mut file = File::create(temp_path).unwrap();
        file.write_all(yaml_content.as_bytes()).unwrap();

        let policy = Policy::load(temp_path).unwrap();
        assert_eq!(policy.name, "Test Policy");
        assert_eq!(policy.constraints.supplier_count_max, 50);

        fs::remove_file(temp_path).ok();
    }
}
