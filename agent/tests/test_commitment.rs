/// Integration Tests für commitment.rs
///
/// Diese Tests wurden aus inline test modules extrahiert um Tarpaulin Coverage-Tracking zu ermöglichen.
/// Tarpaulin hat eine bekannte Limitation mit #[cfg(test)] inline modules.

use cap_agent::commitment::{compute_company_root, compute_supplier_root, compute_ubo_root};
use cap_agent::io::{Supplier, Ubo};

#[test]
fn test_hash_record_creates_valid_hash() {
    let supplier = Supplier {
        name: "Test Supplier".to_string(),
        jurisdiction: "DE".to_string(),
        tier: 1,
    };

    // Hash sollte mit 0x beginnen und ausreichend lang sein
    let hash = compute_supplier_root(&[supplier]).unwrap();
    assert!(hash.starts_with("0x"), "Hash should start with 0x");
    assert!(hash.len() > 10, "Hash should be longer than 10 characters");
}

#[test]
fn test_merkle_root_is_deterministic() {
    let supplier1 = Supplier {
        name: "Supplier A".to_string(),
        jurisdiction: "DE".to_string(),
        tier: 1,
    };
    let supplier2 = Supplier {
        name: "Supplier B".to_string(),
        jurisdiction: "FR".to_string(),
        tier: 2,
    };

    let suppliers = vec![supplier1.clone(), supplier2.clone()];

    // Gleiche Input → Gleicher Output
    let root1 = compute_supplier_root(&suppliers).unwrap();
    let root2 = compute_supplier_root(&suppliers).unwrap();

    assert_eq!(root1, root2, "Merkle root should be deterministic");
}

#[test]
fn test_company_root_combines_roots() {
    let supplier_root = "0xabc123def456";
    let ubo_root = "0x789ghi012jkl";

    let company_root = compute_company_root(supplier_root, ubo_root);

    assert!(company_root.starts_with("0x"), "Company root should start with 0x");
    assert_ne!(company_root, supplier_root, "Company root should differ from supplier root");
    assert_ne!(company_root, ubo_root, "Company root should differ from UBO root");
}

#[test]
fn test_ubo_root_computation() {
    let ubo = Ubo {
        name: "Test UBO".to_string(),
        birthdate: "1980-01-01".to_string(),
        citizenship: "DE".to_string(),
    };

    let root = compute_ubo_root(&[ubo]).unwrap();

    assert!(root.starts_with("0x"), "UBO root should start with 0x");
    assert!(root.len() > 10, "UBO root should be longer than 10 characters");
}

#[test]
fn test_empty_supplier_list() {
    let suppliers: Vec<Supplier> = vec![];
    let root = compute_supplier_root(&suppliers).unwrap();

    // Empty list sollte trotzdem einen validen Root erzeugen
    assert!(root.starts_with("0x"), "Empty supplier root should start with 0x");
}

#[test]
fn test_empty_ubo_list() {
    let ubos: Vec<Ubo> = vec![];
    let root = compute_ubo_root(&ubos).unwrap();

    // Empty list sollte trotzdem einen validen Root erzeugen
    assert!(root.starts_with("0x"), "Empty UBO root should start with 0x");
}

#[test]
fn test_different_suppliers_produce_different_roots() {
    let supplier1 = Supplier {
        name: "Supplier 1".to_string(),
        jurisdiction: "DE".to_string(),
        tier: 1,
    };

    let supplier2 = Supplier {
        name: "Supplier 2".to_string(),
        jurisdiction: "FR".to_string(),
        tier: 2,
    };

    let root1 = compute_supplier_root(&[supplier1]).unwrap();
    let root2 = compute_supplier_root(&[supplier2]).unwrap();

    assert_ne!(root1, root2, "Different suppliers should produce different roots");
}
