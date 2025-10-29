use csv::ReaderBuilder;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::path::Path;

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
}
