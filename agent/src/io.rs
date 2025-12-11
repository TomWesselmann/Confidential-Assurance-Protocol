use csv::ReaderBuilder;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::path::Path;

// ============================================================================
// JsonPersistent Trait - Gemeinsames Interface für JSON-Dateipersistenz
// ============================================================================

/// Trait für Structs die als JSON-Dateien geladen/gespeichert werden können
///
/// Bietet Default-Implementierungen für `load()` und `save()` Methoden.
/// Structs die diesen Trait implementieren müssen `Serialize + DeserializeOwned` sein.
///
/// # Beispiel
/// ```ignore
/// use crate::io::JsonPersistent;
///
/// #[derive(Serialize, Deserialize)]
/// struct MyConfig { name: String }
///
/// impl JsonPersistent for MyConfig {}
///
/// // Dann kann man schreiben:
/// let config = MyConfig::load("config.json")?;
/// config.save("config.json")?;
/// ```
pub trait JsonPersistent: Serialize + DeserializeOwned + Sized {
    /// Lädt das Objekt aus einer JSON-Datei
    ///
    /// # Argumente
    /// * `path` - Pfad zur JSON-Datei
    ///
    /// # Rückgabe
    /// Das deserialisierte Objekt oder ein Fehler
    fn load<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let file = File::open(path)?;
        let obj: Self = serde_json::from_reader(file)?;
        Ok(obj)
    }

    /// Speichert das Objekt als JSON-Datei (pretty-printed)
    ///
    /// # Argumente
    /// * `path` - Zielpfad für die JSON-Datei
    ///
    /// # Rückgabe
    /// Ok(()) bei Erfolg oder ein Fehler
    fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn Error>> {
        let json = serde_json::to_string_pretty(self)?;
        let mut file = File::create(path)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }
}

// ============================================================================
// CSV I/O Strukturen und Funktionen
// ============================================================================

/// Supplier-Datenstruktur für CSV-Import
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Supplier {
    pub name: String,
    pub jurisdiction: String,
    pub tier: u32,
}

/// UBO (Ultimate Beneficial Owner) Datenstruktur für CSV-Import
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Ubo {
    pub name: String,
    pub birthdate: String,
    pub citizenship: String,
}

/// Liest Supplier-Daten aus einer CSV-Datei
///
/// # Argumente
/// * `path` - Pfad zur CSV-Datei
///
/// # Rückgabe
/// Vector mit Supplier-Objekten oder Fehler
pub fn read_suppliers_csv<P: AsRef<Path>>(path: P) -> Result<Vec<Supplier>, Box<dyn Error>> {
    let file = File::open(path)?;
    let mut rdr = ReaderBuilder::new().has_headers(true).from_reader(file);

    let mut suppliers = Vec::new();
    for result in rdr.deserialize() {
        let supplier: Supplier = result?;
        suppliers.push(supplier);
    }

    Ok(suppliers)
}

/// Liest UBO-Daten aus einer CSV-Datei
///
/// # Argumente
/// * `path` - Pfad zur CSV-Datei
///
/// # Rückgabe
/// Vector mit UBO-Objekten oder Fehler
pub fn read_ubos_csv<P: AsRef<Path>>(path: P) -> Result<Vec<Ubo>, Box<dyn Error>> {
    let file = File::open(path)?;
    let mut rdr = ReaderBuilder::new().has_headers(true).from_reader(file);

    let mut ubos = Vec::new();
    for result in rdr.deserialize() {
        let ubo: Ubo = result?;
        ubos.push(ubo);
    }

    Ok(ubos)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_supplier_parsing() {
        let supplier = Supplier {
            name: "Test GmbH".to_string(),
            jurisdiction: "DE".to_string(),
            tier: 1,
        };
        assert_eq!(supplier.name, "Test GmbH");
        assert_eq!(supplier.tier, 1);
    }

    #[test]
    fn test_ubo_parsing() {
        let ubo = Ubo {
            name: "Test Person".to_string(),
            birthdate: "1980-01-01".to_string(),
            citizenship: "DE".to_string(),
        };
        assert_eq!(ubo.name, "Test Person");
        assert_eq!(ubo.citizenship, "DE");
    }

    // ========================================================================
    // JsonPersistent Trait Tests
    // ========================================================================

    /// Test-Struct für JsonPersistent
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestConfig {
        name: String,
        value: i32,
    }

    impl JsonPersistent for TestConfig {}

    #[test]
    fn test_json_persistent_save_load_roundtrip() {
        let original = TestConfig {
            name: "test".to_string(),
            value: 42,
        };

        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();

        // Save
        original.save(path).unwrap();

        // Load
        let loaded = TestConfig::load(path).unwrap();

        assert_eq!(original, loaded);
    }

    #[test]
    fn test_json_persistent_save_creates_pretty_json() {
        let config = TestConfig {
            name: "test".to_string(),
            value: 123,
        };

        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();

        config.save(path).unwrap();

        let content = std::fs::read_to_string(path).unwrap();

        // Pretty-printed JSON hat Newlines
        assert!(content.contains('\n'));
        assert!(content.contains("\"name\""));
        assert!(content.contains("\"value\""));
    }

    #[test]
    fn test_json_persistent_load_nonexistent_fails() {
        let result = TestConfig::load("/nonexistent/path/config.json");
        assert!(result.is_err());
    }

    #[test]
    fn test_json_persistent_load_invalid_json_fails() {
        let temp_file = NamedTempFile::new().unwrap();
        std::fs::write(temp_file.path(), "{ invalid json }").unwrap();

        let result = TestConfig::load(temp_file.path());
        assert!(result.is_err());
    }
}
