//! CLI-Handler für BLOB Store Kommandos
//!
//! Extrahiert aus main.rs für bessere Wartbarkeit.
//! Enthält: run_blob_put, run_blob_get, run_blob_list, run_blob_gc

use super::output;
use crate::audit::AuditLog;
use crate::blob_store::{BlobStore, SqliteBlobStore};
use crate::crypto;
use serde_json::json;
use std::error::Error;
use std::fs;
use std::io::{stdin, stdout, Read, Write};

/// Fügt eine Datei in den BLOB Store ein (CAS + optional Registry-Verknüpfung)
#[allow(clippy::too_many_arguments)]
pub fn run_blob_put(
    file: Option<String>,
    media_type: &str,
    registry_path: &str,
    link_entry_id: Option<String>,
    use_stdin: bool,
    out: Option<String>,
    no_dedup: bool,
) -> Result<(), Box<dyn Error>> {
    // Validiere Medientyp
    let valid_types = ["manifest", "proof", "wasm", "abi", "other"];
    if !valid_types.contains(&media_type) {
        return Err(format!(
            "Ungültiger Medientyp: {}. Erlaubt: {:?}",
            media_type, valid_types
        )
        .into());
    }

    // Lese Daten von Datei oder stdin
    let data = if use_stdin {
        output::input("Lese Daten von stdin...");
        let mut buffer = Vec::new();
        stdin().read_to_end(&mut buffer)?;
        buffer
    } else if let Some(file_path) = file {
        output::input(&format!("Lese Datei: {}", file_path));
        fs::read(&file_path)?
    } else {
        return Err("Entweder --file oder --stdin muss angegeben werden".into());
    };

    output::stats(&format!("Größe: {} bytes, Medientyp: {}", data.len(), media_type));

    // Öffne BLOB Store
    let mut store = SqliteBlobStore::new(registry_path)?;

    // Berechne BLAKE3 Hash für Deduplizierung
    let blob_id_preview = crypto::hex_lower_prefixed32(crypto::blake3_256(&data));

    // Prüfe ob Blob bereits existiert
    if store.exists(&blob_id_preview) && !no_dedup {
        output::success_with("BLOB existiert bereits (dedupliziert)", &blob_id_preview);

        // Erhöhe refcount wenn link_entry_id angegeben
        if link_entry_id.is_some() {
            store.pin(&blob_id_preview)?;
            output::pinned("Refcount erhöht");
        }
    } else {
        // Insert BLOB
        let blob_id = store.put(&data, media_type)?;
        output::success_with("BLOB gespeichert", &blob_id);

        // Erhöhe refcount wenn link_entry_id angegeben
        if link_entry_id.is_some() {
            store.pin(&blob_id)?;
            output::pinned("Refcount erhöht für Registry-Verknüpfung");
        }
    }

    // Schreibe blob_id in Output-Datei falls angegeben
    if let Some(out_path) = out {
        fs::write(&out_path, blob_id_preview.as_bytes())?;
        output::document(&format!("BLOB ID geschrieben nach: {}", out_path));
    } else {
        // Ausgabe auf stdout wenn --out fehlt
        println!("\n{}", blob_id_preview);
    }

    // Audit-Log-Eintrag
    let mut audit = AuditLog::new("build/agent.audit.jsonl")?;
    audit.log_event(
        "blob_put",
        json!({
            "blob_id": blob_id_preview,
            "media_type": media_type,
            "size": data.len(),
            "linked_entry": link_entry_id,
        }),
    )?;

    Ok(())
}

/// Extrahiert Blob-Inhalt anhand blob_id auf Datei oder stdout
pub fn run_blob_get(
    blob_id: &str,
    out: Option<String>,
    use_stdout: bool,
    registry_path: &str,
) -> Result<(), Box<dyn Error>> {
    // Validiere BLOB ID Format
    if !blob_id.starts_with("0x") || blob_id.len() != 66 {
        return Err("Ungültige BLOB ID Format (erwartet: 0x + 64 hex chars)".into());
    }

    // Öffne BLOB Store
    let store = SqliteBlobStore::new(registry_path)?;

    // Hole BLOB
    output::searching(&format!("Suche BLOB: {}", blob_id));
    let data = store.get(blob_id)?;

    output::success_with("BLOB gefunden, Größe", &format!("{} bytes", data.len()));

    // Schreibe auf Datei oder stdout
    if let Some(out_path) = out {
        fs::write(&out_path, &data)?;
        output::document(&format!("BLOB geschrieben nach: {}", out_path));
    } else if use_stdout {
        stdout().write_all(&data)?;
    } else {
        // Default: stdout
        stdout().write_all(&data)?;
    }

    // Audit-Log-Eintrag
    let mut audit = AuditLog::new("build/agent.audit.jsonl")?;
    audit.log_event(
        "blob_get",
        json!({
            "blob_id": blob_id,
            "size": data.len(),
        }),
    )?;

    Ok(())
}

/// Listet Blobs gefiltert/sortiert
#[allow(clippy::too_many_arguments)]
pub fn run_blob_list(
    media_type: Option<String>,
    min_size: Option<u64>,
    max_size: Option<u64>,
    unused_only: bool,
    limit: Option<usize>,
    order: &str,
    registry_path: &str,
) -> Result<(), Box<dyn Error>> {
    // Öffne BLOB Store
    let store = SqliteBlobStore::new(registry_path)?;

    // Hole alle BLOBs
    let mut blobs = store.list()?;

    // Filter: media_type
    if let Some(ref mt) = media_type {
        blobs.retain(|b| &b.media_type == mt);
    }

    // Filter: min_size
    if let Some(min) = min_size {
        blobs.retain(|b| b.size >= min as usize);
    }

    // Filter: max_size
    if let Some(max) = max_size {
        blobs.retain(|b| b.size <= max as usize);
    }

    // Filter: unused_only (refcount = 0)
    if unused_only {
        blobs.retain(|b| b.refcount == 0);
    }

    // Sortierung
    match order {
        "size" => blobs.sort_by_key(|b| b.size),
        "refcount" => blobs.sort_by_key(|b| b.refcount),
        "blob_id" => blobs.sort_by(|a, b| a.blob_id.cmp(&b.blob_id)),
        _ => {
            return Err(format!(
                "Ungültige Sortierung: {}. Erlaubt: size, refcount, blob_id",
                order
            )
            .into())
        }
    }

    // Limit
    if let Some(l) = limit {
        blobs.truncate(l);
    }

    // Ausgabe
    output::listing(&format!("Gefundene BLOBs: {}", blobs.len()));
    output::table_header(&[
        ("BLOB ID", 66),
        ("Size (bytes)", 15),
        ("Media Type", 20),
        ("Refcount", 10),
    ]);

    for blob in &blobs {
        output::table_row(&[
            (&blob.blob_id, 66),
            (&format!("{}", blob.size), 15),
            (&blob.media_type, 20),
            (&format!("{}", blob.refcount), 10),
        ]);
    }

    // Audit-Log-Eintrag
    let mut audit = AuditLog::new("build/agent.audit.jsonl")?;
    audit.log_event(
        "blob_list",
        json!({
            "count": blobs.len(),
            "filters": {
                "media_type": media_type,
                "min_size": min_size,
                "max_size": max_size,
                "unused_only": unused_only,
            }
        }),
    )?;

    Ok(())
}

/// Garbage Collection nicht referenzierter Blobs
pub fn run_blob_gc(
    dry_run: bool,
    force: bool,
    min_age: Option<String>,
    print_ids: bool,
    registry_path: &str,
) -> Result<(), Box<dyn Error>> {
    if min_age.is_some() {
        output::warning("--min-age ist noch nicht implementiert (zukünftige Feature)");
    }

    // Öffne BLOB Store
    let mut store = SqliteBlobStore::new(registry_path)?;

    // Führe GC aus (dry-run oder real)
    output::deleting("Starte Garbage Collection...");
    let gc_candidates = store.gc(true)?; // Erst dry-run für Anzeige

    if gc_candidates.is_empty() {
        output::success("Keine unreferenzierten BLOBs gefunden");
        return Ok(());
    }

    output::stats(&format!("Unreferenzierte BLOBs: {}", gc_candidates.len()));

    if print_ids {
        output::section("");
        output::deleting("Zu löschende BLOB IDs:");
        for id in &gc_candidates {
            output::indent(&format!("- {}", id));
        }
    }

    // Berechne Gesamtgröße
    let mut total_bytes = 0u64;
    for id in &gc_candidates {
        if let Ok(data) = store.get(id) {
            total_bytes += data.len() as u64;
        }
    }

    output::saving(&format!(
        "Freizugebender Speicher: {} bytes ({:.2} MB)",
        total_bytes,
        total_bytes as f64 / 1_048_576.0
    ));

    if dry_run {
        output::section("");
        output::searching("DRY RUN - Keine Löschung durchgeführt");
        output::info("Führen Sie den Befehl mit --force aus, um zu löschen");
        return Ok(());
    }

    if !force {
        output::section("");
        output::warning("Bitte bestätigen Sie die Löschung mit --force");
        return Ok(());
    }

    // Real GC
    output::section("");
    output::deleting("Lösche unreferenzierte BLOBs...");
    store.gc(false)?;
    output::success(&format!(
        "{} BLOBs gelöscht, {} bytes freigegeben",
        gc_candidates.len(),
        total_bytes
    ));

    // Audit-Log-Eintrag
    let mut audit = AuditLog::new("build/agent.audit.jsonl")?;
    audit.log_event(
        "blob_gc",
        json!({
            "deleted_count": gc_candidates.len(),
            "bytes_freed": total_bytes,
            "dry_run": false,
        }),
    )?;

    Ok(())
}
