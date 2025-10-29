use blake3::Hasher;
use csv::ReaderBuilder;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::path::Path;

/// Sanctions-Eintrag (Person auf Sanktionsliste)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SanctionsEntry {
    pub name: String,
    pub birthdate: String,
    pub nationality: String,
}

/// Sanctions Root Info
#[derive(Debug, Serialize, Deserialize)]
pub struct SanctionsRootInfo {
    pub root: String,
    pub count: usize,
    pub source: String,
    pub generated_at: String,
    pub algorithm: String,
}

impl SanctionsEntry {
    /// Hasht einen Sanctions-Eintrag mit BLAKE3
    ///
    /// # Rückgabe
    /// Hex-String des BLAKE3-Hashes
    pub fn hash(&self) -> String {
        let mut hasher = Hasher::new();
        hasher.update(self.name.as_bytes());
        hasher.update(b"|");
        hasher.update(self.birthdate.as_bytes());
        hasher.update(b"|");
        hasher.update(self.nationality.as_bytes());

        let hash = hasher.finalize();
        format!("0x{}", hex::encode(hash.as_bytes()))
    }
}

/// Liest eine Sanctions CSV und berechnet den Merkle Root
///
/// # Argumente
/// * `csv_path` - Pfad zur CSV-Datei
///
/// # Rückgabe
/// Tuple (root_hex, entries)
pub fn compute_sanctions_root<P: AsRef<Path>>(
    csv_path: P,
) -> Result<(String, Vec<SanctionsEntry>), Box<dyn Error>> {
    let file = File::open(&csv_path)?;
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(file);

    let mut entries = Vec::new();
    let mut hashes = Vec::new();

    for result in reader.deserialize() {
        let entry: SanctionsEntry = result?;
        let hash = entry.hash();
        hashes.push(hash);
        entries.push(entry);
    }

    // Berechne Merkle Root (simplified: hash all hashes together)
    let root = compute_merkle_root(&hashes);

    Ok((root, entries))
}

/// Berechnet einen einfachen Merkle Root aus einer Liste von Hashes
///
/// # Argumente
/// * `hashes` - Liste von Hex-Hashes
///
/// # Rückgabe
/// Root-Hash als Hex-String
fn compute_merkle_root(hashes: &[String]) -> String {
    let mut hasher = Hasher::new();

    for hash in hashes {
        hasher.update(hash.as_bytes());
    }

    let root = hasher.finalize();
    format!("0x{}", hex::encode(root.as_bytes()))
}

/// Speichert Sanctions Root Info in eine Datei
///
/// # Argumente
/// * `info` - Die Root-Info
/// * `path` - Zielpfad
pub fn save_sanctions_root_info<P: AsRef<Path>>(
    info: &SanctionsRootInfo,
    path: P,
) -> Result<(), Box<dyn Error>> {
    let content = format!(
        "root: \"{}\"\ncount: {}\nsource: \"{}\"\ngenerated_at: \"{}\"\nalgorithm: \"{}\"",
        info.root, info.count, info.source, info.generated_at, info.algorithm
    );
    std::fs::write(path, content)?;
    Ok(())
}

/// Lädt Sanctions Root aus einer Info-Datei
///
/// # Argumente
/// * `path` - Pfad zur Info-Datei
///
/// # Rückgabe
/// Root-Hash als String
#[allow(dead_code)]
pub fn load_sanctions_root<P: AsRef<Path>>(path: P) -> Result<String, Box<dyn Error>> {
    let content = std::fs::read_to_string(path)?;

    for line in content.lines() {
        if line.starts_with("root:") {
            let root = line
                .split('"')
                .nth(1)
                .ok_or("Ungültiges Root-Format")?
                .to_string();
            return Ok(root);
        }
    }

    Err("Root nicht gefunden in Datei".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn sanctions_root_deterministic() {
        let csv_content = "name,birthdate,nationality\nAli Hassan,1984-01-14,IR\nMaria Petrova,1973-05-22,RU\n";

        let temp_csv = "/tmp/test_sanctions.csv";
        let mut file = File::create(temp_csv).unwrap();
        file.write_all(csv_content.as_bytes()).unwrap();

        let (root1, _) = compute_sanctions_root(temp_csv).unwrap();
        let (root2, _) = compute_sanctions_root(temp_csv).unwrap();

        assert_eq!(root1, root2);
        assert!(root1.starts_with("0x"));

        std::fs::remove_file(temp_csv).ok();
    }

    #[test]
    fn sanctions_entry_hash() {
        let entry = SanctionsEntry {
            name: "John Doe".to_string(),
            birthdate: "1990-01-01".to_string(),
            nationality: "US".to_string(),
        };

        let hash1 = entry.hash();
        let hash2 = entry.hash();

        assert_eq!(hash1, hash2);
        assert!(hash1.starts_with("0x"));
        assert_eq!(hash1.len(), 66); // "0x" + 64 hex chars
    }

    #[test]
    fn sanctions_root_info_roundtrip() {
        let info = SanctionsRootInfo {
            root: "0xabc123".to_string(),
            count: 42,
            source: "test.csv".to_string(),
            generated_at: "2025-11-01T10:00:00Z".to_string(),
            algorithm: "BLAKE3".to_string(),
        };

        let temp_file = "/tmp/test_sanctions.root";
        save_sanctions_root_info(&info, temp_file).unwrap();
        let loaded_root = load_sanctions_root(temp_file).unwrap();

        assert_eq!(loaded_root, "0xabc123");

        std::fs::remove_file(temp_file).ok();
    }
}
