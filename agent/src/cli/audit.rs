//! CLI-Handler für Audit-Kommandos
//!
//! Extrahiert aus main.rs für bessere Wartbarkeit.
//! Enthält: run_audit_tip, run_audit_anchor, run_audit_set_private_anchor,
//!          run_audit_set_public_anchor, run_audit_verify_anchor, run_audit_timestamp,
//!          run_audit_verify_timestamp, run_audit_append, run_audit_verify_chain, run_audit_export

use super::output;
use crate::audit::AuditLog;
use crate::{manifest, registry};
use serde_json::json;
use std::error::Error;

/// Audit tip - Schreibt den Audit-Tip in eine Datei
pub fn run_audit_tip(out: Option<String>) -> Result<(), Box<dyn Error>> {
    output::writing("Schreibe Audit-Tip...");

    let out_path = out.unwrap_or_else(|| "build/audit.head".to_string());
    let audit_log_path = "build/agent.audit.jsonl";

    // Lade Audit-Log
    let audit = AuditLog::new(audit_log_path)?;

    // Schreibe Tip
    audit.write_tip(&out_path)?;

    output::success_with("Audit-Tip geschrieben nach", &out_path);
    output::detail("Tip", &audit.get_tip());

    Ok(())
}

/// Audit anchor - Setzt einen Zeitanker im Manifest
pub fn run_audit_anchor(
    kind: &str,
    reference: &str,
    manifest_in: &str,
    manifest_out: &str,
) -> Result<(), Box<dyn Error>> {
    output::timing("Setze Zeitanker im Manifest...");

    // Prüfe ob build/audit.head existiert
    let tip_path = "build/audit.head";
    if !std::path::Path::new(tip_path).exists() {
        return Err(format!(
            "Precondition-Fehler: {} existiert nicht. Führe zuerst 'audit tip' aus.",
            tip_path
        )
        .into());
    }

    // Lade Audit-Tip
    let audit_tip_hex = AuditLog::read_tip(tip_path)?;

    // Lade Manifest
    let mut manifest = manifest::Manifest::load(manifest_in)?;

    // Setze Zeitanker
    manifest.set_time_anchor(
        kind.to_string(),
        reference.to_string(),
        audit_tip_hex.clone(),
    );

    // Speichere Manifest
    manifest.save(manifest_out)?;

    output::success("Zeitanker gesetzt:");
    output::detail("Kind", kind);
    output::detail("Referenz", reference);
    output::detail("Audit-Tip", &audit_tip_hex);
    output::detail("Output", manifest_out);

    Ok(())
}

/// Audit set-private-anchor - Setzt Private Anchor (Dual-Anchor v0.9.0)
pub fn run_audit_set_private_anchor(
    manifest_path: &str,
    audit_tip: &str,
    created_at: Option<String>,
) -> Result<(), Box<dyn Error>> {
    output::secure("Setze Private Anchor...");

    // Lade Manifest
    let mut manifest = manifest::Manifest::load(manifest_path)?;

    // Setze Private Anchor
    manifest.set_private_anchor(audit_tip.to_string(), created_at.clone())?;

    // Speichere Manifest
    manifest.save(manifest_path)?;

    output::success("Private Anchor gesetzt:");
    output::detail("Audit-Tip", audit_tip);
    output::detail("Created-At", &created_at.unwrap_or_else(|| "jetzt".to_string()));
    output::detail("Manifest", manifest_path);

    Ok(())
}

/// Audit set-public-anchor - Setzt Public Anchor (Dual-Anchor v0.9.0)
pub fn run_audit_set_public_anchor(
    manifest_path: &str,
    chain: &str,
    txid: &str,
    digest: &str,
    created_at: Option<String>,
) -> Result<(), Box<dyn Error>> {
    output::network("Setze Public Anchor...");

    // Parse chain
    let chain_enum = match chain.to_lowercase().as_str() {
        "ethereum" => manifest::PublicChain::Ethereum,
        "hedera" => manifest::PublicChain::Hedera,
        "btc" => manifest::PublicChain::Btc,
        _ => {
            return Err(format!(
                "Invalid chain: {}. Valid options: ethereum, hedera, btc",
                chain
            )
            .into())
        }
    };

    // Lade Manifest
    let mut manifest = manifest::Manifest::load(manifest_path)?;

    // Setze Public Anchor
    manifest.set_public_anchor(
        chain_enum,
        txid.to_string(),
        digest.to_string(),
        created_at.clone(),
    )?;

    // Speichere Manifest
    manifest.save(manifest_path)?;

    output::success("Public Anchor gesetzt:");
    output::detail("Chain", chain);
    output::detail("TxID", txid);
    output::detail("Digest", digest);
    output::detail("Created-At", &created_at.unwrap_or_else(|| "jetzt".to_string()));
    output::detail("Manifest", manifest_path);

    Ok(())
}

/// Audit verify-anchor - Verifiziert Dual-Anchor-Konsistenz
pub fn run_audit_verify_anchor(
    manifest_path: &str,
    out: Option<String>,
) -> Result<(), Box<dyn Error>> {
    output::searching("Verifiziere Dual-Anchor-Konsistenz...");

    // Lade Manifest
    let manifest = manifest::Manifest::load(manifest_path)?;

    // Validiere Dual-Anchor
    let validation_result = manifest.validate_dual_anchor();

    let report = if let Err(e) = validation_result {
        json!({
            "status": "fail",
            "manifest": manifest_path,
            "errors": [e.to_string()],
            "private_ok": false,
            "public_ok": false,
            "digest_match": false,
        })
    } else {
        // Check individual components
        let (has_private, has_public) = manifest
            .time_anchor
            .as_ref()
            .map(|anchor| (anchor.private.is_some(), anchor.public.is_some()))
            .unwrap_or((false, false));

        json!({
            "status": "ok",
            "manifest": manifest_path,
            "errors": [],
            "private_ok": has_private,
            "public_ok": has_public,
            "digest_match": true, // TODO: actual digest validation if needed
        })
    };

    // Print result
    output::section("");
    output::stats("Verifikationsergebnis:");
    output::detail("Status", report["status"].as_str().unwrap_or("unknown"));
    output::detail(
        "Private Anchor",
        if report["private_ok"].as_bool().unwrap_or(false) { "gültig" } else { "fehlt/ungültig" },
    );
    output::detail(
        "Public Anchor",
        if report["public_ok"].as_bool().unwrap_or(false) { "gültig" } else { "fehlt/ungültig" },
    );

    if let Some(errors) = report["errors"].as_array() {
        if !errors.is_empty() {
            output::section("");
            output::error("Fehler:");
            for error in errors {
                output::indent(&format!("- {}", error.as_str().unwrap_or("Unknown error")));
            }
        }
    }

    // Save report if requested
    if let Some(out_path) = out {
        let json_str = serde_json::to_string_pretty(&report)?;
        std::fs::write(&out_path, json_str)?;
        output::section("");
        output::saving(&format!("Report gespeichert: {}", out_path));
    }

    // Return error if validation failed
    if report["status"] == "fail" {
        return Err("Dual-Anchor validation failed".into());
    }

    Ok(())
}

/// Audit timestamp - Erstellt einen Timestamp für den Audit-Head
pub fn run_audit_timestamp(
    head_path: &str,
    out: Option<String>,
    is_mock: bool,
    tsa_url: Option<String>,
) -> Result<(), Box<dyn Error>> {
    output::timing("Erstelle Timestamp für Audit-Head...");

    // Lade Audit-Tip
    let audit_tip_hex = std::fs::read_to_string(head_path)?;
    let audit_tip_hex = audit_tip_hex.trim().to_string();

    // Erstelle Timestamp
    let timestamp = if is_mock {
        output::warning("MOCK TIMESTAMP (nicht für Produktion geeignet)");
        registry::Timestamp::create_mock(audit_tip_hex)
    } else if let Some(url) = tsa_url {
        return Err(format!(
            "Echter TSA-Timestamp noch nicht implementiert. TSA-URL: {}",
            url
        )
        .into());
    } else {
        return Err("Bitte --mock oder --tsa-url angeben".into());
    };

    // Speichere Timestamp
    let out_path = out.unwrap_or_else(|| "build/timestamp.tsr".to_string());
    timestamp.save(&out_path)?;

    // Log Audit-Event
    let mut audit = AuditLog::new("build/agent.audit.jsonl")?;
    audit.log_event(
        "timestamp_generated",
        json!({
            "audit_tip": timestamp.audit_tip_hex,
            "output": out_path,
            "tsa": timestamp.tsa,
            "created_at": timestamp.created_at
        }),
    )?;

    output::success("Timestamp erstellt:");
    output::detail("Audit-Tip", &timestamp.audit_tip_hex);
    output::detail("Erstellt", &timestamp.created_at);
    output::detail("TSA", &timestamp.tsa);
    output::detail("Output", &out_path);

    Ok(())
}

/// Audit verify-timestamp - Verifiziert einen Timestamp gegen Audit-Head
pub fn run_audit_verify_timestamp(
    head_path: &str,
    timestamp_path: &str,
) -> Result<(), Box<dyn Error>> {
    output::searching("Verifiziere Timestamp...");

    // Lade Audit-Tip
    let audit_tip_hex = std::fs::read_to_string(head_path)?;
    let audit_tip_hex = audit_tip_hex.trim();

    // Lade Timestamp
    let timestamp = registry::Timestamp::load(timestamp_path)?;

    // Verifiziere
    if timestamp.verify(audit_tip_hex) {
        output::success("Timestamp valid");
        output::detail("Audit-Tip", &timestamp.audit_tip_hex);
        output::detail("Erstellt", &timestamp.created_at);
        output::detail("TSA", &timestamp.tsa);

        // Log Audit-Event
        let mut audit = AuditLog::new("build/agent.audit.jsonl")?;
        audit.log_event(
            "timestamp_verified",
            json!({
                "audit_tip": audit_tip_hex,
                "timestamp_file": timestamp_path,
                "status": "ok"
            }),
        )?;

        Ok(())
    } else {
        output::error("Timestamp invalid or mismatched head");
        output::detail("Erwartet", audit_tip_hex);
        output::detail("Gefunden", &timestamp.audit_tip_hex);
        Err("Timestamp-Verifikation fehlgeschlagen".into())
    }
}

/// Audit append - Fügt Event zur Audit-Chain hinzu (Track A)
pub fn run_audit_append(
    file_path: &str,
    event: &str,
    policy_id: Option<String>,
    ir_hash: Option<String>,
    manifest_hash: Option<String>,
    result: Option<String>,
    run_id: Option<String>,
) -> Result<(), Box<dyn Error>> {
    use crate::audit::{AuditChain, AuditEventResult};

    output::writing("Füge Event zur Audit-Chain hinzu...");

    // Parse result if provided
    let parsed_result = result
        .as_ref()
        .and_then(|r| match r.to_lowercase().as_str() {
            "ok" => Some(AuditEventResult::Ok),
            "warn" => Some(AuditEventResult::Warn),
            "fail" => Some(AuditEventResult::Fail),
            _ => None,
        });

    // Open or create chain
    let mut chain = AuditChain::new(file_path)?;

    // Append event
    let audit_event = chain.append(
        event.to_string(),
        policy_id,
        ir_hash,
        manifest_hash,
        parsed_result,
        run_id,
    )?;

    output::success("Event hinzugefügt");
    output::detail("Event", &audit_event.event);
    output::detail("Timestamp", &audit_event.ts);
    output::detail("Self-Hash", &audit_event.self_hash);
    output::detail("Chain-Datei", file_path);

    Ok(())
}

/// Audit verify - Verifiziert Audit-Chain-Integrität (Track A)
pub fn run_audit_verify_chain(
    file_path: &str,
    out: Option<String>,
) -> Result<(), Box<dyn Error>> {
    use crate::audit::verify_chain;

    output::searching("Verifiziere Audit-Chain...");

    let report = verify_chain(file_path)?;

    if report.ok {
        output::success("Chain-Integrität OK");
        output::detail_fmt("Events", report.total_events);
        output::detail("Tamper-Index", "None");
    } else {
        output::error("Chain-Integrität VERLETZT");
        output::detail_fmt("Events", report.total_events);
        if let Some(idx) = report.tamper_index {
            output::detail_fmt("Tamper-Index", idx);
        }
        if let Some(err) = &report.error {
            output::detail("Fehler", err);
        }
    }

    // Write JSON report if requested
    if let Some(out_path) = out {
        let report_json = serde_json::json!({
            "ok": report.ok,
            "total_events": report.total_events,
            "tamper_index": report.tamper_index,
            "error": report.error,
        });
        std::fs::write(&out_path, serde_json::to_string_pretty(&report_json)?)?;
        output::document(&format!("Report gespeichert: {}", out_path));
    }

    if !report.ok {
        return Err("Chain-Verifikation fehlgeschlagen".into());
    }

    Ok(())
}

/// Audit export - Exportiert Events aus Audit-Chain (Track A)
pub fn run_audit_export(
    file_path: &str,
    from: Option<String>,
    to: Option<String>,
    policy_id: Option<String>,
    out: Option<String>,
) -> Result<(), Box<dyn Error>> {
    use crate::audit::export_events;

    output::packaging("Exportiere Events aus Audit-Chain...");

    let events = export_events(
        file_path,
        from.as_deref(),
        to.as_deref(),
        policy_id.as_deref(),
    )?;

    output::success_with("Events exportiert", &format!("{}", events.len()));

    // Output to file or stdout
    let json_output = serde_json::to_string_pretty(&events)?;

    if let Some(out_path) = out {
        std::fs::write(&out_path, &json_output)?;
        output::document(&format!("Events gespeichert: {}", out_path));
    } else {
        println!("\n{}", json_output);
    }

    Ok(())
}
