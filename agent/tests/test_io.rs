/// Integration Tests für io.rs
///
/// Diese Tests wurden aus inline test modules extrahiert um Tarpaulin Coverage-Tracking zu ermöglichen.
/// Tarpaulin hat eine bekannte Limitation mit #[cfg(test)] inline modules.

use cap_agent::io::{read_suppliers_csv, read_ubos_csv, Supplier, Ubo};
use std::fs;
use std::io::Write;

#[test]
fn test_supplier_struct_creation() {
    let supplier = Supplier {
        name: "Test GmbH".to_string(),
        jurisdiction: "DE".to_string(),
        tier: 1,
    };
    assert_eq!(supplier.name, "Test GmbH");
    assert_eq!(supplier.jurisdiction, "DE");
    assert_eq!(supplier.tier, 1);
}

#[test]
fn test_ubo_struct_creation() {
    let ubo = Ubo {
        name: "Test Person".to_string(),
        birthdate: "1980-01-01".to_string(),
        citizenship: "DE".to_string(),
    };
    assert_eq!(ubo.name, "Test Person");
    assert_eq!(ubo.birthdate, "1980-01-01");
    assert_eq!(ubo.citizenship, "DE");
}

#[test]
fn test_read_suppliers_csv_valid() {
    // Create temporary CSV file
    let temp_dir = std::env::temp_dir();
    let csv_path = temp_dir.join("test_suppliers.csv");

    let csv_content = "name,jurisdiction,tier\nSupplier A,DE,1\nSupplier B,FR,2\n";
    let mut file = fs::File::create(&csv_path).unwrap();
    file.write_all(csv_content.as_bytes()).unwrap();

    // Read CSV
    let suppliers = read_suppliers_csv(&csv_path).unwrap();

    assert_eq!(suppliers.len(), 2);
    assert_eq!(suppliers[0].name, "Supplier A");
    assert_eq!(suppliers[0].jurisdiction, "DE");
    assert_eq!(suppliers[0].tier, 1);
    assert_eq!(suppliers[1].name, "Supplier B");
    assert_eq!(suppliers[1].jurisdiction, "FR");
    assert_eq!(suppliers[1].tier, 2);

    // Cleanup
    fs::remove_file(&csv_path).ok();
}

#[test]
fn test_read_ubos_csv_valid() {
    // Create temporary CSV file
    let temp_dir = std::env::temp_dir();
    let csv_path = temp_dir.join("test_ubos.csv");

    let csv_content =
        "name,birthdate,citizenship\nJohn Doe,1980-01-01,DE\nJane Smith,1985-05-15,FR\n";
    let mut file = fs::File::create(&csv_path).unwrap();
    file.write_all(csv_content.as_bytes()).unwrap();

    // Read CSV
    let ubos = read_ubos_csv(&csv_path).unwrap();

    assert_eq!(ubos.len(), 2);
    assert_eq!(ubos[0].name, "John Doe");
    assert_eq!(ubos[0].birthdate, "1980-01-01");
    assert_eq!(ubos[0].citizenship, "DE");
    assert_eq!(ubos[1].name, "Jane Smith");
    assert_eq!(ubos[1].birthdate, "1985-05-15");
    assert_eq!(ubos[1].citizenship, "FR");

    // Cleanup
    fs::remove_file(&csv_path).ok();
}

#[test]
fn test_read_suppliers_csv_empty() {
    // Create temporary empty CSV file (with headers only)
    let temp_dir = std::env::temp_dir();
    let csv_path = temp_dir.join("test_suppliers_empty.csv");

    let csv_content = "name,jurisdiction,tier\n";
    let mut file = fs::File::create(&csv_path).unwrap();
    file.write_all(csv_content.as_bytes()).unwrap();

    // Read CSV
    let suppliers = read_suppliers_csv(&csv_path).unwrap();

    assert_eq!(suppliers.len(), 0, "Empty CSV should return empty vector");

    // Cleanup
    fs::remove_file(&csv_path).ok();
}

#[test]
fn test_read_ubos_csv_empty() {
    // Create temporary empty CSV file (with headers only)
    let temp_dir = std::env::temp_dir();
    let csv_path = temp_dir.join("test_ubos_empty.csv");

    let csv_content = "name,birthdate,citizenship\n";
    let mut file = fs::File::create(&csv_path).unwrap();
    file.write_all(csv_content.as_bytes()).unwrap();

    // Read CSV
    let ubos = read_ubos_csv(&csv_path).unwrap();

    assert_eq!(ubos.len(), 0, "Empty CSV should return empty vector");

    // Cleanup
    fs::remove_file(&csv_path).ok();
}

#[test]
fn test_read_suppliers_csv_file_not_found() {
    let result = read_suppliers_csv("/nonexistent/path/suppliers.csv");
    assert!(result.is_err(), "Reading nonexistent file should fail");
}

#[test]
fn test_read_ubos_csv_file_not_found() {
    let result = read_ubos_csv("/nonexistent/path/ubos.csv");
    assert!(result.is_err(), "Reading nonexistent file should fail");
}

#[test]
fn test_read_suppliers_csv_malformed() {
    // Create temporary malformed CSV file (missing tier field)
    let temp_dir = std::env::temp_dir();
    let csv_path = temp_dir.join("test_suppliers_malformed.csv");

    let csv_content = "name,jurisdiction,tier\nSupplier A,DE,invalid_tier\n";
    let mut file = fs::File::create(&csv_path).unwrap();
    file.write_all(csv_content.as_bytes()).unwrap();

    // Read CSV - should fail due to invalid tier (not a number)
    let result = read_suppliers_csv(&csv_path);
    assert!(result.is_err(), "Malformed CSV should fail to parse");

    // Cleanup
    fs::remove_file(&csv_path).ok();
}

#[test]
fn test_supplier_clone() {
    let supplier1 = Supplier {
        name: "Original".to_string(),
        jurisdiction: "DE".to_string(),
        tier: 1,
    };

    let supplier2 = supplier1.clone();

    assert_eq!(supplier2.name, "Original");
    assert_eq!(supplier2.jurisdiction, "DE");
    assert_eq!(supplier2.tier, 1);
}

#[test]
fn test_ubo_clone() {
    let ubo1 = Ubo {
        name: "Original Person".to_string(),
        birthdate: "1980-01-01".to_string(),
        citizenship: "DE".to_string(),
    };

    let ubo2 = ubo1.clone();

    assert_eq!(ubo2.name, "Original Person");
    assert_eq!(ubo2.birthdate, "1980-01-01");
    assert_eq!(ubo2.citizenship, "DE");
}
