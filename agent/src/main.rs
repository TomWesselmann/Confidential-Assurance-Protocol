mod audit;
mod commitment;
mod io;
mod lists;
mod manifest;
mod policy;
mod proof_engine;
mod proof_mock;
mod registry;
mod sign;
mod verifier;
mod zk_system;

use audit::AuditLog;
use clap::{Parser, Subcommand};
use commitment::{compute_company_root, compute_supplier_root, compute_ubo_root, Commitments};
use serde_json::json;
use std::error::Error;
use std::fs;
use zk_system::ProofSystem;

const VERSION: &str = "0.4.0";

/// LkSG Proof Agent - Confidential Assurance Protocol (CAP)
///
/// CLI-Tool zur Erzeugung kryptografisch prüfbarer Commitments
/// für Lieferketten- und Sanktionsprüfungen.
#[derive(Parser)]
#[command(name = "cap-agent")]
#[command(version = VERSION)]
#[command(about = "LkSG Proof Agent (Proof & Verifier Layer MVP)", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Verarbeitet CSV-Dateien und erzeugt Commitments + Audit-Log
    Prepare {
        /// Pfad zur Suppliers CSV-Datei
        #[arg(long)]
        suppliers: String,

        /// Pfad zur UBOs CSV-Datei
        #[arg(long)]
        ubos: String,
    },
    /// Zeigt den Inhalt einer Commitments-Datei an
    Inspect {
        /// Pfad zur commitments.json Datei
        path: String,
    },
    /// Policy-Commands
    #[command(subcommand)]
    Policy(PolicyCommands),
    /// Manifest-Commands
    #[command(subcommand)]
    Manifest(ManifestCommands),
    /// Proof-Commands
    #[command(subcommand)]
    Proof(ProofCommands),
    /// Sign-Commands
    #[command(subcommand)]
    Sign(SignCommands),
    /// Verifier-Commands
    #[command(subcommand)]
    Verifier(VerifierCommands),
    /// Audit-Commands
    #[command(subcommand)]
    Audit(AuditCommands),
    /// Lists-Commands
    #[command(subcommand)]
    Lists(ListsCommands),
    /// Registry-Commands
    #[command(subcommand)]
    Registry(RegistryCommands),
    /// Zeigt die Tool-Version an
    Version,
}

#[derive(Subcommand)]
enum PolicyCommands {
    /// Validiert eine Policy-Datei
    Validate {
        /// Pfad zur Policy-Datei (YAML oder JSON)
        #[arg(long)]
        file: String,
    },
}

#[derive(Subcommand)]
enum ManifestCommands {
    /// Erstellt ein Manifest aus Commitments und Policy
    Build {
        /// Pfad zur Policy-Datei
        #[arg(long)]
        policy: String,

        /// Optional: Output-Pfad (default: build/manifest.json)
        #[arg(long)]
        out: Option<String>,
    },
}

#[derive(Subcommand)]
enum ProofCommands {
    /// Generiert einen Mock-Proof (Tag 2)
    Mock {
        /// Pfad zur Policy-Datei
        #[arg(long)]
        policy: String,

        /// Pfad zum Manifest
        #[arg(long)]
        manifest: String,
    },
    /// Erstellt einen strukturierten Proof (Tag 3)
    Build {
        /// Pfad zur Policy-Datei
        #[arg(long)]
        policy: String,

        /// Pfad zum Manifest
        #[arg(long)]
        manifest: String,
    },
    /// Verifiziert einen Proof gegen Manifest
    Verify {
        /// Pfad zum Proof (.dat oder .json)
        #[arg(long)]
        proof: String,

        /// Pfad zum Manifest
        #[arg(long)]
        manifest: String,
    },
    /// Exportiert ein Proof-Paket
    Export {
        /// Pfad zum Manifest
        #[arg(long)]
        manifest: String,

        /// Pfad zum Proof
        #[arg(long)]
        proof: String,

        /// Optional: Pfad zur Signatur
        #[arg(long)]
        signature: Option<String>,

        /// Output-Verzeichnis (default: build/proof_package)
        #[arg(long)]
        out: Option<String>,
    },
    /// Erstellt einen Zero-Knowledge-Proof (Tag 4)
    ZkBuild {
        /// Pfad zur Policy-Datei
        #[arg(long)]
        policy: String,

        /// Pfad zum Manifest
        #[arg(long)]
        manifest: String,

        /// Output-Pfad (default: build/zk_proof.dat)
        #[arg(long)]
        out: Option<String>,

        /// Optionaler Sanctions-Root (Hex-String)
        #[arg(long)]
        sanctions_root: Option<String>,

        /// Optionaler Jurisdiction-Root (Hex-String)
        #[arg(long)]
        jurisdiction_root: Option<String>,

        /// Optionale Sanctions CSV für Mock-Check
        #[arg(long)]
        sanctions_csv: Option<String>,
    },
    /// Verifiziert einen Zero-Knowledge-Proof
    ZkVerify {
        /// Pfad zum ZK-Proof (.dat oder .json)
        #[arg(long)]
        proof: String,
    },
    /// Benchmark für ZK-Proof-Erstellung und Verifikation
    Bench {
        /// Pfad zur Policy-Datei
        #[arg(long)]
        policy: String,

        /// Pfad zum Manifest
        #[arg(long)]
        manifest: String,

        /// Anzahl der Iterationen (default: 1)
        #[arg(long, default_value = "1")]
        iterations: usize,
    },
}

#[derive(Subcommand)]
enum SignCommands {
    /// Generiert ein Ed25519-Schlüsselpaar
    Keygen {
        /// Optional: Verzeichnis für Keys (default: keys/)
        #[arg(long)]
        dir: Option<String>,
    },
    /// Signiert ein Manifest
    Manifest {
        /// Pfad zum privaten Schlüssel
        #[arg(long)]
        key: String,

        /// Pfad zum Manifest
        #[arg(long, value_name = "in")]
        manifest_in: String,

        /// Output-Pfad für signiertes Manifest
        #[arg(long)]
        out: String,

        /// Optional: Name des Signierers (default: "Company")
        #[arg(long)]
        signer: Option<String>,
    },
    /// Verifiziert ein signiertes Manifest
    VerifyManifest {
        /// Pfad zum öffentlichen Schlüssel
        #[arg(long)]
        pub_key: String,

        /// Pfad zum signierten Manifest
        #[arg(long, value_name = "in")]
        signed_in: String,
    },
}

#[derive(Subcommand)]
enum VerifierCommands {
    /// Verifiziert ein Proof-Paket
    Run {
        /// Pfad zum Proof-Paket-Verzeichnis
        #[arg(long)]
        package: String,
    },
    /// Extrahiert Informationen aus Manifest
    Extract {
        /// Pfad zum Proof-Paket-Verzeichnis
        #[arg(long)]
        package: String,
    },
    /// Zeigt Audit-Trail an
    Audit {
        /// Pfad zum Proof-Paket-Verzeichnis
        #[arg(long)]
        package: String,
    },
}

#[derive(Subcommand)]
enum AuditCommands {
    /// Schreibt den Audit-Tip (aktueller Hash der Audit-Chain) in eine Datei
    Tip {
        /// Output-Pfad (default: build/audit.head)
        #[arg(long)]
        out: Option<String>,
    },
    /// Setzt einen Zeitanker im Manifest
    Anchor {
        /// Art des Zeitankers (tsa, blockchain, file)
        #[arg(long)]
        kind: String,

        /// Referenz (Pfad, TxID oder URI)
        #[arg(long, value_name = "ref")]
        reference: String,

        /// Input-Manifest-Pfad
        #[arg(long)]
        manifest_in: String,

        /// Output-Manifest-Pfad
        #[arg(long)]
        manifest_out: String,
    },
    /// Erstellt einen Timestamp für den Audit-Head
    Timestamp {
        /// Pfad zur Audit-Head-Datei
        #[arg(long)]
        head: String,

        /// Output-Pfad (default: build/timestamp.tsr)
        #[arg(long)]
        out: Option<String>,

        /// Verwendet Mock-Timestamp (default: true)
        #[arg(long, default_value = "true")]
        mock: bool,

        /// Optionale TSA-URL (für echten Timestamp)
        #[arg(long)]
        tsa_url: Option<String>,
    },
    /// Verifiziert einen Timestamp gegen Audit-Head
    VerifyTimestamp {
        /// Pfad zur Audit-Head-Datei
        #[arg(long)]
        head: String,

        /// Pfad zur Timestamp-Datei
        #[arg(long)]
        timestamp: String,
    },
}

#[derive(Subcommand)]
enum ListsCommands {
    /// Generiert Sanctions Merkle Root aus CSV
    SanctionsRoot {
        /// Pfad zur Sanctions CSV
        #[arg(long)]
        csv: String,

        /// Output-Pfad (default: build/sanctions.root)
        #[arg(long)]
        out: Option<String>,
    },
    /// Generiert Jurisdictions Merkle Root aus CSV
    JurisdictionsRoot {
        /// Pfad zur Jurisdictions CSV
        #[arg(long)]
        csv: String,

        /// Output-Pfad (default: build/jurisdictions.root)
        #[arg(long)]
        out: Option<String>,
    },
}

#[derive(Subcommand)]
enum RegistryCommands {
    /// Fügt einen Proof zur Registry hinzu
    Add {
        /// Pfad zum Manifest
        #[arg(long)]
        manifest: String,

        /// Pfad zum Proof
        #[arg(long)]
        proof: String,

        /// Optionaler Pfad zur Timestamp-Datei
        #[arg(long)]
        timestamp: Option<String>,

        /// Registry-Datei (default: build/registry.json)
        #[arg(long)]
        registry: Option<String>,
    },
    /// Listet alle Registry-Einträge auf
    List {
        /// Registry-Datei (default: build/registry.json)
        #[arg(long)]
        registry: Option<String>,
    },
    /// Verifiziert einen Proof gegen die Registry
    Verify {
        /// Pfad zum Manifest
        #[arg(long)]
        manifest: String,

        /// Pfad zum Proof
        #[arg(long)]
        proof: String,

        /// Registry-Datei (default: build/registry.json)
        #[arg(long)]
        registry: Option<String>,
    },
}

/// Hauptfunktion: Führt das prepare-Kommando aus
///
/// Liest CSV-Dateien, berechnet Merkle-Roots und speichert Commitments + Audit-Log
fn run_prepare(suppliers_path: &str, ubos_path: &str) -> Result<(), Box<dyn Error>> {
    println!("🔄 Starte Commitment-Berechnung...");

    // Erstelle build-Verzeichnis falls nicht vorhanden
    fs::create_dir_all("build")?;

    // Initialisiere Audit-Log
    let mut audit = AuditLog::new("build/agent.audit.jsonl")?;

    // Log: Start des Prepare-Prozesses
    audit.log_event(
        "prepare_started",
        json!({
            "suppliers_file": suppliers_path,
            "ubos_file": ubos_path
        }),
    )?;

    // Lese CSV-Dateien
    println!("📂 Lese Suppliers aus: {}", suppliers_path);
    let suppliers = io::read_suppliers_csv(suppliers_path)?;
    audit.log_event(
        "data_loaded",
        json!({
            "type": "suppliers",
            "count": suppliers.len()
        }),
    )?;

    println!("📂 Lese UBOs aus: {}", ubos_path);
    let ubos = io::read_ubos_csv(ubos_path)?;
    audit.log_event(
        "data_loaded",
        json!({
            "type": "ubos",
            "count": ubos.len()
        }),
    )?;

    // Berechne Merkle-Roots
    println!("🧮 Berechne Supplier-Root...");
    let supplier_root = compute_supplier_root(&suppliers)?;
    audit.log_event(
        "merkle_root_computed",
        json!({
            "target": "suppliers",
            "root": &supplier_root
        }),
    )?;

    println!("🧮 Berechne UBO-Root...");
    let ubo_root = compute_ubo_root(&ubos)?;
    audit.log_event(
        "merkle_root_computed",
        json!({
            "target": "ubos",
            "root": &ubo_root
        }),
    )?;

    println!("🧮 Berechne Company-Commitment-Root...");
    let company_commitment_root = compute_company_root(&supplier_root, &ubo_root);
    audit.log_event(
        "merkle_root_computed",
        json!({
            "target": "company",
            "root": &company_commitment_root
        }),
    )?;

    // Erstelle Commitments-Objekt
    let commitments = Commitments {
        supplier_root: supplier_root.clone(),
        ubo_root: ubo_root.clone(),
        company_commitment_root: company_commitment_root.clone(),
        supplier_count: Some(suppliers.len()),
        ubo_count: Some(ubos.len()),
    };

    // Speichere Commitments
    let output_path = "build/commitments.json";
    println!("💾 Speichere Commitments nach: {}", output_path);
    commitment::save_commitments(&commitments, output_path)?;

    audit.log_event(
        "commitments_saved",
        json!({
            "path": output_path
        }),
    )?;

    println!("✅ Erfolgreich abgeschlossen!");
    println!("\n📊 Ergebnisse:");
    println!("  Supplier Root:  {}", supplier_root);
    println!("  UBO Root:       {}", ubo_root);
    println!("  Company Root:   {}", company_commitment_root);
    println!("\n📁 Ausgabedateien:");
    println!("  - {}", output_path);
    println!("  - build/agent.audit.jsonl");

    Ok(())
}

/// Führt das inspect-Kommando aus
///
/// Lädt und zeigt eine Commitments-Datei an
fn run_inspect(path: &str) -> Result<(), Box<dyn Error>> {
    println!("🔍 Lese Commitments von: {}", path);

    let commitments = commitment::load_commitments(path)?;
    let json = serde_json::to_string_pretty(&commitments)?;

    println!("\n{}", json);

    Ok(())
}

/// Policy validate
fn run_policy_validate(file: &str) -> Result<(), Box<dyn Error>> {
    println!("🔍 Validiere Policy: {}", file);

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

    println!("✅ Policy ist gültig!");
    println!("  Name:    {}", policy.name);
    println!("  Version: {}", policy.version);
    println!("  Hash:    {}", hash);

    Ok(())
}

/// Manifest build
fn run_manifest_build(policy_path: &str, output: Option<String>) -> Result<(), Box<dyn Error>> {
    println!("🔨 Erstelle Manifest...");

    fs::create_dir_all("build")?;
    let mut audit = AuditLog::new("build/agent.audit.jsonl")?;

    // Lade Policy
    let policy = policy::Policy::load(policy_path)?;
    policy.validate()?;
    let policy_info = policy.to_info()?;

    // Lade Commitments
    let commitments = commitment::load_commitments("build/commitments.json")?;

    // Erstelle Manifest
    let manifest =
        manifest::Manifest::build(&commitments, policy_info, "build/agent.audit.jsonl")?;

    // Speichere Manifest
    let output_path = output.unwrap_or_else(|| "build/manifest.json".to_string());
    manifest.save(&output_path)?;

    audit.log_event(
        "manifest_built",
        json!({
            "output": &output_path,
            "policy": &policy.name
        }),
    )?;

    println!("✅ Manifest erstellt: {}", output_path);

    Ok(())
}

/// Proof mock
fn run_proof_mock(policy_path: &str, manifest_path: &str) -> Result<(), Box<dyn Error>> {
    println!("🔬 Generiere Mock-Proof...");

    fs::create_dir_all("build")?;
    let mut audit = AuditLog::new("build/agent.audit.jsonl")?;

    // Lade Policy und Manifest
    let policy = policy::Policy::load(policy_path)?;
    let manifest = manifest::Manifest::load(manifest_path)?;

    // Lade Original-Daten für Count
    let suppliers = io::read_suppliers_csv("../examples/suppliers.csv").unwrap_or_default();
    let ubos = io::read_ubos_csv("../examples/ubos.csv").unwrap_or_default();

    // Generiere Mock-Proof
    let proof = proof_mock::MockProof::generate(&policy, &manifest, suppliers.len(), ubos.len())?;

    // Speichere
    let output_path = "build/proof.mock.json";
    proof.save(output_path)?;

    audit.log_event(
        "mock_proof_generated",
        json!({
            "output": output_path,
            "status": &proof.status
        }),
    )?;

    println!("✅ Mock-Proof generiert: {}", output_path);
    println!("  Status: {}", proof.status);

    Ok(())
}

/// Sign keygen
fn run_sign_keygen(dir: Option<String>) -> Result<(), Box<dyn Error>> {
    let key_dir = dir.unwrap_or_else(|| "keys".to_string());
    fs::create_dir_all(&key_dir)?;

    let priv_path = format!("{}/company.ed25519", key_dir);
    let pub_path = format!("{}/company.pub", key_dir);

    println!("🔑 Generiere Ed25519-Schlüsselpaar...");

    sign::generate_keypair(&priv_path, &pub_path)?;

    println!("✅ Schlüsselpaar generiert:");
    println!("  Private: {}", priv_path);
    println!("  Public:  {}", pub_path);

    Ok(())
}

/// Sign manifest
fn run_sign_manifest(
    key_path: &str,
    manifest_path: &str,
    output: &str,
    signer: Option<String>,
) -> Result<(), Box<dyn Error>> {
    println!("✍️  Signiere Manifest...");

    let mut audit = AuditLog::new("build/agent.audit.jsonl")?;

    // Lade Schlüssel und Manifest
    let signing_key = sign::load_private_key(key_path)?;
    let manifest = manifest::Manifest::load(manifest_path)?;

    // Signiere
    let signer_name = signer.unwrap_or_else(|| "Company".to_string());
    let signed = sign::sign_manifest(&manifest, &signing_key, &signer_name)?;

    // Speichere
    signed.save(output)?;

    audit.log_event(
        "manifest_signed",
        json!({
            "output": output,
            "signer": &signer_name
        }),
    )?;

    println!("✅ Manifest signiert: {}", output);

    Ok(())
}

/// Verify signed manifest
fn run_verify_manifest(pub_key_path: &str, signed_path: &str) -> Result<(), Box<dyn Error>> {
    println!("🔍 Verifiziere signiertes Manifest...");

    let mut audit = AuditLog::new("build/agent.audit.jsonl")?;

    // Lade Public Key und signiertes Manifest
    let verifying_key = sign::load_public_key(pub_key_path)?;
    let signed = manifest::SignedManifest::load(signed_path)?;

    // Verifiziere
    sign::verify_manifest(&signed, &verifying_key)?;

    audit.log_event("manifest_verified", json!({ "file": signed_path }))?;

    println!("✅ Signatur ist gültig!");
    println!("  Signer: {}", signed.signature.signer);

    Ok(())
}

/// Proof build - Erstellt strukturierten Proof
fn run_proof_build(policy_path: &str, manifest_path: &str) -> Result<(), Box<dyn Error>> {
    println!("🔬 Erstelle Proof...");

    fs::create_dir_all("build")?;
    let mut audit = AuditLog::new("build/agent.audit.jsonl")?;

    // Lade Policy und Manifest
    let policy = policy::Policy::load(policy_path)?;
    let manifest = manifest::Manifest::load(manifest_path)?;

    // Lade Original-Daten für Count
    let suppliers = io::read_suppliers_csv("../examples/suppliers.csv").unwrap_or_default();
    let ubos = io::read_ubos_csv("../examples/ubos.csv").unwrap_or_default();

    // Generiere Proof
    let proof = proof_engine::Proof::build(&policy, &manifest, suppliers.len(), ubos.len())?;

    // Speichere als .dat und .json
    let output_path_dat = "build/proof.dat";
    let output_path_json = "build/proof.json";
    proof.save_as_dat(output_path_dat)?;
    proof.save(output_path_json)?;

    audit.log_event(
        "proof_built",
        json!({
            "output_dat": output_path_dat,
            "output_json": output_path_json,
            "status": &proof.status
        }),
    )?;

    println!("✅ Proof erstellt:");
    println!("  - {}", output_path_dat);
    println!("  - {}", output_path_json);
    println!("  Status: {}", proof.status);

    Ok(())
}

/// Proof verify - Verifiziert Proof gegen Manifest
fn run_proof_verify_v3(proof_path: &str, manifest_path: &str) -> Result<(), Box<dyn Error>> {
    println!("🔍 Verifiziere Proof...");

    let mut audit = AuditLog::new("build/agent.audit.jsonl")?;

    // Lade Proof (automatische Erkennung .dat oder .json)
    let proof = if proof_path.ends_with(".dat") {
        proof_engine::Proof::load_from_dat(proof_path)?
    } else {
        proof_engine::Proof::load(proof_path)?
    };

    // Lade Manifest
    let manifest = manifest::Manifest::load(manifest_path)?;

    // Verifiziere
    proof.verify(&manifest)?;

    audit.log_event("proof_verified", json!({ "proof": proof_path }))?;

    println!("✅ Proof ist gültig!");
    println!("  Manifest Hash: {}", proof.manifest_hash);
    println!("  Policy Hash: {}", proof.policy_hash);
    println!("  Status: {}", proof.status);

    Ok(())
}

/// Proof export - Exportiert Proof-Paket
fn run_proof_export(
    manifest_path: &str,
    proof_path: &str,
    signature_path: Option<String>,
    output: Option<String>,
) -> Result<(), Box<dyn Error>> {
    println!("📦 Exportiere Proof-Paket...");

    let mut audit = AuditLog::new("build/agent.audit.jsonl")?;

    // Lade Manifest und Proof
    let manifest = manifest::Manifest::load(manifest_path)?;
    let proof = if proof_path.ends_with(".dat") {
        proof_engine::Proof::load_from_dat(proof_path)?
    } else {
        proof_engine::Proof::load(proof_path)?
    };

    // Output-Verzeichnis
    let output_dir = output.unwrap_or_else(|| "build/proof_package".to_string());

    // Exportiere Paket
    proof_engine::export_proof_package(
        &manifest,
        &proof,
        signature_path.as_deref(),
        &output_dir,
    )?;

    audit.log_event(
        "proof_exported",
        json!({
            "output": &output_dir,
            "has_signature": signature_path.is_some()
        }),
    )?;

    println!("✅ Proof-Paket exportiert: {}", output_dir);

    Ok(())
}

/// ZK Build - Erstellt einen Zero-Knowledge-Proof (Tag 4)
fn run_zk_build(
    policy_path: &str,
    manifest_path: &str,
    output: Option<String>,
    sanctions_root: Option<String>,
    jurisdiction_root: Option<String>,
    sanctions_csv: Option<String>,
) -> Result<(), Box<dyn Error>> {
    println!("🔬 Erstelle Zero-Knowledge-Proof...");

    let mut audit = AuditLog::new("build/agent.audit.jsonl")?;

    // Lade Policy und Manifest
    let policy = policy::Policy::load(policy_path)?;
    let manifest = manifest::Manifest::load(manifest_path)?;

    // Prüfe ob alle geforderten Statement-Roots vorhanden sind
    policy.check_required_statement_roots(&sanctions_root, &jurisdiction_root)?;

    // Lade Commitments für Witness-Daten
    let commitments = Commitments::load("build/commitments.json")?;

    // Lade optional Sanctions-Liste für Mock-Check
    let sanctions_list = if let Some(ref csv_path) = sanctions_csv {
        let (_, entries) = lists::compute_sanctions_root(csv_path)?;
        Some(entries.iter().map(|e| e.hash()).collect())
    } else {
        None
    };

    // Erstelle Statement (öffentliche Daten)
    let statement = zk_system::Statement {
        policy_hash: manifest.policy.hash.clone(),
        company_commitment_root: manifest.company_commitment_root.clone(),
        constraints: vec![
            if policy.constraints.require_at_least_one_ubo {
                "require_at_least_one_ubo".to_string()
            } else {
                String::new()
            },
            format!("supplier_count_max_{}", policy.constraints.supplier_count_max),
        ]
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect(),
        sanctions_root,
        jurisdiction_root,
    };

    // Erstelle Witness (private Daten)
    // In echter Implementation würden hier die tatsächlichen Supplier/UBO-Hashes kommen
    let witness = zk_system::Witness {
        suppliers: vec![manifest.supplier_root.clone()],
        ubos: vec![manifest.ubo_root.clone()],
        supplier_count: commitments.supplier_count.unwrap_or(0),
        ubo_count: commitments.ubo_count.unwrap_or(0),
        sanctions_list,
    };

    // Erstelle ZK-Proof
    let zk = zk_system::SimplifiedZK::new();
    let proof = zk.prove(&statement, &witness)?;

    // Speichere Proof
    let out_dat = output.clone().unwrap_or_else(|| "build/zk_proof.dat".to_string());
    let out_json = out_dat.replace(".dat", ".json");

    zk_system::save_zk_proof_dat(&proof, &out_dat)?;
    zk_system::save_zk_proof_json(&proof, &out_json)?;

    audit.log_event(
        "zk_proof_generated",
        json!({
            "system": proof.system,
            "status": proof.status,
            "policy": policy_path,
            "output": &out_dat
        }),
    )?;

    println!("✅ ZK-Proof erstellt:");
    println!("  - {}", out_dat);
    println!("  - {}", out_json);
    println!("  System: {}", proof.system);
    println!("  Status: {}", proof.status);

    Ok(())
}

/// ZK Verify - Verifiziert einen Zero-Knowledge-Proof
fn run_zk_verify(proof_path: &str) -> Result<(), Box<dyn Error>> {
    println!("🔍 Verifiziere Zero-Knowledge-Proof...");

    let mut audit = AuditLog::new("build/agent.audit.jsonl")?;

    // Lade Proof
    let proof = if proof_path.ends_with(".dat") {
        zk_system::load_zk_proof_dat(proof_path)?
    } else {
        zk_system::load_zk_proof_json(proof_path)?
    };

    // Verifiziere mit passendem Backend
    let is_valid = match proof.system.as_str() {
        "simplified" => {
            let zk = zk_system::SimplifiedZK::new();
            zk.verify(&proof)?
        }
        other => {
            return Err(format!("Unbekanntes ZK-System: {}", other).into());
        }
    };

    audit.log_event(
        "zk_proof_verified",
        json!({
            "proof": proof_path,
            "system": proof.system,
            "valid": is_valid
        }),
    )?;

    if is_valid {
        println!("✅ ZK-Proof ist gültig!");
        println!("  System: {}", proof.system);
        println!("  Policy Hash: {}", proof.public_inputs.policy_hash);
        println!("  Company Root: {}", proof.public_inputs.company_commitment_root);
        println!("  Constraints: {}", proof.public_inputs.constraints.len());
    } else {
        println!("❌ ZK-Proof ist UNGÜLTIG!");
        return Err("Proof-Verifikation fehlgeschlagen".into());
    }

    Ok(())
}

/// ZK Bench - Benchmark für ZK-Proof-Erstellung und Verifikation
fn run_zk_bench(
    policy_path: &str,
    manifest_path: &str,
    iterations: usize,
) -> Result<(), Box<dyn Error>> {
    println!("⏱️  Starte ZK-Proof-Benchmark...");
    println!("   Iterationen: {}", iterations);

    let mut audit = AuditLog::new("build/agent.audit.jsonl")?;

    // Lade Policy und Manifest
    let policy = policy::Policy::load(policy_path)?;
    let manifest = manifest::Manifest::load(manifest_path)?;
    let commitments = Commitments::load("build/commitments.json")?;

    // Bereite Statement und Witness vor
    let statement = zk_system::Statement {
        policy_hash: manifest.policy.hash.clone(),
        company_commitment_root: manifest.company_commitment_root.clone(),
        constraints: vec![
            if policy.constraints.require_at_least_one_ubo {
                "require_at_least_one_ubo".to_string()
            } else {
                String::new()
            },
            format!("supplier_count_max_{}", policy.constraints.supplier_count_max),
        ]
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect(),
        sanctions_root: None,
        jurisdiction_root: None,
    };

    let witness = zk_system::Witness {
        suppliers: vec![manifest.supplier_root.clone()],
        ubos: vec![manifest.ubo_root.clone()],
        supplier_count: commitments.supplier_count.unwrap_or(0),
        ubo_count: commitments.ubo_count.unwrap_or(0),
        sanctions_list: None,
    };

    let zk = zk_system::SimplifiedZK::new();

    // Benchmark Proving
    println!("\n📊 Proving-Benchmark:");
    let prove_start = std::time::Instant::now();
    let mut proofs = Vec::new();
    for i in 0..iterations {
        let proof = zk.prove(&statement, &witness)?;
        proofs.push(proof);
        if (i + 1) % 10 == 0 || i == iterations - 1 {
            println!("   Iteration {}/{} abgeschlossen", i + 1, iterations);
        }
    }
    let prove_duration = prove_start.elapsed();
    let prove_avg = prove_duration / iterations as u32;

    println!("\n  Gesamt: {:?}", prove_duration);
    println!("  Durchschnitt: {:?}", prove_avg);
    println!("  Throughput: {:.2} proofs/s", 1000.0 / prove_avg.as_millis() as f64);

    // Benchmark Verifying
    println!("\n📊 Verify-Benchmark:");
    let verify_start = std::time::Instant::now();
    for (i, proof) in proofs.iter().enumerate() {
        let is_valid = zk.verify(proof)?;
        assert!(is_valid, "Proof {} sollte gültig sein", i);
        if (i + 1) % 10 == 0 || i == iterations - 1 {
            println!("   Iteration {}/{} abgeschlossen", i + 1, iterations);
        }
    }
    let verify_duration = verify_start.elapsed();
    let verify_avg = verify_duration / iterations as u32;

    println!("\n  Gesamt: {:?}", verify_duration);
    println!("  Durchschnitt: {:?}", verify_avg);
    println!("  Throughput: {:.2} verifications/s", 1000.0 / verify_avg.as_millis() as f64);

    audit.log_event(
        "zk_bench_executed",
        json!({
            "iterations": iterations,
            "prove_avg_ms": prove_avg.as_millis(),
            "verify_avg_ms": verify_avg.as_millis(),
            "system": "simplified"
        }),
    )?;

    println!("\n✅ Benchmark abgeschlossen!");

    Ok(())
}

/// Verifier run - Verifiziert Proof-Paket
fn run_verifier_run(package_path: &str) -> Result<(), Box<dyn Error>> {
    println!("🔍 Verifiziere Proof-Paket...");

    let mut audit = AuditLog::new("build/agent.audit.jsonl")?;

    let verifier = verifier::Verifier::new(package_path);

    // Prüfe Integrität
    let integrity = verifier.check_package_integrity()?;
    println!("📋 {}", integrity);

    // Verifiziere
    let result = verifier.verify()?;

    audit.log_event(
        "verifier_run",
        json!({
            "package": package_path,
            "success": result.success,
            "checks_passed": result.checks_passed,
            "checks_total": result.checks_total
        }),
    )?;

    println!("\n✅ Verifikation erfolgreich!");
    println!("  Manifest Hash: {}", result.manifest_hash);
    println!("  Policy Hash: {}", result.policy_hash);
    println!("  Proof Status: {}", result.proof_status);
    println!("  Checks: {}/{}", result.checks_passed, result.checks_total);

    Ok(())
}

/// Verifier extract - Extrahiert Manifest-Infos
fn run_verifier_extract(package_path: &str) -> Result<(), Box<dyn Error>> {
    println!("🔍 Extrahiere Informationen aus Proof-Paket...");

    let summary = verifier::show_package_summary(package_path)?;
    println!("\n{}", summary);

    Ok(())
}

/// Verifier audit - Zeigt Audit-Trail
fn run_verifier_audit(package_path: &str) -> Result<(), Box<dyn Error>> {
    println!("🔍 Zeige Audit-Trail...");

    let verifier = verifier::Verifier::new(package_path);
    let (tail_digest, events_count) = verifier.show_audit_trail()?;

    println!("\n📊 Audit-Trail:");
    println!("  Events gesamt: {}", events_count);
    println!("  Tail Digest:   {}", tail_digest);

    Ok(())
}

/// Audit tip - Schreibt den Audit-Tip in eine Datei
fn run_audit_tip(out: Option<String>) -> Result<(), Box<dyn Error>> {
    println!("📝 Schreibe Audit-Tip...");

    let out_path = out.unwrap_or_else(|| "build/audit.head".to_string());
    let audit_log_path = "build/agent.audit.jsonl";

    // Lade Audit-Log
    let audit = AuditLog::new(audit_log_path)?;

    // Schreibe Tip
    audit.write_tip(&out_path)?;

    println!("✅ Audit-Tip geschrieben nach: {}", out_path);
    println!("   Tip: {}", audit.get_tip());

    Ok(())
}

/// Audit anchor - Setzt einen Zeitanker im Manifest
fn run_audit_anchor(
    kind: &str,
    reference: &str,
    manifest_in: &str,
    manifest_out: &str,
) -> Result<(), Box<dyn Error>> {
    println!("⏰ Setze Zeitanker im Manifest...");

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
    manifest.set_time_anchor(kind.to_string(), reference.to_string(), audit_tip_hex.clone());

    // Speichere Manifest
    manifest.save(manifest_out)?;

    println!("✅ Zeitanker gesetzt:");
    println!("   Kind:           {}", kind);
    println!("   Referenz:       {}", reference);
    println!("   Audit-Tip:      {}", audit_tip_hex);
    println!("   Output:         {}", manifest_out);

    Ok(())
}

/// Audit timestamp - Erstellt einen Timestamp für den Audit-Head
fn run_audit_timestamp(
    head_path: &str,
    output: Option<String>,
    is_mock: bool,
    tsa_url: Option<String>,
) -> Result<(), Box<dyn Error>> {
    println!("⏰ Erstelle Timestamp für Audit-Head...");

    // Lade Audit-Tip
    let audit_tip_hex = std::fs::read_to_string(head_path)?;
    let audit_tip_hex = audit_tip_hex.trim().to_string();

    // Erstelle Timestamp
    let timestamp = if is_mock {
        println!("   ⚠️  MOCK TIMESTAMP (nicht für Produktion geeignet)");
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
    let out_path = output.unwrap_or_else(|| "build/timestamp.tsr".to_string());
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

    println!("✅ Timestamp erstellt:");
    println!("   Audit-Tip:      {}", timestamp.audit_tip_hex);
    println!("   Erstellt:       {}", timestamp.created_at);
    println!("   TSA:            {}", timestamp.tsa);
    println!("   Output:         {}", out_path);

    Ok(())
}

/// Audit verify-timestamp - Verifiziert einen Timestamp gegen Audit-Head
fn run_audit_verify_timestamp(
    head_path: &str,
    timestamp_path: &str,
) -> Result<(), Box<dyn Error>> {
    println!("🔍 Verifiziere Timestamp...");

    // Lade Audit-Tip
    let audit_tip_hex = std::fs::read_to_string(head_path)?;
    let audit_tip_hex = audit_tip_hex.trim();

    // Lade Timestamp
    let timestamp = registry::Timestamp::load(timestamp_path)?;

    // Verifiziere
    if timestamp.verify(audit_tip_hex) {
        println!("✅ Timestamp valid");
        println!("   Audit-Tip:      {}", timestamp.audit_tip_hex);
        println!("   Erstellt:       {}", timestamp.created_at);
        println!("   TSA:            {}", timestamp.tsa);

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
        println!("❌ Timestamp invalid or mismatched head");
        println!("   Erwartet:       {}", audit_tip_hex);
        println!("   Gefunden:       {}", timestamp.audit_tip_hex);
        Err("Timestamp-Verifikation fehlgeschlagen".into())
    }
}

/// Lists sanctions-root - Generiert Sanctions Merkle Root
fn run_lists_sanctions_root(csv_path: &str, output: Option<String>) -> Result<(), Box<dyn Error>> {
    println!("📋 Generiere Sanctions Merkle Root...");

    let mut audit = AuditLog::new("build/agent.audit.jsonl")?;

    // Berechne Root aus CSV
    let (root, entries) = lists::compute_sanctions_root(csv_path)?;

    let out_path = output.unwrap_or_else(|| "build/sanctions.root".to_string());

    // Erstelle Root-Info
    let info = lists::SanctionsRootInfo {
        root: root.clone(),
        count: entries.len(),
        source: csv_path.to_string(),
        generated_at: chrono::Utc::now().to_rfc3339(),
        algorithm: "BLAKE3".to_string(),
    };

    // Speichere Root-Info
    lists::save_sanctions_root_info(&info, &out_path)?;

    // Audit-Event
    audit.log_event(
        "sanctions_root_generated",
        json!({
            "root": root,
            "count": entries.len(),
            "source": csv_path,
            "output": out_path
        }),
    )?;

    println!("✅ Sanctions Root generiert:");
    println!("   Root:   {}", root);
    println!("   Count:  {}", entries.len());
    println!("   Output: {}", out_path);

    Ok(())
}

/// Lists jurisdictions-root - Generiert Jurisdictions Merkle Root
fn run_lists_jurisdictions_root(csv_path: &str, output: Option<String>) -> Result<(), Box<dyn Error>> {
    println!("🌍 Generiere Jurisdictions Merkle Root...");

    let mut audit = AuditLog::new("build/agent.audit.jsonl")?;

    // Berechne Root aus CSV
    let (root, entries) = lists::compute_jurisdictions_root(csv_path)?;

    let out_path = output.unwrap_or_else(|| "build/jurisdictions.root".to_string());

    // Erstelle Root-Info
    let info = lists::JurisdictionsRootInfo {
        root: root.clone(),
        count: entries.len(),
        source: csv_path.to_string(),
        generated_at: chrono::Utc::now().to_rfc3339(),
        algorithm: "BLAKE3".to_string(),
    };

    // Speichere Root-Info
    lists::save_jurisdictions_root_info(&info, &out_path)?;

    // Audit-Event
    audit.log_event(
        "jurisdictions_root_generated",
        json!({
            "root": root,
            "count": entries.len(),
            "source": csv_path,
            "output": out_path
        }),
    )?;

    println!("✅ Jurisdictions Root generiert:");
    println!("   Root:   {}", root);
    println!("   Count:  {}", entries.len());
    println!("   Output: {}", out_path);

    Ok(())
}

/// Registry add - Fügt einen Proof zur Registry hinzu
fn run_registry_add(
    manifest_path: &str,
    proof_path: &str,
    timestamp_path: Option<String>,
    registry_path: Option<String>,
) -> Result<(), Box<dyn Error>> {
    println!("📝 Füge Proof zur Registry hinzu...");

    let registry_file = registry_path.unwrap_or_else(|| "build/registry.json".to_string());

    // Berechne Hashes
    let manifest_hash = registry::compute_file_hash(manifest_path)?;
    let proof_hash = registry::compute_file_hash(proof_path)?;

    println!("   Manifest-Hash:  {}", manifest_hash);
    println!("   Proof-Hash:     {}", proof_hash);

    // Lade oder erstelle Registry
    let mut registry = if std::path::Path::new(&registry_file).exists() {
        registry::Registry::load(&registry_file)?
    } else {
        registry::Registry::new()
    };

    // Füge Eintrag hinzu
    let id = registry.add_entry(manifest_hash.clone(), proof_hash.clone(), timestamp_path.clone());

    // Speichere Registry
    registry.save(&registry_file)?;

    // Log Audit-Event
    let mut audit = AuditLog::new("build/agent.audit.jsonl")?;
    audit.log_event(
        "registry_entry_added",
        json!({
            "id": id,
            "manifest_hash": manifest_hash,
            "proof_hash": proof_hash,
            "timestamp_file": timestamp_path,
            "registry_file": registry_file
        }),
    )?;

    println!("✅ Proof zur Registry hinzugefügt:");
    println!("   ID:             {}", id);
    println!("   Registry:       {}", registry_file);
    println!("   Einträge total: {}", registry.count());

    Ok(())
}

/// Registry list - Listet alle Registry-Einträge auf
fn run_registry_list(registry_path: Option<String>) -> Result<(), Box<dyn Error>> {
    let registry_file = registry_path.unwrap_or_else(|| "build/registry.json".to_string());

    if !std::path::Path::new(&registry_file).exists() {
        println!("⚠️  Registry-Datei nicht gefunden: {}", registry_file);
        println!("   Verwende 'registry add' um Einträge hinzuzufügen.");
        return Ok(());
    }

    let registry = registry::Registry::load(&registry_file)?;

    println!("──────────────────────────────────────────────");
    println!("Proofs in local registry ({})", registry_file);
    println!("──────────────────────────────────────────────");

    if registry.entries.is_empty() {
        println!("   (keine Einträge)");
    } else {
        for (idx, entry) in registry.entries.iter().enumerate() {
            println!(
                "#{:<3} Manifest: {}…  Proof: {}…  Date: {}",
                idx + 1,
                &entry.manifest_hash[..12],
                &entry.proof_hash[..12],
                entry.registered_at
            );
            if let Some(ref ts) = entry.timestamp_file {
                println!("     Timestamp: {}", ts);
            }
        }
    }

    println!("──────────────────────────────────────────────");
    println!("Total: {} Einträge", registry.count());

    Ok(())
}

/// Registry verify - Verifiziert einen Proof gegen die Registry
fn run_registry_verify(
    manifest_path: &str,
    proof_path: &str,
    registry_path: Option<String>,
) -> Result<(), Box<dyn Error>> {
    println!("🔍 Verifiziere Proof gegen Registry...");

    let registry_file = registry_path.unwrap_or_else(|| "build/registry.json".to_string());

    if !std::path::Path::new(&registry_file).exists() {
        println!("❌ Registry-Datei nicht gefunden: {}", registry_file);
        return Err("Registry existiert nicht".into());
    }

    // Berechne Hashes
    let manifest_hash = registry::compute_file_hash(manifest_path)?;
    let proof_hash = registry::compute_file_hash(proof_path)?;

    println!("   Manifest-Hash:  {}", manifest_hash);
    println!("   Proof-Hash:     {}", proof_hash);

    // Lade Registry
    let registry = registry::Registry::load(&registry_file)?;

    // Verifiziere
    if registry.verify_entry(&manifest_hash, &proof_hash) {
        let entry = registry.find_entry(&manifest_hash, &proof_hash).unwrap();
        println!("✅ Entry verified in registry");
        println!("   ID:             {}", entry.id);
        println!("   Registered:     {}", entry.registered_at);
        if let Some(ref ts) = entry.timestamp_file {
            println!("   Timestamp:      {}", ts);
        }

        // Log Audit-Event
        let mut audit = AuditLog::new("build/agent.audit.jsonl")?;
        audit.log_event(
            "registry_verified",
            json!({
                "manifest_hash": manifest_hash,
                "proof_hash": proof_hash,
                "registry_file": registry_file,
                "status": "ok"
            }),
        )?;

        Ok(())
    } else {
        println!("❌ Hash mismatch or not registered");
        println!("   Registry:       {}", registry_file);
        println!("   Einträge:       {}", registry.count());
        Err("Proof nicht in Registry gefunden".into())
    }
}

/// Zeigt die Version an
fn run_version() {
    println!("cap-agent v{}", VERSION);
}

fn main() {
    let cli = Cli::parse();

    let result = match &cli.command {
        Commands::Prepare { suppliers, ubos } => run_prepare(suppliers, ubos),
        Commands::Inspect { path } => run_inspect(path),
        Commands::Policy(cmd) => match cmd {
            PolicyCommands::Validate { file } => run_policy_validate(file),
        },
        Commands::Manifest(cmd) => match cmd {
            ManifestCommands::Build { policy, out } => {
                run_manifest_build(policy, out.clone())
            }
        },
        Commands::Proof(cmd) => match cmd {
            ProofCommands::Mock { policy, manifest } => run_proof_mock(policy, manifest),
            ProofCommands::Build { policy, manifest } => run_proof_build(policy, manifest),
            ProofCommands::Verify { proof, manifest } => run_proof_verify_v3(proof, manifest),
            ProofCommands::Export {
                manifest,
                proof,
                signature,
                out,
            } => run_proof_export(manifest, proof, signature.clone(), out.clone()),
            ProofCommands::ZkBuild {
                policy,
                manifest,
                out,
                sanctions_root,
                jurisdiction_root,
                sanctions_csv,
            } => run_zk_build(policy, manifest, out.clone(), sanctions_root.clone(), jurisdiction_root.clone(), sanctions_csv.clone()),
            ProofCommands::ZkVerify { proof } => run_zk_verify(proof),
            ProofCommands::Bench {
                policy,
                manifest,
                iterations,
            } => run_zk_bench(policy, manifest, *iterations),
        },
        Commands::Sign(cmd) => match cmd {
            SignCommands::Keygen { dir } => run_sign_keygen(dir.clone()),
            SignCommands::Manifest {
                key,
                manifest_in,
                out,
                signer,
            } => run_sign_manifest(key, manifest_in, out, signer.clone()),
            SignCommands::VerifyManifest { pub_key, signed_in } => {
                run_verify_manifest(pub_key, signed_in)
            }
        },
        Commands::Verifier(cmd) => match cmd {
            VerifierCommands::Run { package } => run_verifier_run(package),
            VerifierCommands::Extract { package } => run_verifier_extract(package),
            VerifierCommands::Audit { package } => run_verifier_audit(package),
        },
        Commands::Audit(cmd) => match cmd {
            AuditCommands::Tip { out } => run_audit_tip(out.clone()),
            AuditCommands::Anchor {
                kind,
                reference,
                manifest_in,
                manifest_out,
            } => run_audit_anchor(kind, reference, manifest_in, manifest_out),
            AuditCommands::Timestamp {
                head,
                out,
                mock,
                tsa_url,
            } => run_audit_timestamp(head, out.clone(), *mock, tsa_url.clone()),
            AuditCommands::VerifyTimestamp { head, timestamp } => {
                run_audit_verify_timestamp(head, timestamp)
            }
        },
        Commands::Lists(cmd) => match cmd {
            ListsCommands::SanctionsRoot { csv, out } => run_lists_sanctions_root(csv, out.clone()),
            ListsCommands::JurisdictionsRoot { csv, out } => run_lists_jurisdictions_root(csv, out.clone()),
        },
        Commands::Registry(cmd) => match cmd {
            RegistryCommands::Add {
                manifest,
                proof,
                timestamp,
                registry,
            } => run_registry_add(manifest, proof, timestamp.clone(), registry.clone()),
            RegistryCommands::List { registry } => run_registry_list(registry.clone()),
            RegistryCommands::Verify {
                manifest,
                proof,
                registry,
            } => run_registry_verify(manifest, proof, registry.clone()),
        },
        Commands::Version => {
            run_version();
            Ok(())
        }
    };

    if let Err(e) = result {
        eprintln!("❌ Fehler: {}", e);
        std::process::exit(1);
    }
}
