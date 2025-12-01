//! CLI-Handler für Policy-Kommandos
//!
//! Extrahiert aus main.rs für bessere Wartbarkeit.
//! Enthält: run_policy_validate

use super::output;
use crate::audit::AuditLog;
use crate::policy;
use serde_json::json;
use std::error::Error;

/// Policy validate
pub fn run_policy_validate(file: &str) -> Result<(), Box<dyn Error>> {
    output::searching(&format!("Validiere Policy: {}", file));

    let mut audit = AuditLog::new("build/agent.audit.jsonl")?;

    let policy = policy::Policy::load(file)?;
    audit.log_event(
        "policy_loaded",
        json!({
            "file": file,
            "name": &policy.name,
            "version": &policy.version
        }),
    )?;

    policy.validate()?;
    audit.log_event("policy_validated", json!({ "name": &policy.name }))?;

    let hash = policy.compute_hash()?;

    output::success("Policy ist gültig!");
    output::detail("Name", &policy.name);
    output::detail("Version", &policy.version);
    output::detail("Hash", &hash);

    Ok(())
}
