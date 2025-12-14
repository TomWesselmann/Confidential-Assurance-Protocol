//! CLI-Handler für Registry- und Lists-Kommandos
//!
//! Extrahiert aus main.rs für bessere Wartbarkeit.
//! Enthält: run_lists_sanctions_root, run_lists_jurisdictions_root,
//!          run_registry_add, run_registry_list, run_registry_verify,
//!          run_registry_migrate, run_registry_inspect, run_registry_backfill_kid

use super::output;
use crate::audit::AuditLog;
use crate::registry;
use registry::RegistryBackend;
use serde_json::json;
use std::error::Error;
use std::fs;
use std::path::Path;

// ============================================================================
// Helper-Funktionen
// ============================================================================

/// Parst Backend-String zu RegistryBackend (lenient: default = Json)
fn parse_backend(backend_str: &str) -> RegistryBackend {
    match backend_str {
        "sqlite" => RegistryBackend::Sqlite,
        _ => RegistryBackend::Json,
    }
}

/// Parst Backend-String zu RegistryBackend (strict: Fehler bei unbekannt)
fn parse_backend_strict(backend_str: &str) -> Result<RegistryBackend, Box<dyn Error>> {
    match backend_str {
        "sqlite" => Ok(RegistryBackend::Sqlite),
        "json" => Ok(RegistryBackend::Json),
        _ => Err(format!("Unbekanntes Backend: {}", backend_str).into()),
    }
}

/// Ermittelt Registry-Dateipfad basierend auf Backend
fn get_registry_file(registry_path: Option<String>, backend: RegistryBackend) -> String {
    registry_path.unwrap_or_else(|| match backend {
        RegistryBackend::Json => "build/registry.json".to_string(),
        RegistryBackend::Sqlite => "build/registry.sqlite".to_string(),
    })
}

/// Erstellt einen neuen RegistryEntry mit Default-Werten
fn create_registry_entry(
    id: String,
    manifest_hash: String,
    proof_hash: String,
    timestamp_path: Option<String>,
) -> registry::RegistryEntry {
    registry::RegistryEntry {
        id,
        manifest_hash,
        proof_hash,
        timestamp_file: timestamp_path,
        registered_at: chrono::Utc::now().to_rfc3339(),
        signature: None,
        public_key: None,
        blob_manifest: None,
        blob_proof: None,
        blob_wasm: None,
        blob_abi: None,
        selfverify_status: None,
        selfverify_at: None,
        verifier_name: None,
        verifier_version: None,
        kid: None,
        signature_scheme: None,
    }
}

/// Signiert Entry mit optionalem Schlüssel und validiert Key-Status
fn sign_and_validate_entry(
    entry: &mut registry::RegistryEntry,
    signing_key_path: Option<String>,
    validate_key: bool,
    keys_dir: &str,
) -> Result<(), Box<dyn Error>> {
    let Some(key_path) = signing_key_path else {
        return Ok(());
    };

    let key_file = if key_path.is_empty() {
        "keys/company.ed25519"
    } else {
        &key_path
    };

    output::detail("Signing-Key", key_file);

    // Load signing key
    let key_bytes = fs::read(key_file)
        .map_err(|e| format!("Failed to read signing key from {}: {}", key_file, e))?;

    if key_bytes.len() != 32 {
        return Err(format!(
            "Invalid signing key length (expected 32 bytes, got {})",
            key_bytes.len()
        )
        .into());
    }

    let key_array: [u8; 32] = key_bytes
        .try_into()
        .map_err(|_| "Failed to convert key bytes to array")?;
    let signing_key = ed25519_dalek::SigningKey::from_bytes(&key_array);

    // Sign entry
    registry::sign_entry(entry, &signing_key)?;
    output::indent("✓ Entry signed with Ed25519");

    // Validate key status if requested
    if validate_key {
        if let Some(ref kid) = entry.kid {
            output::indent("Validating key status...");
            registry::validate_key_status(kid, keys_dir)?;
            output::indent("✓ Key status validated (active)");
        } else {
            return Err("Cannot validate key status: KID not set".into());
        }
    }

    Ok(())
}

// ============================================================================
// Öffentliche CLI-Funktionen
// ============================================================================

// Note: run_lists_sanctions_root and run_lists_jurisdictions_root removed
// in minimal local agent. These features require the lists module.

/// Registry add - Fügt einen Proof zur Registry hinzu
#[allow(clippy::too_many_arguments)]
pub fn run_registry_add(
    manifest_path: &str,
    proof_path: &str,
    timestamp_path: Option<String>,
    registry_path: Option<String>,
    backend_str: &str,
    signing_key_path: Option<String>,
    validate_key: bool,
    keys_dir: &str,
) -> Result<(), Box<dyn Error>> {
    output::writing("Füge Proof zur Registry hinzu...");

    // Parse backend and get registry file
    let backend = parse_backend(backend_str);
    let registry_file = get_registry_file(registry_path, backend);
    output::detail("Backend", backend_str);

    // Open store
    let store = registry::open_store(backend, Path::new(&registry_file))?;

    // Berechne Hashes
    let manifest_hash = registry::compute_file_hash(manifest_path)?;
    let proof_hash = registry::compute_file_hash(proof_path)?;
    output::detail("Manifest-Hash", &manifest_hash);
    output::detail("Proof-Hash", &proof_hash);

    // Load current registry to get next ID
    let current_reg = store.load()?;
    let id = format!("proof_{:03}", current_reg.entries.len() + 1);

    // Create entry
    let mut entry = create_registry_entry(
        id.clone(),
        manifest_hash.clone(),
        proof_hash.clone(),
        timestamp_path.clone(),
    );

    // Sign entry if signing key provided
    sign_and_validate_entry(&mut entry, signing_key_path, validate_key, keys_dir)?;

    // Add entry
    store.add_entry(entry)?;

    // Log Audit-Event
    let mut audit = AuditLog::new("build/agent.audit.jsonl")?;
    audit.log_event(
        "registry_entry_added",
        json!({
            "id": id,
            "manifest_hash": manifest_hash,
            "proof_hash": proof_hash,
            "timestamp_file": timestamp_path,
            "registry_file": registry_file,
            "backend": backend_str
        }),
    )?;

    let total = store.list()?.len();
    output::success("Proof zur Registry hinzugefügt:");
    output::detail("ID", &id);
    output::detail("Registry", &registry_file);
    output::detail_fmt("Einträge total", total);

    Ok(())
}

/// Registry list - Listet alle Registry-Einträge auf
pub fn run_registry_list(
    registry_path: Option<String>,
    backend_str: &str,
) -> Result<(), Box<dyn Error>> {
    let backend = parse_backend(backend_str);
    let registry_file = get_registry_file(registry_path, backend);

    if !Path::new(&registry_file).exists() {
        output::warning(&format!("Registry-Datei nicht gefunden: {}", registry_file));
        output::indent("Verwende 'registry add' um Einträge hinzuzufügen.");
        return Ok(());
    }

    // Open store and load entries
    let store = registry::open_store(backend, Path::new(&registry_file))?;
    let entries = store.list()?;

    output::separator();
    output::info(&format!("Proofs in local registry ({})", registry_file));
    output::separator();

    if entries.is_empty() {
        output::indent("(keine Einträge)");
    } else {
        for (idx, entry) in entries.iter().enumerate() {
            output::indent(&format!(
                "#{:<3} Manifest: {}…  Proof: {}…  Date: {}",
                idx + 1,
                &entry.manifest_hash[..12],
                &entry.proof_hash[..12],
                entry.registered_at
            ));
            if let Some(ref ts) = entry.timestamp_file {
                output::indent(&format!("     Timestamp: {}", ts));
            }
        }
    }

    output::separator();
    output::detail_fmt("Total", format!("{} Einträge", entries.len()));

    Ok(())
}

/// Registry verify - Verifiziert einen Proof gegen die Registry
pub fn run_registry_verify(
    manifest_path: &str,
    proof_path: &str,
    registry_path: Option<String>,
    backend_str: &str,
) -> Result<(), Box<dyn Error>> {
    output::searching("Verifiziere Proof gegen Registry...");

    let backend = parse_backend(backend_str);
    let registry_file = get_registry_file(registry_path, backend);

    if !Path::new(&registry_file).exists() {
        output::error(&format!("Registry-Datei nicht gefunden: {}", registry_file));
        return Err("Registry existiert nicht".into());
    }

    // Berechne Hashes
    let manifest_hash = registry::compute_file_hash(manifest_path)?;
    let proof_hash = registry::compute_file_hash(proof_path)?;
    output::detail("Manifest-Hash", &manifest_hash);
    output::detail("Proof-Hash", &proof_hash);

    // Open store and find entry
    let store = registry::open_store(backend, Path::new(&registry_file))?;
    let entry_opt = store.find_by_hashes(&manifest_hash, &proof_hash)?;

    // Verifiziere
    if let Some(entry) = entry_opt {
        output::success("Entry verified in registry");
        output::detail("ID", &entry.id);
        output::detail("Registered", &entry.registered_at);
        if let Some(ref ts) = entry.timestamp_file {
            output::detail("Timestamp", ts);
        }

        // Verify signature if present
        let signature_valid = registry::verify_entry_signature(&entry)?;
        if signature_valid {
            output::indent("✓ Ed25519 signature valid");
        } else {
            output::indent("⚠ No signature present (backward compatibility)");
        }

        // Log Audit-Event
        let mut audit = AuditLog::new("build/agent.audit.jsonl")?;
        audit.log_event(
            "registry_verified",
            json!({
                "manifest_hash": manifest_hash,
                "proof_hash": proof_hash,
                "registry_file": registry_file,
                "backend": backend_str,
                "signature_valid": signature_valid,
                "status": "ok"
            }),
        )?;

        Ok(())
    } else {
        let total = store.list()?.len();
        output::error("Hash mismatch or not registered");
        output::detail("Registry", &registry_file);
        output::detail_fmt("Einträge", total);
        Err("Proof nicht in Registry gefunden".into())
    }
}

/// Registry migrate - Migriert Registry zwischen Backends
pub fn run_registry_migrate(
    from_backend_str: &str,
    from_path: &str,
    to_backend_str: &str,
    to_path: &str,
) -> Result<(), Box<dyn Error>> {
    output::info("Migriere Registry...");
    output::detail("Von", &format!("{} ({})", from_path, from_backend_str));
    output::detail("Nach", &format!("{} ({})", to_path, to_backend_str));

    let from_backend = parse_backend_strict(from_backend_str)?;
    let to_backend = parse_backend_strict(to_backend_str)?;

    if !Path::new(from_path).exists() {
        return Err(format!("Quell-Registry nicht gefunden: {}", from_path).into());
    }

    // Open source store and load data
    let from_store = registry::open_store(from_backend, Path::new(from_path))?;
    output::indent("Lade Daten...");
    let registry_data = from_store.load()?;
    let entry_count = registry_data.entries.len();

    // Open target store
    let to_store = registry::open_store(to_backend, Path::new(to_path))?;

    // Save all data
    output::indent(&format!("Schreibe {} Einträge...", entry_count));
    to_store.save(&registry_data)?;

    // Log Audit-Event
    let mut audit = AuditLog::new("build/agent.audit.jsonl")?;
    audit.log_event(
        "registry_migrated",
        json!({
            "from_backend": from_backend_str,
            "from_path": from_path,
            "to_backend": to_backend_str,
            "to_path": to_path,
            "entries_migrated": entry_count
        }),
    )?;

    output::success("Migration erfolgreich:");
    output::detail_fmt("Einträge", entry_count);
    output::detail("Ziel", to_path);

    Ok(())
}

/// Registry inspect - Zeigt Registry-Metadaten und Statistiken an (v1.1)
pub fn run_registry_inspect(registry_path: Option<String>) -> Result<(), Box<dyn Error>> {
    use registry::UnifiedRegistry;
    use std::path::Path;

    let path = registry_path.unwrap_or_else(|| "build/registry.json".to_string());

    output::searching("Inspiziere Registry...");
    output::detail("Datei", &path);

    if !Path::new(&path).exists() {
        return Err(format!("Registry nicht gefunden: {}", path).into());
    }

    // Load registry (auto-detects v1.0/v1.1)
    let unified_registry = UnifiedRegistry::load(Path::new(&path))?;

    output::section("");
    output::stats("Registry-Informationen:");
    output::detail("Version", unified_registry.source_version());
    output::detail_fmt("Einträge", unified_registry.count());
    output::detail(
        "Migriert",
        if unified_registry.was_migrated() {
            "Ja (v1.0 → v1.1)"
        } else {
            "Nein"
        },
    );

    // Show v1.1 metadata if available
    let v1_1 = unified_registry.as_v1_1();
    output::section("");
    output::document("Metadaten (v1.1):");
    output::detail("Schema-Version", &v1_1.meta.schema_version);
    output::detail("Tool-Version", &v1_1.meta.tool_version);
    output::detail("Erstellt", &v1_1.meta.created_at);

    if let Some(migrated_from) = &v1_1.meta.migrated_from {
        output::detail("Migriert von", migrated_from);
    }
    if let Some(migrated_at) = &v1_1.meta.migrated_at {
        output::detail("Migriert am", migrated_at);
    }

    // Validate registry
    output::section("");
    output::success("Validierung:");
    match unified_registry.validate() {
        Ok(_) => output::indent("Registry ist gültig ✓"),
        Err(e) => {
            output::warning(&format!("Validierung fehlgeschlagen: {}", e));
            return Err(e.into());
        }
    }

    // Log audit event
    let mut audit = AuditLog::new("build/agent.audit.jsonl")?;
    audit.log_event(
        "registry_inspected",
        json!({
            "path": path,
            "version": unified_registry.source_version(),
            "entry_count": unified_registry.count(),
            "was_migrated": unified_registry.was_migrated()
        }),
    )?;

    Ok(())
}

/// Registry backfill-kid - Backfills KID-Felder aus public_key (v1.1)
pub fn run_registry_backfill_kid(
    registry_path: Option<String>,
    output_path: Option<String>,
) -> Result<(), Box<dyn Error>> {
    use registry::UnifiedRegistry;
    use std::path::Path;

    let input = registry_path.unwrap_or_else(|| "build/registry.json".to_string());
    let output = output_path.unwrap_or_else(|| input.clone());

    output::key("Backfilling KID-Felder...");
    output::detail("Input", &input);
    output::detail("Output", &output);

    if !Path::new(&input).exists() {
        return Err(format!("Registry nicht gefunden: {}", input).into());
    }

    // Load registry (auto-detects v1.0/v1.1)
    let mut unified_registry = UnifiedRegistry::load(Path::new(&input))?;

    output::detail_fmt("Einträge", unified_registry.count());

    // Backfill KIDs
    let backfilled_count = unified_registry.backfill_kids()?;

    if backfilled_count == 0 {
        output::section("");
        output::success("Keine KID-Backfills erforderlich");
        output::indent("Alle Einträge haben bereits KIDs oder keine public_key");
        return Ok(());
    }

    // Save updated registry
    unified_registry.save(Path::new(&output))?;

    output::section("");
    output::success("Backfill erfolgreich:");
    output::detail_fmt("KIDs hinzugefügt", backfilled_count);
    output::detail("Gespeichert in", &output);

    // Log audit event
    let mut audit = AuditLog::new("build/agent.audit.jsonl")?;
    audit.log_event(
        "registry_kid_backfilled",
        json!({
            "input": input,
            "output": output,
            "backfilled_count": backfilled_count
        }),
    )?;

    Ok(())
}
