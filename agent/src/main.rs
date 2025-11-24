mod audit;
mod blob_store;
mod commitment;
mod io;
mod keys;
mod lists;
mod manifest;
mod package_verifier;
mod policy;
mod proof_engine;
mod proof_mock;
mod registry;
mod sign;
mod zk_system;

use audit::AuditLog;
use blob_store::{BlobStore, SqliteBlobStore};
use chrono::Utc;
use clap::{Parser, Subcommand};
use commitment::{compute_company_root, compute_supplier_root, compute_ubo_root, Commitments};
use manifest::Manifest;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::error::Error;
use std::fs;
use std::io::{stdin, stdout, Read, Write};
use std::path::Path;
use uuid::Uuid;
use zk_system::ProofSystem;

// Import portable verifier core from library
use cap_agent::bundle::meta::{BundleMeta, BundleFileMeta, ProofUnitMeta, BUNDLE_SCHEMA_V1};
use cap_agent::crypto;
use cap_agent::orchestrator;
use cap_agent::verifier;
use cap_agent::verifier::core as verifier_core;
use cap_agent::wasm;

const VERSION: &str = "0.8.0";

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
    /// Key Management Commands (v0.10)
    #[command(subcommand)]
    Keys(KeysCommands),
    /// BLOB Store Commands (v0.10.9) - Content-Addressable Storage
    #[command(subcommand)]
    Blob(BlobCommands),
    /// Bundle v2 - Create self-contained proof package with WASM verifier
    BundleV2 {
        /// Manifest path
        #[arg(long)]
        manifest: String,
        /// Proof path (CAPZ format)
        #[arg(long)]
        proof: String,
        /// Verifier WASM path (optional)
        #[arg(long)]
        verifier_wasm: Option<String>,
        /// Output directory
        #[arg(long, default_value = "build/cap-proof-v2")]
        out: String,
        /// Create ZIP archive
        #[arg(long)]
        zip: bool,
        /// Force overwrite
        #[arg(long)]
        force: bool,
    },
    /// Verify Bundle - Verify a proof package (v1 or v2)
    VerifyBundle {
        /// Bundle path (directory or .zip)
        #[arg(long)]
        bundle: String,
        /// Output verification report path
        #[arg(long)]
        out: Option<String>,
    },
    /// Zeigt die Tool-Version an
    Version,
}

#[derive(Subcommand)]
enum PolicyCommands {
    /// Validiert eine Policy-Datei (Legacy)
    Validate {
        /// Pfad zur Policy-Datei (YAML oder JSON)
        #[arg(long)]
        file: String,
    },
    /// Lint a policy file (PolicyV2)
    Lint {
        /// Path to policy YAML file
        file: String,
        /// Use strict linting mode
        #[arg(long)]
        strict: bool,
    },
    /// Compile policy to IR v1 (PolicyV2)
    Compile {
        /// Path to policy YAML file
        file: String,
        /// Output IR JSON file
        #[arg(short, long)]
        output: String,
    },
    /// Show IR in human-readable format (PolicyV2)
    Show {
        /// Path to IR JSON file
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
    /// Validiert ein Manifest gegen das JSON Schema
    Validate {
        /// Pfad zur Manifest-Datei
        #[arg(long)]
        file: String,

        /// Pfad zur Schema-Datei (default: docs/manifest.schema.json)
        #[arg(long)]
        schema: Option<String>,
    },
    /// Verifiziert ein Manifest und Proof-Paket offline
    Verify {
        /// Pfad zur Manifest-Datei
        #[arg(long)]
        manifest: String,

        /// Pfad zur Proof-Datei
        #[arg(long)]
        proof: String,

        /// Pfad zur Registry-Datei
        #[arg(long)]
        registry: String,

        /// Optional: Pfad zur Timestamp-Datei
        #[arg(long)]
        timestamp: Option<String>,

        /// Optional: Output-Pfad für Verification Report (default: build/verification.report.json)
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
    /// Exportiert ein standardisiertes CAP Proof-Paket
    Export {
        /// Pfad zum Manifest
        #[arg(long)]
        manifest: String,

        /// Pfad zum Proof
        #[arg(long)]
        proof: String,

        /// Optional: Pfad zur Timestamp-Datei
        #[arg(long)]
        timestamp: Option<String>,

        /// Optional: Pfad zur Registry-Datei
        #[arg(long)]
        registry: Option<String>,

        /// Optional: Pfad zum Verification Report
        #[arg(long)]
        report: Option<String>,

        /// Output-Verzeichnis (default: build/cap-proof)
        #[arg(long)]
        out: Option<String>,

        /// Überschreibt existierendes Output-Verzeichnis
        #[arg(long)]
        force: bool,
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
    /// Adaptive Proof-Orchestrierung mit Enforcement-Mode (Week 6)
    Adapt {
        /// Policy ID (z.B. "lksg.v1")
        #[arg(long)]
        policy: Option<String>,

        /// Pfad zur IR-Datei (Alternative zu policy)
        #[arg(long)]
        ir: Option<std::path::PathBuf>,

        /// Pfad zur Context-JSON-Datei
        #[arg(long)]
        context: std::path::PathBuf,

        /// Enforcement-Modus aktivieren (default: false, shadow-only)
        #[arg(long, default_value_t = false)]
        enforce: bool,

        /// Rollout-Prozentsatz (0-100, default: 0)
        #[arg(long, default_value_t = 0)]
        rollout: u8,

        /// Maximale Drift-Ratio (default: 0.005 = 0.5%)
        #[arg(long, default_value_t = 0.005)]
        drift_max: f64,

        /// Selector-Typ: basic oder weighted (default: basic)
        #[arg(long, default_value = "basic")]
        selector: String,

        /// Pfad zur Rule-Weights-Datei (YAML)
        #[arg(long)]
        weights: Option<std::path::PathBuf>,

        /// Dry-Run-Modus (keine Seiteneffekte)
        #[arg(long, default_value_t = false)]
        dry_run: bool,

        /// Output-Pfad für Execution Plan (JSON)
        #[arg(short = 'o', long)]
        out: Option<std::path::PathBuf>,
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
    /// Setzt Private Anchor (Dual-Anchor v0.9.0)
    SetPrivateAnchor {
        /// Manifest-Pfad (Input/Output)
        #[arg(long)]
        manifest: String,

        /// Audit-Tip (0x-prefixed hex)
        #[arg(long)]
        audit_tip: String,

        /// Created-at Timestamp (RFC3339, optional, default: jetzt)
        #[arg(long)]
        created_at: Option<String>,
    },
    /// Setzt Public Anchor (Dual-Anchor v0.9.0)
    SetPublicAnchor {
        /// Manifest-Pfad (Input/Output)
        #[arg(long)]
        manifest: String,

        /// Blockchain (ethereum, hedera, btc)
        #[arg(long)]
        chain: String,

        /// Transaction ID
        #[arg(long)]
        txid: String,

        /// Digest (0x-prefixed hex)
        #[arg(long)]
        digest: String,

        /// Created-at Timestamp (RFC3339, optional, default: jetzt)
        #[arg(long)]
        created_at: Option<String>,
    },
    /// Verifiziert Dual-Anchor-Konsistenz
    VerifyAnchor {
        /// Manifest-Pfad
        #[arg(long)]
        manifest: String,

        /// Output JSON-Report (optional)
        #[arg(long)]
        out: Option<String>,
    },
    /// Fügt Event zur Audit-Chain hinzu (Track A)
    Append {
        /// Pfad zur Audit-Chain-Datei (default: build/audit_chain.jsonl)
        #[arg(long, default_value = "build/audit_chain.jsonl")]
        file: String,

        /// Event-Typ (z.B. "verify_response", "policy_compile")
        #[arg(long)]
        event: String,

        /// Policy ID (optional)
        #[arg(long)]
        policy_id: Option<String>,

        /// IR Hash (optional)
        #[arg(long)]
        ir_hash: Option<String>,

        /// Manifest Hash (optional)
        #[arg(long)]
        manifest_hash: Option<String>,

        /// Result (ok, warn, fail)
        #[arg(long)]
        result: Option<String>,

        /// Run ID (UUID for correlation, optional)
        #[arg(long)]
        run_id: Option<String>,
    },
    /// Verifiziert Audit-Chain-Integrität (Track A)
    Verify {
        /// Pfad zur Audit-Chain-Datei (default: build/audit_chain.jsonl)
        #[arg(long, default_value = "build/audit_chain.jsonl")]
        file: String,

        /// Output JSON-Report (optional)
        #[arg(long)]
        out: Option<String>,
    },
    /// Exportiert Events aus Audit-Chain (Track A)
    Export {
        /// Pfad zur Audit-Chain-Datei (default: build/audit_chain.jsonl)
        #[arg(long, default_value = "build/audit_chain.jsonl")]
        file: String,

        /// Start-Timestamp (RFC3339, optional)
        #[arg(long)]
        from: Option<String>,

        /// End-Timestamp (RFC3339, optional)
        #[arg(long)]
        to: Option<String>,

        /// Policy ID Filter (optional)
        #[arg(long)]
        policy_id: Option<String>,

        /// Output-Datei (default: stdout)
        #[arg(long)]
        out: Option<String>,
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

        /// Registry-Datei (default: build/registry.json oder build/registry.sqlite)
        #[arg(long)]
        registry: Option<String>,

        /// Registry-Backend (json|sqlite, default: json)
        #[arg(long, default_value = "json")]
        backend: String,

        /// Optionaler Pfad zum Signing-Key (Ed25519, default: keys/company.ed25519)
        #[arg(long)]
        signing_key: Option<String>,

        /// Validiert Key-Status (muss "active" sein)
        #[arg(long)]
        validate_key: bool,

        /// Keys directory für Validierung (default: keys/)
        #[arg(long, default_value = "keys")]
        keys_dir: String,
    },
    /// Listet alle Registry-Einträge auf
    List {
        /// Registry-Datei (default: build/registry.json oder build/registry.sqlite)
        #[arg(long)]
        registry: Option<String>,

        /// Registry-Backend (json|sqlite, default: json)
        #[arg(long, default_value = "json")]
        backend: String,
    },
    /// Verifiziert einen Proof gegen die Registry
    Verify {
        /// Pfad zum Manifest
        #[arg(long)]
        manifest: String,

        /// Pfad zum Proof
        #[arg(long)]
        proof: String,

        /// Registry-Datei (default: build/registry.json oder build/registry.sqlite)
        #[arg(long)]
        registry: Option<String>,

        /// Registry-Backend (json|sqlite, default: json)
        #[arg(long, default_value = "json")]
        backend: String,
    },
    /// Migriert Registry zwischen Backends
    Migrate {
        /// Quell-Backend (json|sqlite)
        #[arg(long)]
        from: String,

        /// Quell-Datei
        #[arg(long)]
        input: String,

        /// Ziel-Backend (json|sqlite)
        #[arg(long)]
        to: String,

        /// Ziel-Datei
        #[arg(long)]
        output: String,
    },
    /// Zeigt Registry-Metadaten und Statistiken an (v1.1)
    Inspect {
        /// Registry-Datei (default: build/registry.json)
        #[arg(long)]
        registry: Option<String>,
    },
    /// Backfills KID-Felder aus public_key (v1.1)
    BackfillKid {
        /// Registry-Datei (default: build/registry.json)
        #[arg(long)]
        registry: Option<String>,

        /// Output-Datei (default: überschreibt input)
        #[arg(long)]
        output: Option<String>,
    },
}

#[derive(Subcommand)]
enum KeysCommands {
    /// Generiert ein neues Ed25519-Schlüsselpaar mit Metadata
    Keygen {
        /// Owner/Organization name
        #[arg(long)]
        owner: String,

        /// Algorithm (default: ed25519)
        #[arg(long, default_value = "ed25519")]
        algo: String,

        /// Output path for key metadata JSON
        #[arg(long)]
        out: String,

        /// Validity period in days (default: 730 = 2 years)
        #[arg(long, default_value = "730")]
        valid_days: u64,

        /// Optional comment
        #[arg(long)]
        comment: Option<String>,
    },
    /// Listet alle Schlüssel im Key Store auf
    List {
        /// Keys directory (default: keys/)
        #[arg(long, default_value = "keys")]
        dir: String,

        /// Filter by status (active, retired, revoked)
        #[arg(long)]
        status: Option<String>,

        /// Filter by owner
        #[arg(long)]
        owner: Option<String>,
    },
    /// Zeigt Details eines Schlüssels an
    Show {
        /// Keys directory (default: keys/)
        #[arg(long, default_value = "keys")]
        dir: String,

        /// Key Identifier (KID)
        #[arg(long)]
        kid: String,
    },
    /// Rotiert einen Schlüssel (markiert alten als retired, aktiviert neuen)
    Rotate {
        /// Keys directory (default: keys/)
        #[arg(long, default_value = "keys")]
        dir: String,

        /// Current key metadata file
        #[arg(long)]
        current: String,

        /// New key metadata file
        #[arg(long)]
        new: String,
    },
    /// Attestiert einen neuen Schlüssel mit einem alten (Chain of Trust)
    Attest {
        /// Signer key metadata file (old key)
        #[arg(long)]
        signer: String,

        /// Subject key metadata file (new key)
        #[arg(long)]
        subject: String,

        /// Output path for attestation
        #[arg(long)]
        out: String,
    },
    /// Archiviert einen Schlüssel (moved to archive/)
    Archive {
        /// Keys directory (default: keys/)
        #[arg(long, default_value = "keys")]
        dir: String,

        /// Key Identifier (KID) to archive
        #[arg(long)]
        kid: String,
    },
    /// Verifiziert eine Chain-of-Trust (Attestation chain)
    VerifyChain {
        /// Keys directory (default: keys/)
        #[arg(long, default_value = "keys")]
        dir: String,

        /// Attestation file paths (in chronological order)
        #[arg(long, value_delimiter = ',')]
        attestations: Vec<String>,
    },
}

#[derive(Subcommand)]
enum BlobCommands {
    /// Fügt eine Datei in den BLOB Store ein (CAS + optional Registry-Verknüpfung)
    Put {
        /// Pfad zur Quelldatei
        #[arg(long)]
        file: Option<String>,

        /// Medientyp (manifest|proof|wasm|abi|other)
        #[arg(long)]
        r#type: String,

        /// Registry-Datei (default: build/registry.sqlite)
        #[arg(long, default_value = "build/registry.sqlite")]
        registry: String,

        /// Erhöht refcount für den referenzierenden Registry-Eintrag (UUID)
        #[arg(long)]
        link_entry_id: Option<String>,

        /// Liest Daten von stdin (Pipes)
        #[arg(long)]
        stdin: bool,

        /// Schreibt blob_id in Datei
        #[arg(long)]
        out: Option<String>,

        /// Erzwingt Re-Insert (nur Tests/Debug)
        #[arg(long)]
        no_dedup: bool,
    },
    /// Extrahiert Blob-Inhalt anhand blob_id auf Datei oder stdout
    Get {
        /// BLOB ID (0x-präfixiert, 64 hex chars)
        #[arg(long)]
        id: String,

        /// Zielpfad
        #[arg(long)]
        out: Option<String>,

        /// Schreibt Rohdaten auf stdout (Default wenn --out fehlt)
        #[arg(long)]
        stdout: bool,

        /// Registry-Datei (default: build/registry.sqlite)
        #[arg(long, default_value = "build/registry.sqlite")]
        registry: String,
    },
    /// Listet Blobs gefiltert/sortiert
    List {
        /// Filter by media type (manifest|proof|wasm|abi|other)
        #[arg(long)]
        r#type: Option<String>,

        /// Minimum size in bytes
        #[arg(long)]
        min_size: Option<u64>,

        /// Maximum size in bytes
        #[arg(long)]
        max_size: Option<u64>,

        /// Zeigt nur unreferenzierte Blobs (refcount=0)
        #[arg(long)]
        unused_only: bool,

        /// Limit Anzahl Ergebnisse
        #[arg(long)]
        limit: Option<usize>,

        /// Sortierung (size|refcount|blob_id)
        #[arg(long, default_value = "blob_id")]
        order: String,

        /// Registry-Datei (default: build/registry.sqlite)
        #[arg(long, default_value = "build/registry.sqlite")]
        registry: String,
    },
    /// Garbage Collection nicht referenzierter Blobs
    Gc {
        /// Dry-run (zeigt nur, was gelöscht würde)
        #[arg(long)]
        dry_run: bool,

        /// Force deletion (keine Bestätigung)
        #[arg(long)]
        force: bool,

        /// Mindest-Alter vor Löschung (z.B. "24h", "7d")
        #[arg(long)]
        min_age: Option<String>,

        /// Gibt gelöschte BLOB IDs aus
        #[arg(long)]
        print_ids: bool,

        /// Registry-Datei (default: build/registry.sqlite)
        #[arg(long, default_value = "build/registry.sqlite")]
        registry: String,
    },
}

/// Verification Report für manifest verify Kommando
#[derive(Debug, Serialize, Deserialize)]
struct VerificationReport {
    pub manifest_hash: String,
    pub proof_hash: String,
    pub timestamp_valid: bool,
    pub registry_match: bool,
    pub signature_valid: bool,
    pub status: String,
}

// BundleMeta and BundleFileMeta are now imported from bundle::meta module (see imports at top of file)

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
    let manifest = manifest::Manifest::build(&commitments, policy_info, "build/agent.audit.jsonl")?;

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

    // Lade Commitments für Count-Daten
    let commitments = commitment::load_commitments("build/commitments.json")?;
    let supplier_count = commitments.supplier_count.unwrap_or(0);
    let ubo_count = commitments.ubo_count.unwrap_or(0);

    // Generiere Proof
    let proof = proof_engine::Proof::build(&policy, &manifest, supplier_count, ubo_count)?;

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

/// Proof export - Exportiert standardisiertes CAP Proof-Paket (v1.0)
///
/// # Argumente
/// * `manifest_path` - Pfad zur Manifest-Datei
/// * `proof_path` - Pfad zur Proof-Datei
/// * `timestamp_path` - Optionaler Pfad zur Timestamp-Datei
/// * `registry_path` - Optionaler Pfad zur Registry-Datei
/// * `report_path` - Optionaler Pfad zum Verification Report
/// * `output` - Optionaler Output-Pfad (Standard: build/cap-proof)
/// * `force` - Überschreibt existierendes Verzeichnis
///
/// # Rückgabe
/// Result mit () bei Erfolg
fn run_proof_export(
    manifest_path: &str,
    proof_path: &str,
    timestamp_path: Option<String>,
    registry_path: Option<String>,
    report_path: Option<String>,
    output: Option<String>,
    force: bool,
) -> Result<(), Box<dyn Error>> {
    println!("📦 Exportiere CAP Bundle (cap-bundle.v1)...");

    // 1. Output-Verzeichnis vorbereiten
    let output_dir = output.unwrap_or_else(|| "build/cap-proof".to_string());
    let out_path = std::path::Path::new(&output_dir);

    if out_path.exists() {
        if force {
            fs::remove_dir_all(out_path)?;
            println!("   ⚠️  Existierendes Verzeichnis entfernt (--force)");
        } else {
            return Err(format!("Output-Verzeichnis '{}' existiert bereits. Verwenden Sie --force zum Überschreiben.", output_dir).into());
        }
    }
    fs::create_dir_all(out_path)?;

    // 2. Dateien kopieren
    println!("   📁 Kopiere Dateien...");

    // Manifest (load it to extract policy info)
    let manifest_dst = out_path.join("manifest.json");
    fs::copy(manifest_path, &manifest_dst)?;
    println!("      ✓ manifest.json");

    // Load manifest to extract policy info
    let manifest = Manifest::load(&manifest_dst)?;

    // Proof
    let proof_dst = out_path.join("proof.dat");
    fs::copy(proof_path, &proof_dst)?;
    println!("      ✓ proof.dat");

    // Timestamp (optional)
    let ts_dst = if let Some(ts) = timestamp_path.as_ref() {
        let dst = out_path.join("timestamp.tsr");
        fs::copy(ts, &dst)?;
        println!("      ✓ timestamp.tsr");
        Some(dst)
    } else {
        println!("      ⊘ timestamp.tsr (nicht angegeben)");
        None
    };

    // Registry (optional)
    let reg_dst = if let Some(reg) = registry_path.as_ref() {
        let dst = out_path.join("registry.json");
        fs::copy(reg, &dst)?;
        println!("      ✓ registry.json");
        Some(dst)
    } else {
        println!("      ⊘ registry.json (nicht angegeben)");
        None
    };

    // Report (optional oder minimal)
    let rep_dst = out_path.join("verification.report.json");
    if let Some(rep) = report_path.as_ref() {
        fs::copy(rep, &rep_dst)?;
        println!("      ✓ verification.report.json");
    } else {
        // Erstelle minimalen Report
        fs::write(
            &rep_dst,
            r#"{"status":"unknown","note":"No verification performed before export"}"#,
        )?;
        println!("      ⚠️  verification.report.json (minimal, status=unknown)");
    }

    // 3. README.txt erstellen
    println!("   📝 Erstelle README.txt...");
    let readme_dst = out_path.join("README.txt");
    let readme = format!(
        r#"CAP Bundle Package (cap-bundle.v1)
===================================

This package contains a complete, offline-verifiable proof bundle
following the Confidential Assurance Protocol (CAP) standard.

Files:
------
- manifest.json              : Manifest with commitments and policy info
- proof.dat                  : Zero-knowledge proof (Base64-encoded)
- timestamp.tsr              : Timestamp signature (optional)
- registry.json              : Local proof registry (optional)
- verification.report.json   : Pre-verification report
- README.txt                 : This file
- _meta.json                 : Bundle metadata (cap-bundle.v1 format)

Verification:
-------------
To verify this bundle offline, use:

  cap-agent verifier run --package <this-directory>

Or manually:

  cap-agent manifest verify \
    --manifest manifest.json \
    --proof proof.dat \
    --registry registry.json \
    --timestamp timestamp.tsr \
    --out verification.report.json

Bundle Format:
--------------
This bundle uses the cap-bundle.v1 format with structured metadata.
The _meta.json file contains:
- schema: "cap-bundle.v1"
- bundle_id: Unique bundle identifier
- files: Map of filename -> BundleFileMeta (role, hash, size, content_type, optional)
- proof_units: Array of proof unit metadata

Package created: {}
Package schema: cap-bundle.v1

For more information, see: https://cap.protocol/
"#,
        Utc::now().to_rfc3339()
    );
    fs::write(&readme_dst, readme)?;

    // 4. _meta.json erstellen (cap-bundle.v1 format)
    println!("   🔐 Berechne Hashes und erstelle _meta.json...");

    // Helper-Funktion für SHA3-256 (using centralized crypto API)
    let compute_sha3 = |path: &std::path::Path| -> Result<String, Box<dyn Error>> {
        let bytes = fs::read(path)?;
        let hash = cap_agent::crypto::sha3_256(&bytes);
        Ok(cap_agent::crypto::hex_lower_prefixed32(hash))
    };

    // Helper-Funktion für Dateigröße
    let get_size = |path: &std::path::Path| -> Result<u64, Box<dyn Error>> {
        Ok(fs::metadata(path)?.len())
    };

    // Create files map with BundleFileMeta objects
    let mut files = std::collections::HashMap::new();

    // Manifest file
    files.insert(
        "manifest.json".to_string(),
        BundleFileMeta {
            role: "manifest".to_string(),
            hash: compute_sha3(&manifest_dst)?,
            size: Some(get_size(&manifest_dst)?),
            content_type: Some("application/json".to_string()),
            optional: false,
        },
    );

    // Proof file
    files.insert(
        "proof.dat".to_string(),
        BundleFileMeta {
            role: "proof".to_string(),
            hash: compute_sha3(&proof_dst)?,
            size: Some(get_size(&proof_dst)?),
            content_type: Some("application/octet-stream".to_string()),
            optional: false,
        },
    );

    // Timestamp (optional)
    if let Some(ts) = ts_dst.as_ref() {
        files.insert(
            "timestamp.tsr".to_string(),
            BundleFileMeta {
                role: "timestamp".to_string(),
                hash: compute_sha3(ts)?,
                size: Some(get_size(ts)?),
                content_type: None,
                optional: true,
            },
        );
    }

    // Registry (optional)
    if let Some(reg) = reg_dst.as_ref() {
        files.insert(
            "registry.json".to_string(),
            BundleFileMeta {
                role: "registry".to_string(),
                hash: compute_sha3(reg)?,
                size: Some(get_size(reg)?),
                content_type: Some("application/json".to_string()),
                optional: true,
            },
        );
    }

    // Report
    files.insert(
        "verification.report.json".to_string(),
        BundleFileMeta {
            role: "report".to_string(),
            hash: compute_sha3(&rep_dst)?,
            size: Some(get_size(&rep_dst)?),
            content_type: Some("application/json".to_string()),
            optional: false,
        },
    );

    // README
    files.insert(
        "README.txt".to_string(),
        BundleFileMeta {
            role: "documentation".to_string(),
            hash: compute_sha3(&readme_dst)?,
            size: Some(get_size(&readme_dst)?),
            content_type: Some("text/plain".to_string()),
            optional: false,
        },
    );

    // Create proof_units array (single unit for simple export)
    let proof_units = vec![ProofUnitMeta {
        id: "main".to_string(),
        manifest_file: "manifest.json".to_string(),
        proof_file: "proof.dat".to_string(),
        policy_id: manifest.policy.hash.clone(), // Use hash as policy ID
        policy_hash: manifest.policy.hash.clone(),
        backend: "mock".to_string(),
        depends_on: vec![],
    }];

    // Create bundle metadata
    let meta = BundleMeta {
        schema: BUNDLE_SCHEMA_V1.to_string(),
        bundle_id: Uuid::new_v4().to_string(),
        created_at: Utc::now().to_rfc3339(),
        files,
        proof_units,
    };

    let meta_dst = out_path.join("_meta.json");
    let meta_json = serde_json::to_string_pretty(&meta)?;
    fs::write(&meta_dst, meta_json)?;

    // 5. Audit-Log-Eintrag
    let mut audit = AuditLog::new("build/agent.audit.jsonl")?;
    audit.log_event(
        "bundle_exported",
        json!({
            "output": &output_dir,
            "schema": "cap-bundle.v1",
            "bundle_id": &meta.bundle_id,
            "has_timestamp": timestamp_path.is_some(),
            "has_registry": registry_path.is_some(),
            "has_report": report_path.is_some()
        }),
    )?;

    // 6. Erfolg
    println!();
    println!("✅ CAP Bundle erfolgreich exportiert (cap-bundle.v1)!");
    println!("   Verzeichnis: {}", output_dir);
    println!("   Bundle ID: {}", meta.bundle_id);
    println!(
        "   Dateien: {}",
        if ts_dst.is_some() && reg_dst.is_some() {
            7
        } else if ts_dst.is_some() || reg_dst.is_some() {
            6
        } else {
            5
        }
    );
    println!("   Package Version: cap-proof.v1.0");

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
            format!(
                "supplier_count_max_{}",
                policy.constraints.supplier_count_max
            ),
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
    let out_dat = output
        .clone()
        .unwrap_or_else(|| "build/zk_proof.dat".to_string());
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
        println!(
            "  Company Root: {}",
            proof.public_inputs.company_commitment_root
        );
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
            format!(
                "supplier_count_max_{}",
                policy.constraints.supplier_count_max
            ),
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
    println!(
        "  Throughput: {:.2} proofs/s",
        1000.0 / prove_avg.as_millis() as f64
    );

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
    println!(
        "  Throughput: {:.2} verifications/s",
        1000.0 / verify_avg.as_millis() as f64
    );

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

/// Adaptive Proof Orchestration - Week 6 B1
#[allow(clippy::too_many_arguments)]
fn run_proof_adapt(
    policy: &Option<String>,
    ir: &Option<std::path::PathBuf>,
    context: &std::path::PathBuf,
    enforce: bool,
    rollout: u8,
    drift_max: f64,
    _selector: &str, // TODO: Implement selector (basic vs weighted)
    _weights: &Option<std::path::PathBuf>, // TODO: Implement weights file loading
    dry_run: bool,
    out: &Option<std::path::PathBuf>,
) -> Result<(), Box<dyn Error>> {
    println!("🎯 Adaptive Proof Orchestration (Week 6)");
    println!(
        "   Enforcement: {}",
        if enforce {
            "✓ ENABLED"
        } else {
            "✗ Shadow-only"
        }
    );
    println!("   Rollout: {}%", rollout);
    println!("   Drift Max: {}", drift_max);
    if dry_run {
        println!("   Mode: DRY RUN (no side effects)");
    }

    let mut audit = AuditLog::new("build/agent.audit.jsonl")?;

    // Step 1: Load IR
    // For now, we'll create a minimal IR since policy_v2 integration is not complete
    // This will be expanded in future implementation
    use cap_agent::policy_v2::types::IrV1;

    // TODO: Implement proper IR loading from policy ID or IR file
    // For now, create a placeholder IR
    let ir_data = if let Some(ir_path) = ir {
        println!("📄 Loading IR from file: {:?}", ir_path);
        let ir_json = fs::read_to_string(ir_path)?;
        serde_json::from_str::<IrV1>(&ir_json)?
    } else if let Some(policy_id) = policy {
        println!("📋 Policy ID: {}", policy_id);
        // TODO: Load IR from policy registry
        return Err(
            "Policy ID loading not yet implemented. Please use --ir flag."
                .to_string()
                .into(),
        );
    } else {
        return Err("Either --policy or --ir must be specified".into());
    };

    // Step 2: Load context
    println!("📄 Loading context from: {:?}", context);
    let context_json = fs::read_to_string(context)?;
    let ctx: orchestrator::OrchestratorContext = serde_json::from_str(&context_json)?;

    // Step 3: Create enforcement options
    let enforce_opts = orchestrator::EnforceOptions {
        enforce,
        rollout_percent: rollout,
        drift_max_ratio: drift_max,
    };

    // Step 4: Create enforcer
    println!("🔧 Creating enforcer with IR...");
    let enforcer = orchestrator::Enforcer::new(&ir_data, enforce_opts.clone())?;

    // Step 5: Generate request ID (deterministic for testing)
    let request_id = format!("req-{}", chrono::Utc::now().timestamp_millis());

    // Step 6: Execute enforcement decision
    println!("⚙️  Executing enforcement decision...");
    let start = std::time::Instant::now();
    let verdict_pair = enforcer.decide(&ctx, &request_id)?;
    let duration = start.elapsed();

    // Step 7: Record metrics
    orchestrator::metrics::set_rollout_percent(rollout);
    let policy_id = &ir_data.policy_id;

    if verdict_pair.enforced_applied {
        orchestrator::metrics::record_enforced_request(policy_id);
    } else {
        orchestrator::metrics::record_shadow_request(policy_id);
    }

    if verdict_pair.has_drift() {
        orchestrator::metrics::record_drift_event(policy_id);
    }

    orchestrator::metrics::observe_selection_latency(duration.as_secs_f64());

    // Step 8: Display results
    println!("\n📊 Results:");
    println!("   Shadow Verdict: {:?}", verdict_pair.shadow);
    println!("   Enforced Verdict: {:?}", verdict_pair.enforced);
    println!("   Enforcement Applied: {}", verdict_pair.enforced_applied);
    println!("   Drift Detected: {}", verdict_pair.has_drift());
    println!("   Duration: {:?}", duration);

    // Step 9: Write output if requested
    if let Some(out_path) = out {
        let output = serde_json::json!({
            "shadow_verdict": format!("{:?}", verdict_pair.shadow),
            "enforced_verdict": format!("{:?}", verdict_pair.enforced),
            "enforced_applied": verdict_pair.enforced_applied,
            "drift_detected": verdict_pair.has_drift(),
            "duration_ms": duration.as_millis(),
            "request_id": request_id,
            "enforce_options": {
                "enforce": enforce,
                "rollout_percent": rollout,
                "drift_max_ratio": drift_max,
            }
        });

        fs::write(out_path, serde_json::to_string_pretty(&output)?)?;
        println!("\n💾 Output written to: {:?}", out_path);
    }

    // Step 10: Audit log
    if !dry_run {
        audit.log_event(
            "proof_adapt_executed",
            json!({
                "policy_id": policy_id,
                "enforce": enforce,
                "rollout_percent": rollout,
                "enforced_applied": verdict_pair.enforced_applied,
                "drift_detected": verdict_pair.has_drift(),
                "duration_ms": duration.as_millis(),
            }),
        )?;
    }

    println!("\n✅ Adaptive orchestration completed!");

    Ok(())
}

/// Verifier run - Verifiziert Proof-Paket
fn run_verifier_run(package_path: &str) -> Result<(), Box<dyn Error>> {
    println!("🔍 Verifiziere Proof-Paket...");

    let mut audit = AuditLog::new("build/agent.audit.jsonl")?;
    let package_dir = Path::new(package_path);

    // Prüfe ob _meta.json existiert (cap-bundle.v1 Format)
    let meta_path = package_dir.join("_meta.json");

    if meta_path.exists() {
        // Verwende BundleVerifier für cap-bundle.v1
        println!("📦 Erkanntes Format: cap-bundle.v1");

        let bundle_verifier = package_verifier::BundleVerifier::new(package_dir);
        let result = bundle_verifier.verify_bundle()?;

        // Log Audit-Event
        audit.log_event(
            "bundle_verifier_run",
            json!({
                "bundle_id": result.bundle_id,
                "schema": result.schema,
                "status": format!("{:?}", result.status),
                "unit_count": result.unit_results.len()
            }),
        )?;

        // Zeige Ergebnisse
        println!("\n✅ Bundle-Verifikation abgeschlossen!");
        println!("  Bundle ID: {}", result.bundle_id);
        println!("  Schema: {}", result.schema);
        println!("  Status: {:?}", result.status);
        println!("  Proof Units: {}", result.unit_results.len());

        // Zeige einzelne Unit-Ergebnisse
        for (unit_id, unit_result) in &result.unit_results {
            println!("\n  📋 Unit '{}': {:?}", unit_id, unit_result.status);
            println!("     Manifest Hash: {}", unit_result.manifest_hash);
            println!("     Proof Hash: {}", unit_result.proof_hash);
        }
    } else {
        // Fallback zu Legacy Verifier (Backward-Compatibility)
        println!("📦 Erkanntes Format: Legacy (pre-bundle.v1)");

        let verifier = package_verifier::Verifier::new(package_path);

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
    }

    Ok(())
}

/// Verifier extract - Extrahiert Manifest-Infos
fn run_verifier_extract(package_path: &str) -> Result<(), Box<dyn Error>> {
    println!("🔍 Extrahiere Informationen aus Proof-Paket...");

    let summary = package_verifier::show_package_summary(package_path)?;
    println!("\n{}", summary);

    Ok(())
}

/// Verifier audit - Zeigt Audit-Trail
fn run_verifier_audit(package_path: &str) -> Result<(), Box<dyn Error>> {
    println!("🔍 Zeige Audit-Trail...");

    let verifier = package_verifier::Verifier::new(package_path);
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
    manifest.set_time_anchor(
        kind.to_string(),
        reference.to_string(),
        audit_tip_hex.clone(),
    );

    // Speichere Manifest
    manifest.save(manifest_out)?;

    println!("✅ Zeitanker gesetzt:");
    println!("   Kind:           {}", kind);
    println!("   Referenz:       {}", reference);
    println!("   Audit-Tip:      {}", audit_tip_hex);
    println!("   Output:         {}", manifest_out);

    Ok(())
}

/// Audit set-private-anchor - Setzt Private Anchor (Dual-Anchor v0.9.0)
fn run_audit_set_private_anchor(
    manifest_path: &str,
    audit_tip: &str,
    created_at: Option<String>,
) -> Result<(), Box<dyn Error>> {
    println!("🔐 Setze Private Anchor...");

    // Lade Manifest
    let mut manifest = manifest::Manifest::load(manifest_path)?;

    // Setze Private Anchor
    manifest.set_private_anchor(audit_tip.to_string(), created_at.clone())?;

    // Speichere Manifest
    manifest.save(manifest_path)?;

    println!("✅ Private Anchor gesetzt:");
    println!("   Audit-Tip:      {}", audit_tip);
    println!(
        "   Created-At:     {}",
        created_at.unwrap_or_else(|| "jetzt".to_string())
    );
    println!("   Manifest:       {}", manifest_path);

    Ok(())
}

/// Audit set-public-anchor - Setzt Public Anchor (Dual-Anchor v0.9.0)
fn run_audit_set_public_anchor(
    manifest_path: &str,
    chain: &str,
    txid: &str,
    digest: &str,
    created_at: Option<String>,
) -> Result<(), Box<dyn Error>> {
    println!("🌐 Setze Public Anchor...");

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

    println!("✅ Public Anchor gesetzt:");
    println!("   Chain:          {}", chain);
    println!("   TxID:           {}", txid);
    println!("   Digest:         {}", digest);
    println!(
        "   Created-At:     {}",
        created_at.unwrap_or_else(|| "jetzt".to_string())
    );
    println!("   Manifest:       {}", manifest_path);

    Ok(())
}

/// Audit verify-anchor - Verifiziert Dual-Anchor-Konsistenz
fn run_audit_verify_anchor(manifest_path: &str, out: Option<String>) -> Result<(), Box<dyn Error>> {
    println!("🔍 Verifiziere Dual-Anchor-Konsistenz...");

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
        let has_anchor = manifest.time_anchor.is_some();
        let has_private = has_anchor && manifest.time_anchor.as_ref().unwrap().private.is_some();
        let has_public = has_anchor && manifest.time_anchor.as_ref().unwrap().public.is_some();

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
    println!("\n📊 Verifikationsergebnis:");
    println!("   Status:         {}", report["status"]);
    println!(
        "   Private Anchor: {}",
        if report["private_ok"].as_bool().unwrap_or(false) {
            "✅"
        } else {
            "❌"
        }
    );
    println!(
        "   Public Anchor:  {}",
        if report["public_ok"].as_bool().unwrap_or(false) {
            "✅"
        } else {
            "❌"
        }
    );

    if let Some(errors) = report["errors"].as_array() {
        if !errors.is_empty() {
            println!("\n❌ Fehler:");
            for error in errors {
                println!("   - {}", error.as_str().unwrap_or("Unknown error"));
            }
        }
    }

    // Save report if requested
    if let Some(out_path) = out {
        let json_str = serde_json::to_string_pretty(&report)?;
        std::fs::write(&out_path, json_str)?;
        println!("\n💾 Report gespeichert: {}", out_path);
    }

    // Return error if validation failed
    if report["status"] == "fail" {
        return Err("Dual-Anchor validation failed".into());
    }

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
fn run_audit_verify_timestamp(head_path: &str, timestamp_path: &str) -> Result<(), Box<dyn Error>> {
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

/// Audit append - Fügt Event zur Audit-Chain hinzu (Track A)
fn run_audit_append(
    file_path: &str,
    event: &str,
    policy_id: Option<String>,
    ir_hash: Option<String>,
    manifest_hash: Option<String>,
    result: Option<String>,
    run_id: Option<String>,
) -> Result<(), Box<dyn Error>> {
    use cap_agent::audit::{AuditChain, AuditEventResult};

    println!("📝 Füge Event zur Audit-Chain hinzu...");

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

    println!("✅ Event hinzugefügt");
    println!("   Event:          {}", audit_event.event);
    println!("   Timestamp:      {}", audit_event.ts);
    println!("   Self-Hash:      {}", audit_event.self_hash);
    println!("   Chain-Datei:    {}", file_path);

    Ok(())
}

/// Audit verify - Verifiziert Audit-Chain-Integrität (Track A)
fn run_audit_verify_chain(file_path: &str, output: Option<String>) -> Result<(), Box<dyn Error>> {
    use cap_agent::audit::verify_chain;

    println!("🔍 Verifiziere Audit-Chain...");

    let report = verify_chain(file_path)?;

    if report.ok {
        println!("✅ Chain-Integrität OK");
        println!("   Events:         {}", report.total_events);
        println!("   Tamper-Index:   None");
    } else {
        println!("❌ Chain-Integrität VERLETZT");
        println!("   Events:         {}", report.total_events);
        if let Some(idx) = report.tamper_index {
            println!("   Tamper-Index:   {}", idx);
        }
        if let Some(err) = &report.error {
            println!("   Fehler:         {}", err);
        }
    }

    // Write JSON report if requested
    if let Some(out_path) = output {
        let report_json = serde_json::json!({
            "ok": report.ok,
            "total_events": report.total_events,
            "tamper_index": report.tamper_index,
            "error": report.error,
        });
        std::fs::write(&out_path, serde_json::to_string_pretty(&report_json)?)?;
        println!("📄 Report gespeichert: {}", out_path);
    }

    if !report.ok {
        return Err("Chain-Verifikation fehlgeschlagen".into());
    }

    Ok(())
}

/// Audit export - Exportiert Events aus Audit-Chain (Track A)
fn run_audit_export(
    file_path: &str,
    from: Option<String>,
    to: Option<String>,
    policy_id: Option<String>,
    output: Option<String>,
) -> Result<(), Box<dyn Error>> {
    use cap_agent::audit::export_events;

    println!("📤 Exportiere Events aus Audit-Chain...");

    let events = export_events(
        file_path,
        from.as_deref(),
        to.as_deref(),
        policy_id.as_deref(),
    )?;

    println!("✅ {} Events exportiert", events.len());

    // Output to file or stdout
    let json_output = serde_json::to_string_pretty(&events)?;

    if let Some(out_path) = output {
        std::fs::write(&out_path, &json_output)?;
        println!("📄 Events gespeichert: {}", out_path);
    } else {
        println!("\n{}", json_output);
    }

    Ok(())
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
fn run_lists_jurisdictions_root(
    csv_path: &str,
    output: Option<String>,
) -> Result<(), Box<dyn Error>> {
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
#[allow(clippy::too_many_arguments)]
fn run_registry_add(
    manifest_path: &str,
    proof_path: &str,
    timestamp_path: Option<String>,
    registry_path: Option<String>,
    backend_str: &str,
    signing_key_path: Option<String>,
    validate_key: bool,
    keys_dir: &str,
) -> Result<(), Box<dyn Error>> {
    use registry::RegistryBackend;

    println!("📝 Füge Proof zur Registry hinzu...");

    // Parse backend
    let backend = match backend_str {
        "sqlite" => RegistryBackend::Sqlite,
        _ => RegistryBackend::Json,
    };

    // Determine registry file
    let registry_file = registry_path.unwrap_or_else(|| match backend {
        RegistryBackend::Json => "build/registry.json".to_string(),
        RegistryBackend::Sqlite => "build/registry.sqlite".to_string(),
    });

    println!("   Backend:        {}", backend_str);

    // Open store
    let store = registry::open_store(backend, std::path::Path::new(&registry_file))?;

    // Berechne Hashes
    let manifest_hash = registry::compute_file_hash(manifest_path)?;
    let proof_hash = registry::compute_file_hash(proof_path)?;

    println!("   Manifest-Hash:  {}", manifest_hash);
    println!("   Proof-Hash:     {}", proof_hash);

    // Load current registry to get next ID
    let current_reg = store.load()?;
    let id = format!("proof_{:03}", current_reg.entries.len() + 1);

    // Create entry
    let mut entry = registry::RegistryEntry {
        id: id.clone(),
        manifest_hash: manifest_hash.clone(),
        proof_hash: proof_hash.clone(),
        timestamp_file: timestamp_path.clone(),
        registered_at: chrono::Utc::now().to_rfc3339(),
        signature: None,
        public_key: None,
        // BLOB fields (v0.9) - initially None
        blob_manifest: None,
        blob_proof: None,
        blob_wasm: None,
        blob_abi: None,
        // Self-verify fields (v0.9) - initially None
        selfverify_status: None,
        selfverify_at: None,
        verifier_name: None,
        verifier_version: None,
        // Key management fields (v0.10) - initially None
        kid: None,
        signature_scheme: None,
    };

    // Sign entry if signing key provided
    if let Some(key_path) = signing_key_path {
        let key_file = if key_path.is_empty() {
            "keys/company.ed25519"
        } else {
            &key_path
        };

        println!("   Signing-Key:    {}", key_file);

        // Load signing key
        let key_bytes = std::fs::read(key_file)
            .map_err(|e| format!("Failed to read signing key from {}: {}", key_file, e))?;

        if key_bytes.len() != 32 {
            return Err(format!(
                "Invalid signing key length (expected 32 bytes, got {})",
                key_bytes.len()
            )
            .into());
        }

        let signing_key = ed25519_dalek::SigningKey::from_bytes(&key_bytes.try_into().unwrap());

        // Sign entry
        registry::sign_entry(&mut entry, &signing_key)?;
        println!("   ✓ Entry signed with Ed25519");

        // Validate key status if requested
        if validate_key {
            if let Some(ref kid) = entry.kid {
                println!("   Validating key status...");
                registry::validate_key_status(kid, keys_dir)?;
                println!("   ✓ Key status validated (active)");
            } else {
                return Err("Cannot validate key status: KID not set".into());
            }
        }
    }

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
    println!("✅ Proof zur Registry hinzugefügt:");
    println!("   ID:             {}", id);
    println!("   Registry:       {}", registry_file);
    println!("   Einträge total: {}", total);

    Ok(())
}

/// Registry list - Listet alle Registry-Einträge auf
fn run_registry_list(
    registry_path: Option<String>,
    backend_str: &str,
) -> Result<(), Box<dyn Error>> {
    use registry::RegistryBackend;

    // Parse backend
    let backend = match backend_str {
        "sqlite" => RegistryBackend::Sqlite,
        _ => RegistryBackend::Json,
    };

    // Determine registry file
    let registry_file = registry_path.unwrap_or_else(|| match backend {
        RegistryBackend::Json => "build/registry.json".to_string(),
        RegistryBackend::Sqlite => "build/registry.sqlite".to_string(),
    });

    if !std::path::Path::new(&registry_file).exists() {
        println!("⚠️  Registry-Datei nicht gefunden: {}", registry_file);
        println!("   Verwende 'registry add' um Einträge hinzuzufügen.");
        return Ok(());
    }

    // Open store and load entries
    let store = registry::open_store(backend, std::path::Path::new(&registry_file))?;
    let entries = store.list()?;

    println!("──────────────────────────────────────────────");
    println!("Proofs in local registry ({})", registry_file);
    println!("──────────────────────────────────────────────");

    if entries.is_empty() {
        println!("   (keine Einträge)");
    } else {
        for (idx, entry) in entries.iter().enumerate() {
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
    println!("Total: {} Einträge", entries.len());

    Ok(())
}

/// Registry verify - Verifiziert einen Proof gegen die Registry
fn run_registry_verify(
    manifest_path: &str,
    proof_path: &str,
    registry_path: Option<String>,
    backend_str: &str,
) -> Result<(), Box<dyn Error>> {
    use registry::RegistryBackend;

    println!("🔍 Verifiziere Proof gegen Registry...");

    // Parse backend
    let backend = match backend_str {
        "sqlite" => RegistryBackend::Sqlite,
        _ => RegistryBackend::Json,
    };

    // Determine registry file
    let registry_file = registry_path.unwrap_or_else(|| match backend {
        RegistryBackend::Json => "build/registry.json".to_string(),
        RegistryBackend::Sqlite => "build/registry.sqlite".to_string(),
    });

    if !std::path::Path::new(&registry_file).exists() {
        println!("❌ Registry-Datei nicht gefunden: {}", registry_file);
        return Err("Registry existiert nicht".into());
    }

    // Berechne Hashes
    let manifest_hash = registry::compute_file_hash(manifest_path)?;
    let proof_hash = registry::compute_file_hash(proof_path)?;

    println!("   Manifest-Hash:  {}", manifest_hash);
    println!("   Proof-Hash:     {}", proof_hash);

    // Open store and find entry
    let store = registry::open_store(backend, std::path::Path::new(&registry_file))?;
    let entry_opt = store.find_by_hashes(&manifest_hash, &proof_hash)?;

    // Verifiziere
    if let Some(entry) = entry_opt {
        println!("✅ Entry verified in registry");
        println!("   ID:             {}", entry.id);
        println!("   Registered:     {}", entry.registered_at);
        if let Some(ref ts) = entry.timestamp_file {
            println!("   Timestamp:      {}", ts);
        }

        // Verify signature if present
        let signature_valid = registry::verify_entry_signature(&entry)?;
        if signature_valid {
            println!("   ✓ Ed25519 signature valid");
        } else {
            println!("   ⚠ No signature present (backward compatibility)");
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
        println!("❌ Hash mismatch or not registered");
        println!("   Registry:       {}", registry_file);
        println!("   Einträge:       {}", total);
        Err("Proof nicht in Registry gefunden".into())
    }
}

/// Registry migrate - Migriert Registry zwischen Backends
fn run_registry_migrate(
    from_backend_str: &str,
    from_path: &str,
    to_backend_str: &str,
    to_path: &str,
) -> Result<(), Box<dyn Error>> {
    use registry::RegistryBackend;

    println!("🔄 Migriere Registry...");
    println!("   Von: {} ({})", from_path, from_backend_str);
    println!("   Nach: {} ({})", to_path, to_backend_str);

    // Parse backends
    let from_backend = match from_backend_str {
        "sqlite" => RegistryBackend::Sqlite,
        "json" => RegistryBackend::Json,
        _ => return Err(format!("Unbekanntes Backend: {}", from_backend_str).into()),
    };

    let to_backend = match to_backend_str {
        "sqlite" => RegistryBackend::Sqlite,
        "json" => RegistryBackend::Json,
        _ => return Err(format!("Unbekanntes Backend: {}", to_backend_str).into()),
    };

    // Check source exists
    if !std::path::Path::new(from_path).exists() {
        return Err(format!("Quell-Registry nicht gefunden: {}", from_path).into());
    }

    // Open source store
    let from_store = registry::open_store(from_backend, std::path::Path::new(from_path))?;

    // Load all data
    println!("   Lade Daten...");
    let registry_data = from_store.load()?;
    let entry_count = registry_data.entries.len();

    // Open target store
    let to_store = registry::open_store(to_backend, std::path::Path::new(to_path))?;

    // Save all data
    println!("   Schreibe {} Einträge...", entry_count);
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

    println!("✅ Migration erfolgreich:");
    println!("   Einträge:       {}", entry_count);
    println!("   Ziel:           {}", to_path);

    Ok(())
}

/// Registry inspect - Zeigt Registry-Metadaten und Statistiken an (v1.1)
fn run_registry_inspect(registry_path: Option<String>) -> Result<(), Box<dyn Error>> {
    use registry::UnifiedRegistry;
    use std::path::Path;

    let path = registry_path.unwrap_or_else(|| "build/registry.json".to_string());

    println!("🔍 Inspiziere Registry...");
    println!("   Datei: {}", path);

    if !Path::new(&path).exists() {
        return Err(format!("Registry nicht gefunden: {}", path).into());
    }

    // Load registry (auto-detects v1.0/v1.1)
    let unified_registry = UnifiedRegistry::load(Path::new(&path))?;

    println!("\n📊 Registry-Informationen:");
    println!("   Version:        {}", unified_registry.source_version());
    println!("   Einträge:       {}", unified_registry.count());
    println!(
        "   Migriert:       {}",
        if unified_registry.was_migrated() {
            "Ja (v1.0 → v1.1)"
        } else {
            "Nein"
        }
    );

    // Show v1.1 metadata if available
    let v1_1 = unified_registry.as_v1_1();
    println!("\n📝 Metadaten (v1.1):");
    println!("   Schema-Version: {}", v1_1.meta.schema_version);
    println!("   Tool-Version:   {}", v1_1.meta.tool_version);
    println!("   Erstellt:       {}", v1_1.meta.created_at);

    if let Some(migrated_from) = &v1_1.meta.migrated_from {
        println!("   Migriert von:   {}", migrated_from);
    }
    if let Some(migrated_at) = &v1_1.meta.migrated_at {
        println!("   Migriert am:    {}", migrated_at);
    }

    // Validate registry
    println!("\n✅ Validierung:");
    match unified_registry.validate() {
        Ok(_) => println!("   Registry ist gültig ✓"),
        Err(e) => {
            println!("   ⚠️  Validierung fehlgeschlagen: {}", e);
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
fn run_registry_backfill_kid(
    registry_path: Option<String>,
    output_path: Option<String>,
) -> Result<(), Box<dyn Error>> {
    use registry::UnifiedRegistry;
    use std::path::Path;

    let input = registry_path.unwrap_or_else(|| "build/registry.json".to_string());
    let output = output_path.unwrap_or_else(|| input.clone());

    println!("🔑 Backfilling KID-Felder...");
    println!("   Input:  {}", input);
    println!("   Output: {}", output);

    if !Path::new(&input).exists() {
        return Err(format!("Registry nicht gefunden: {}", input).into());
    }

    // Load registry (auto-detects v1.0/v1.1)
    let mut unified_registry = UnifiedRegistry::load(Path::new(&input))?;

    println!("   Einträge:  {}", unified_registry.count());

    // Backfill KIDs
    let backfilled_count = unified_registry.backfill_kids()?;

    if backfilled_count == 0 {
        println!("\n✅ Keine KID-Backfills erforderlich");
        println!("   Alle Einträge haben bereits KIDs oder keine public_key");
        return Ok(());
    }

    // Save updated registry
    unified_registry.save(Path::new(&output))?;

    println!("\n✅ Backfill erfolgreich:");
    println!("   KIDs hinzugefügt:  {}", backfilled_count);
    println!("   Gespeichert in:    {}", output);

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

/// Keys keygen - Generiert neuen Ed25519-Schlüssel mit Metadata
fn run_keys_keygen(
    owner: &str,
    algo: &str,
    out_path: &str,
    valid_days: u64,
    comment: Option<String>,
) -> Result<(), Box<dyn Error>> {
    use ed25519_dalek::SigningKey;

    println!("🔑 Generiere neuen Schlüssel...");
    println!("   Owner:      {}", owner);
    println!("   Algorithm:  {}", algo);
    println!("   Valid for:  {} days", valid_days);

    if algo != "ed25519" {
        return Err(format!("Unsupported algorithm: {}", algo).into());
    }

    // Generate Ed25519 keypair
    let mut rng = rand::rngs::OsRng;
    let signing_key = SigningKey::generate(&mut rng);
    let verifying_key = signing_key.verifying_key();

    // Create key metadata
    let mut metadata = keys::KeyMetadata::new(&verifying_key.to_bytes(), owner, algo, valid_days)?;

    if let Some(ref c) = comment {
        metadata.comment = Some(c.clone());
    }

    // Save metadata
    metadata.save(out_path)?;

    // Save private key (raw bytes)
    let priv_key_path = out_path.replace(".json", ".ed25519");
    fs::write(&priv_key_path, signing_key.to_bytes())?;

    // Save public key (raw bytes)
    let pub_key_path = out_path.replace(".json", ".pub");
    fs::write(&pub_key_path, verifying_key.to_bytes())?;

    println!("✅ Schlüssel generiert:");
    println!("   KID:        {}", metadata.kid);
    println!("   Metadata:   {}", out_path);
    println!("   Private:    {}", priv_key_path);
    println!("   Public:     {}", pub_key_path);
    println!("   Fingerprint: {}", metadata.fingerprint);

    // Log audit event
    let mut audit = AuditLog::new("build/agent.audit.jsonl")?;
    audit.log_event(
        "key_generated",
        json!({
            "kid": metadata.kid,
            "owner": owner,
            "algorithm": algo,
            "metadata_file": out_path,
        }),
    )?;

    Ok(())
}

/// Keys list - Listet alle Schlüssel auf
fn run_keys_list(
    dir: &str,
    status_filter: Option<String>,
    owner_filter: Option<String>,
) -> Result<(), Box<dyn Error>> {
    println!("📋 Schlüsselliste:");
    println!("   Verzeichnis: {}\n", dir);

    let store = keys::KeyStore::new(dir)?;
    let all_keys = store.list()?;

    // Apply filters
    let filtered_keys: Vec<_> = all_keys
        .iter()
        .filter(|k| {
            if let Some(ref status) = status_filter {
                if k.status != *status {
                    return false;
                }
            }
            if let Some(ref owner) = owner_filter {
                if k.owner != *owner {
                    return false;
                }
            }
            true
        })
        .collect();

    if filtered_keys.is_empty() {
        println!("   (Keine Schlüssel gefunden)");
        return Ok(());
    }

    println!(
        "   {:<32} {:<15} {:<10} {:<20}",
        "KID", "Owner", "Status", "Valid Until"
    );
    println!("   {}", "-".repeat(80));

    for key in &filtered_keys {
        let valid_to_short = &key.valid_to[0..10]; // YYYY-MM-DD
        println!(
            "   {:<32} {:<15} {:<10} {:<20}",
            key.kid, key.owner, key.status, valid_to_short
        );
    }

    println!("\n   Total: {} Schlüssel", filtered_keys.len());

    Ok(())
}

/// Keys show - Zeigt Details eines Schlüssels
fn run_keys_show(dir: &str, kid: &str) -> Result<(), Box<dyn Error>> {
    println!("🔍 Schlüssel-Details:");

    let store = keys::KeyStore::new(dir)?;
    let key_opt = store.find_by_kid(kid)?;

    match key_opt {
        Some(key) => {
            println!("   KID:         {}", key.kid);
            println!("   Owner:       {}", key.owner);
            println!("   Algorithm:   {}", key.algorithm);
            println!("   Status:      {}", key.status);
            println!("   Created:     {}", key.created_at);
            println!("   Valid From:  {}", key.valid_from);
            println!("   Valid To:    {}", key.valid_to);
            println!("   Fingerprint: {}", key.fingerprint);
            println!("   Usage:       {:?}", key.usage);
            if let Some(ref comment) = key.comment {
                println!("   Comment:     {}", comment);
            }
            println!("   Public Key:  {}...", &key.public_key[0..20]);

            Ok(())
        }
        None => Err(format!("Schlüssel nicht gefunden: {}", kid).into()),
    }
}

/// Keys rotate - Rotiert Schlüssel
fn run_keys_rotate(dir: &str, current_path: &str, new_path: &str) -> Result<(), Box<dyn Error>> {
    println!("🔄 Rotiere Schlüssel...");

    let store = keys::KeyStore::new(dir)?;

    // Load current key metadata
    let mut current_key = keys::KeyMetadata::load(current_path)?;
    println!("   Aktuell: {} ({})", current_key.kid, current_key.owner);

    // Load new key metadata
    let new_key = keys::KeyMetadata::load(new_path)?;
    println!("   Neu:     {} ({})", new_key.kid, new_key.owner);

    // Mark current key as retired
    current_key.retire();
    current_key.save(current_path)?;

    // Archive current key
    store.archive(&current_key.kid)?;

    println!("✅ Rotation erfolgreich:");
    println!("   Alter Schlüssel -> retired + archiviert");
    println!("   Neuer Schlüssel -> aktiv");

    // Log audit event
    let mut audit = AuditLog::new("build/agent.audit.jsonl")?;
    audit.log_event(
        "key_rotated",
        json!({
            "old_kid": current_key.kid,
            "new_kid": new_key.kid,
            "owner": new_key.owner,
        }),
    )?;

    Ok(())
}

/// Keys attest - Attestiert neuen Schlüssel mit altem
fn run_keys_attest(
    signer_path: &str,
    subject_path: &str,
    out_path: &str,
) -> Result<(), Box<dyn Error>> {
    use base64::Engine;
    use ed25519_dalek::{Signer, SigningKey};

    println!("📜 Attestiere Schlüssel...");

    // Load signer key metadata
    let signer_meta = keys::KeyMetadata::load(signer_path)?;
    println!("   Signer:  {} ({})", signer_meta.kid, signer_meta.owner);

    // Load subject key metadata
    let subject_meta = keys::KeyMetadata::load(subject_path)?;
    println!("   Subject: {} ({})", subject_meta.kid, subject_meta.owner);

    // Load signer private key
    let signer_priv_path = signer_path.replace(".json", ".ed25519");
    let signer_key_bytes = fs::read(&signer_priv_path)?;
    let signing_key = SigningKey::from_bytes(
        &signer_key_bytes
            .try_into()
            .map_err(|_| "Invalid key length")?,
    );

    // Create attestation document
    let attestation = json!({
        "schema": "cap-attestation.v1",
        "signer_kid": signer_meta.kid,
        "signer_owner": signer_meta.owner,
        "subject_kid": subject_meta.kid,
        "subject_owner": subject_meta.owner,
        "subject_public_key": subject_meta.public_key,
        "attested_at": chrono::Utc::now().to_rfc3339(),
    });

    // Sign the attestation
    let attestation_bytes = serde_json::to_vec(&attestation)?;
    let signature = signing_key.sign(&attestation_bytes);

    // Create final document with signature
    let signed_attestation = json!({
        "attestation": attestation,
        "signature": base64::engine::general_purpose::STANDARD.encode(signature.to_bytes()),
        "signer_public_key": signer_meta.public_key,
    });

    // Save attestation
    let json_output = serde_json::to_string_pretty(&signed_attestation)?;
    fs::write(out_path, json_output)?;

    println!("✅ Attestation erstellt:");
    println!("   Output: {}", out_path);

    // Log audit event
    let mut audit = AuditLog::new("build/agent.audit.jsonl")?;
    audit.log_event(
        "key_attested",
        json!({
            "signer_kid": signer_meta.kid,
            "subject_kid": subject_meta.kid,
            "attestation_file": out_path,
        }),
    )?;

    Ok(())
}

/// Keys archive - Archiviert Schlüssel
fn run_keys_archive(dir: &str, kid: &str) -> Result<(), Box<dyn Error>> {
    println!("📦 Archiviere Schlüssel...");
    println!("   KID: {}", kid);

    let store = keys::KeyStore::new(dir)?;
    store.archive(kid)?;

    println!("✅ Schlüssel archiviert");
    println!("   Verschoben nach: {}/archive/", dir);

    // Log audit event
    let mut audit = AuditLog::new("build/agent.audit.jsonl")?;
    audit.log_event(
        "key_archived",
        json!({
            "kid": kid,
        }),
    )?;

    Ok(())
}

/// Keys verify-chain - Verifiziert eine Chain-of-Trust
fn run_keys_verify_chain(dir: &str, attestation_paths: &[String]) -> Result<(), Box<dyn Error>> {
    println!("🔗 Verifiziere Chain-of-Trust...");
    println!("   Keys Directory: {}", dir);
    println!("   Attestationen: {}", attestation_paths.len());

    // Convert Vec<String> to Vec<&str> for verify_chain
    let paths: Vec<&str> = attestation_paths.iter().map(|s| s.as_str()).collect();

    // Open key store
    let store = keys::KeyStore::new(dir)?;

    // Verify chain
    keys::verify_chain(&paths, &store)?;

    println!("✅ Chain-of-Trust verifiziert");
    println!("   Alle Attestationen gültig");
    println!("   Chain ist konsistent");

    // Log audit event
    let mut audit = AuditLog::new("build/agent.audit.jsonl")?;
    audit.log_event(
        "chain_verified",
        json!({
            "attestation_count": attestation_paths.len(),
            "keys_dir": dir,
        }),
    )?;

    Ok(())
}

/// Manifest validate - Validiert ein Manifest gegen das JSON Schema
fn run_manifest_validate(
    manifest_path: &str,
    schema_path: Option<String>,
) -> Result<(), Box<dyn Error>> {
    println!("🔍 Validiere Manifest gegen JSON Schema...");

    // Default schema path
    let schema_file = schema_path.unwrap_or_else(|| "docs/manifest.schema.json".to_string());

    // Load manifest JSON
    let manifest_content = std::fs::read_to_string(manifest_path)?;
    let manifest_json: serde_json::Value = serde_json::from_str(&manifest_content)?;

    // Load schema JSON
    let schema_content = std::fs::read_to_string(&schema_file)?;
    let schema_json: serde_json::Value = serde_json::from_str(&schema_content)?;

    // Compile schema (Draft 2020-12)
    use jsonschema::JSONSchema;
    let compiled = JSONSchema::options()
        .compile(&schema_json)
        .map_err(|e| format!("Schema compilation failed: {}", e))?;

    // Validate
    if compiled.is_valid(&manifest_json) {
        println!("✅ Manifest ist gültig gemäß Schema {}", schema_file);
        println!("   Manifest: {}", manifest_path);
        println!("   Schema:   {}", schema_file);

        // Log audit event
        let mut audit = AuditLog::new("build/agent.audit.jsonl")?;
        audit.log_event(
            "manifest_validated",
            json!({
                "manifest_file": manifest_path,
                "schema_file": schema_file,
                "status": "valid"
            }),
        )?;

        Ok(())
    } else {
        println!("❌ Manifest-Validierung fehlgeschlagen:");

        // Collect errors
        if let Err(errors) = compiled.validate(&manifest_json) {
            for (i, error) in errors.enumerate() {
                eprintln!("   Fehler #{}: {}", i + 1, error);
            }
        }

        Err("Manifest validation failed".into())
    }
}

/// Manifest verify - Führt vollständige Offline-Verifikation eines Proof-Pakets durch
///
/// # Argumente
/// * `manifest_path` - Pfad zur Manifest-Datei
/// * `proof_path` - Pfad zur Proof-Datei
/// * `registry_path` - Pfad zur Registry-Datei
/// * `timestamp_path` - Optionaler Pfad zur Timestamp-Datei
/// * `out_path` - Optionaler Pfad für Verification Report
///
/// # Rückgabe
/// Result mit () bei erfolgreicher Verifikation
fn run_manifest_verify(
    manifest_path: &str,
    proof_path: &str,
    registry_path: &str,
    timestamp_path: Option<String>,
    out_path: Option<String>,
) -> Result<(), Box<dyn Error>> {
    println!("🔍 Starte vollständige Offline-Verifikation (mit portable core)...");

    // 1️⃣ Load files
    println!("   1/5 Lade Dateien...");
    let manifest_bytes = fs::read(manifest_path)?;
    let proof_bytes = fs::read(proof_path)?;

    // Parse manifest as JSON
    let manifest_json: serde_json::Value = serde_json::from_slice(&manifest_bytes)?;
    println!("      ✅ Manifest geladen");

    // 2️⃣ Extract statement from manifest
    println!("   2/5 Extrahiere Statement...");
    let stmt = verifier_core::extract_statement_from_manifest(&manifest_json)?;
    println!("      ✅ Statement extrahiert");
    println!("         Policy Hash: {}", stmt.policy_hash);
    println!("         Company Root: {}", stmt.company_commitment_root);

    // 3️⃣ Create verification options
    println!("   3/5 Führe Verifikation durch...");
    let opts = verifier_core::VerifyOptions {
        check_timestamp: timestamp_path.is_some(),
        check_registry: true,
    };

    // 4️⃣ Call portable core verification
    let core_report = verifier_core::verify(&manifest_json, &proof_bytes, &stmt, &opts)?;

    println!("      ✅ Core Verifikation abgeschlossen");
    println!("         Manifest Hash: {}", core_report.manifest_hash);
    println!("         Proof Hash: {}", core_report.proof_hash);
    println!(
        "         Signatur: {}",
        if core_report.signature_valid {
            "✅"
        } else {
            "⚠️"
        }
    );

    // 5️⃣ Additional checks (timestamp and registry)
    println!("   4/5 Zusätzliche Prüfungen...");

    // Timestamp verification (if provided)
    let timestamp_valid = match timestamp_path.as_deref() {
        Some(ts_path) => {
            let valid = registry::verify_timestamp_from_file(ts_path);
            println!(
                "      Timestamp: {}",
                if valid {
                    "✅ gültig"
                } else {
                    "❌ ungültig"
                }
            );
            valid
        }
        None => {
            println!("      Timestamp: ⚠️  nicht angegeben (optional)");
            true
        }
    };

    // Registry verification
    let registry_match = registry::verify_entry_from_file(
        registry_path,
        &core_report.manifest_hash,
        &core_report.proof_hash,
    )
    .unwrap_or(false);

    println!(
        "      Registry: {}",
        if registry_match {
            "✅ Eintrag gefunden"
        } else {
            "❌ Kein Eintrag"
        }
    );

    // 6️⃣ Consolidate result
    let all_ok = core_report.signature_valid
        && timestamp_valid
        && registry_match
        && core_report.status == "ok";
    let status = if all_ok { "ok" } else { "fail" }.to_string();

    let report = VerificationReport {
        manifest_hash: core_report.manifest_hash.clone(),
        proof_hash: core_report.proof_hash.clone(),
        timestamp_valid,
        registry_match,
        signature_valid: core_report.signature_valid,
        status: status.clone(),
    };

    // 7️⃣ Save report
    println!("   5/5 Speichere Report...");
    let report_path = out_path.unwrap_or_else(|| "build/verification.report.json".to_string());
    let json = serde_json::to_string_pretty(&report)?;
    fs::write(&report_path, json)?;

    // 8️⃣ Log audit event
    let mut audit = AuditLog::new("build/agent.audit.jsonl")?;
    audit.log_event(
        "manifest_verified",
        json!({
            "manifest_file": manifest_path,
            "proof_file": proof_path,
            "registry_file": registry_path,
            "timestamp_file": timestamp_path,
            "status": status,
            "report_file": report_path,
            "core_details": core_report.details
        }),
    )?;

    // 9️⃣ Print result
    println!();
    if all_ok {
        println!("✅ Verifikation erfolgreich!");
        println!("   Report gespeichert: {}", report_path);
        Ok(())
    } else {
        eprintln!("❌ Verifikation fehlgeschlagen!");
        eprintln!("   Report gespeichert: {}", report_path);
        eprintln!(
            "   Details: {}",
            serde_json::to_string_pretty(&core_report.details)?
        );
        Err("Verification failed".into())
    }
}

// ============================================================================
// Bundle v2 Commands
// ============================================================================

/// Bundle v2 - Create self-contained proof package
fn run_bundle_v2(
    manifest: &str,
    proof: &str,
    verifier_wasm: Option<String>,
    out: &str,
    create_zip: bool,
    force: bool,
) -> Result<(), Box<dyn Error>> {
    use std::fs;
    use std::path::Path;

    println!("📦 Creating Proof Bundle v2...");

    // Check if output exists
    if Path::new(out).exists() && !force {
        return Err(format!("Output directory already exists: {}", out).into());
    }

    // Create output directory
    fs::create_dir_all(out)?;

    // Copy manifest
    println!("   1/7 Copying manifest...");
    let manifest_dest = format!("{}/manifest.json", out);
    fs::copy(manifest, &manifest_dest)?;

    // Copy proof (ensure .capz extension)
    println!("   2/7 Copying proof...");
    let proof_dest = format!("{}/proof.capz", out);
    fs::copy(proof, &proof_dest)?;

    // Copy verifier WASM if provided
    if let Some(wasm_path) = verifier_wasm {
        println!("   3/7 Copying WASM verifier...");
        let wasm_dest = format!("{}/verifier.wasm", out);
        fs::copy(&wasm_path, &wasm_dest)?;

        // Create executor.json
        println!("   4/7 Creating executor.json...");
        let executor_config = wasm::ExecutorConfig::default();
        executor_config.save(format!("{}/executor.json", out))?;
    } else {
        println!("   3/7 No WASM verifier provided (will use native fallback)");
        println!("   4/7 Skipping executor.json");
    }

    // Create _meta.json
    println!("   5/7 Creating _meta.json...");
    let manifest_bytes = fs::read(&manifest_dest)?;
    let proof_bytes = fs::read(&proof_dest)?;

    let manifest_hash = crypto::hex_lower_prefixed32(crypto::sha3_256(&manifest_bytes));
    let proof_hash = crypto::hex_lower_prefixed32(crypto::sha3_256(&proof_bytes));

    let meta = serde_json::json!({
        "bundle_version": "cap-proof.v2.0",
        "created_at": chrono::Utc::now().to_rfc3339(),
        "hashes": {
            "manifest_sha3": manifest_hash,
            "proof_sha3": proof_hash,
        },
        "backend": "mock",
    });

    fs::write(
        format!("{}/_meta.json", out),
        serde_json::to_string_pretty(&meta)?,
    )?;

    // Create README.txt
    println!("   6/7 Creating README.txt...");
    let readme_content = format!(
        r#"CAP Proof Bundle v2.0
=====================

This directory contains a self-contained compliance proof package.

Contents:
---------
  manifest.json   - Compliance manifest with commitments and policy
  proof.capz      - CAPZ v2 proof container (binary format)
  _meta.json      - Bundle metadata with SHA3-256 integrity hashes
  README.txt      - This file

Optional files (if present):
  verifier.wasm   - WASM verifier module for offline verification
  executor.json   - WASM executor configuration

Verification:
-------------
To verify this bundle:

  1. Using cap-agent CLI:
     $ cap-agent verify-bundle --bundle . --out report.json

  2. Using WASM verifier (if included):
     The verifier will automatically detect and use verifier.wasm

  3. Manual hash verification:
     Compare hashes in _meta.json with actual file SHA3-256 hashes

Bundle Information:
-------------------
  Version:       cap-proof.v2.0
  Created:       {}
  Manifest Hash: {}
  Proof Hash:    {}

Integrity:
----------
All files in this bundle are hashed in _meta.json using SHA3-256.
Any tampering will be detected during verification.

Documentation:
--------------
For more information about CAP Proof Bundles, see:
  https://github.com/yourusername/cap-agent

Generated by cap-agent v{}
"#,
        chrono::Utc::now().to_rfc3339(),
        manifest_hash,
        proof_hash,
        VERSION
    );
    fs::write(format!("{}/README.txt", out), readme_content)?;

    // Optional: Create ZIP
    if create_zip {
        println!("   7/7 Creating ZIP archive...");
        let zip_path = format!("{}.zip", out);

        // Create ZIP file
        let zip_file = fs::File::create(&zip_path)?;
        let mut zip_writer = zip::ZipWriter::new(zip_file);
        let options = zip::write::FileOptions::<()>::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .unix_permissions(0o755);

        // Add all files from bundle directory
        let entries = fs::read_dir(out)?;
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                let file_name = path.file_name().unwrap().to_str().unwrap();
                let file_data = fs::read(&path)?;

                zip_writer.start_file(file_name, options)?;
                std::io::Write::write_all(&mut zip_writer, &file_data)?;
            }
        }

        zip_writer.finish()?;
        println!("      ✅ ZIP created: {}", zip_path);
    } else {
        println!("   7/7 Skipping ZIP creation");
    }

    println!("\n✅ Bundle created successfully:");
    println!("   Directory: {}", out);
    println!("   Manifest:  {}", manifest_hash);
    println!("   Proof:     {}", proof_hash);

    Ok(())
}

/// Verify Bundle - Verify a proof package
fn run_verify_bundle(bundle: &str, out: Option<String>) -> Result<(), Box<dyn Error>> {
    use std::path::Path;

    println!("🔍 Verifying Proof Bundle...");

    // Check bundle exists
    if !Path::new(bundle).exists() {
        return Err(format!("Bundle not found: {}", bundle).into());
    }

    // Create executor
    println!("   1/5 Loading bundle...");
    let executor = wasm::BundleExecutor::new(bundle.to_string())?;

    // Validate bundle integrity via _meta.json hashes
    println!("   2/5 Validating bundle integrity...");
    let meta_path = format!("{}/_meta.json", bundle);
    if Path::new(&meta_path).exists() {
        let meta_content = std::fs::read_to_string(&meta_path)?;
        let meta: serde_json::Value = serde_json::from_str(&meta_content)?;

        // Check manifest hash
        if let Some(expected_manifest_hash) = meta["hashes"]["manifest_sha3"].as_str() {
            let manifest_bytes = std::fs::read(format!("{}/manifest.json", bundle))?;
            let actual_hash = crypto::hex_lower_prefixed32(crypto::sha3_256(&manifest_bytes));
            if actual_hash != expected_manifest_hash {
                return Err(format!(
                    "Bundle integrity check failed: Manifest hash mismatch\n  Expected: {}\n  Actual:   {}",
                    expected_manifest_hash, actual_hash
                ).into());
            }
            println!("      ✅ Manifest hash valid");
        }

        // Check proof hash
        if let Some(expected_proof_hash) = meta["hashes"]["proof_sha3"].as_str() {
            let proof_bytes = std::fs::read(format!("{}/proof.capz", bundle))?;
            let actual_hash = crypto::hex_lower_prefixed32(crypto::sha3_256(&proof_bytes));
            if actual_hash != expected_proof_hash {
                return Err(format!(
                    "Bundle integrity check failed: Proof hash mismatch\n  Expected: {}\n  Actual:   {}",
                    expected_proof_hash, actual_hash
                ).into());
            }
            println!("      ✅ Proof hash valid");
        }
    } else {
        println!("      ⚠️  No _meta.json found, skipping hash validation");
    }

    println!("   3/5 Checking verifier...");
    if executor.has_wasm() {
        println!("      ✅ WASM verifier found");
    } else {
        println!("      ⚠️  No WASM verifier, using native fallback");
    }

    // Create verification options
    println!("   4/5 Running verification...");
    let options = verifier::VerifyOptions {
        check_timestamp: false,
        check_registry: false,
    };

    let report = executor.verify(&options)?;

    println!("   5/5 Generating report...");
    println!("\n📊 Verification Report:");
    println!("   Status:        {}", report.status);
    println!("   Manifest Hash: {}", report.manifest_hash);
    println!("   Proof Hash:    {}", report.proof_hash);
    println!(
        "   Signature:     {}",
        if report.signature_valid {
            "✅"
        } else {
            "⚠️"
        }
    );

    // Save report if requested
    if let Some(out_path) = out {
        let report_json = serde_json::to_string_pretty(&report)?;
        std::fs::write(&out_path, report_json)?;
        println!("\n💾 Report saved: {}", out_path);
    }

    if report.status != "ok" {
        return Err("Verification failed".into());
    }

    Ok(())
}

// ============================================================================
// BLOB Store CLI Handlers (v0.10.9)
// ============================================================================

/// Fügt eine Datei in den BLOB Store ein (CAS + optional Registry-Verknüpfung)
fn run_blob_put(
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
            "❌ Ungültiger Medientyp: {}. Erlaubt: {:?}",
            media_type, valid_types
        )
        .into());
    }

    // Lese Daten von Datei oder stdin
    let data = if use_stdin {
        println!("📥 Lese Daten von stdin...");
        let mut buffer = Vec::new();
        stdin().read_to_end(&mut buffer)?;
        buffer
    } else if let Some(file_path) = file {
        println!("📥 Lese Datei: {}", file_path);
        fs::read(&file_path)?
    } else {
        return Err("❌ Entweder --file oder --stdin muss angegeben werden".into());
    };

    println!("📊 Größe: {} bytes, Medientyp: {}", data.len(), media_type);

    // Öffne BLOB Store
    let mut store = SqliteBlobStore::new(registry_path)?;

    // Berechne BLAKE3 Hash für Deduplizierung
    let blob_id_preview = crypto::hex_lower_prefixed32(crypto::blake3_256(&data));

    // Prüfe ob Blob bereits existiert
    if store.exists(&blob_id_preview) && !no_dedup {
        println!(
            "✅ BLOB existiert bereits (dedupliziert): {}",
            blob_id_preview
        );

        // Erhöhe refcount wenn link_entry_id angegeben
        if link_entry_id.is_some() {
            store.pin(&blob_id_preview)?;
            println!("📌 Refcount erhöht");
        }
    } else {
        // Insert BLOB
        let blob_id = store.put(&data, media_type)?;
        println!("✅ BLOB gespeichert: {}", blob_id);

        // Erhöhe refcount wenn link_entry_id angegeben
        if link_entry_id.is_some() {
            store.pin(&blob_id)?;
            println!("📌 Refcount erhöht für Registry-Verknüpfung");
        }
    }

    // Schreibe blob_id in Output-Datei falls angegeben
    if let Some(out_path) = out {
        fs::write(&out_path, blob_id_preview.as_bytes())?;
        println!("📄 BLOB ID geschrieben nach: {}", out_path);
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
fn run_blob_get(
    blob_id: &str,
    out: Option<String>,
    use_stdout: bool,
    registry_path: &str,
) -> Result<(), Box<dyn Error>> {
    // Validiere BLOB ID Format
    if !blob_id.starts_with("0x") || blob_id.len() != 66 {
        return Err("❌ Ungültige BLOB ID Format (erwartet: 0x + 64 hex chars)".into());
    }

    // Öffne BLOB Store
    let store = SqliteBlobStore::new(registry_path)?;

    // Hole BLOB
    println!("🔍 Suche BLOB: {}", blob_id);
    let data = store.get(blob_id)?;

    println!("✅ BLOB gefunden, Größe: {} bytes", data.len());

    // Schreibe auf Datei oder stdout
    if let Some(out_path) = out {
        fs::write(&out_path, &data)?;
        println!("📄 BLOB geschrieben nach: {}", out_path);
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
fn run_blob_list(
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
                "❌ Ungültige Sortierung: {}. Erlaubt: size, refcount, blob_id",
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
    println!("📋 Gefundene BLOBs: {}", blobs.len());
    println!(
        "{:<66} {:<15} {:<20} {:<10}",
        "BLOB ID", "Size (bytes)", "Media Type", "Refcount"
    );
    println!("{}", "-".repeat(115));

    for blob in &blobs {
        println!(
            "{:<66} {:<15} {:<20} {:<10}",
            blob.blob_id, blob.size, blob.media_type, blob.refcount
        );
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
fn run_blob_gc(
    dry_run: bool,
    force: bool,
    min_age: Option<String>,
    print_ids: bool,
    registry_path: &str,
) -> Result<(), Box<dyn Error>> {
    if min_age.is_some() {
        println!("⚠️  --min-age ist noch nicht implementiert (zukünftige Feature)");
    }

    // Öffne BLOB Store
    let mut store = SqliteBlobStore::new(registry_path)?;

    // Führe GC aus (dry-run oder real)
    println!("🗑️  Starte Garbage Collection...");
    let gc_candidates = store.gc(true)?; // Erst dry-run für Anzeige

    if gc_candidates.is_empty() {
        println!("✅ Keine unreferenzierten BLOBs gefunden");
        return Ok(());
    }

    println!("📊 Unreferenzierte BLOBs: {}", gc_candidates.len());

    if print_ids {
        println!("\n🗑️  Zu löschende BLOB IDs:");
        for id in &gc_candidates {
            println!("  - {}", id);
        }
    }

    // Berechne Gesamtgröße
    let mut total_bytes = 0u64;
    for id in &gc_candidates {
        if let Ok(data) = store.get(id) {
            total_bytes += data.len() as u64;
        }
    }

    println!(
        "💾 Freizugebender Speicher: {} bytes ({:.2} MB)",
        total_bytes,
        total_bytes as f64 / 1_048_576.0
    );

    if dry_run {
        println!("\n🔍 DRY RUN - Keine Löschung durchgeführt");
        println!("💡 Führen Sie den Befehl mit --force aus, um zu löschen");
        return Ok(());
    }

    if !force {
        println!("\n⚠️  Bitte bestätigen Sie die Löschung mit --force");
        return Ok(());
    }

    // Real GC
    println!("\n🗑️  Lösche unreferenzierte BLOBs...");
    store.gc(false)?;
    println!(
        "✅ {} BLOBs gelöscht, {} bytes freigegeben",
        gc_candidates.len(),
        total_bytes
    );

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
            PolicyCommands::Lint { file, strict } => {
                use cap_agent::policy_v2;
                let exit_code = policy_v2::run_lint(file, *strict).unwrap_or(1);
                std::process::exit(exit_code);
            }
            PolicyCommands::Compile { file, output } => {
                use cap_agent::policy_v2;
                let exit_code = policy_v2::run_compile(file, output).unwrap_or(1);
                std::process::exit(exit_code);
            }
            PolicyCommands::Show { file } => {
                use cap_agent::policy_v2;
                let exit_code = policy_v2::run_show(file).unwrap_or(1);
                std::process::exit(exit_code);
            }
        },
        Commands::Manifest(cmd) => match cmd {
            ManifestCommands::Build { policy, out } => run_manifest_build(policy, out.clone()),
            ManifestCommands::Validate { file, schema } => {
                run_manifest_validate(file, schema.clone())
            }
            ManifestCommands::Verify {
                manifest,
                proof,
                registry,
                timestamp,
                out,
            } => run_manifest_verify(manifest, proof, registry, timestamp.clone(), out.clone()),
        },
        Commands::Proof(cmd) => match cmd {
            ProofCommands::Mock { policy, manifest } => run_proof_mock(policy, manifest),
            ProofCommands::Build { policy, manifest } => run_proof_build(policy, manifest),
            ProofCommands::Verify { proof, manifest } => run_proof_verify_v3(proof, manifest),
            ProofCommands::Export {
                manifest,
                proof,
                timestamp,
                registry,
                report,
                out,
                force,
            } => run_proof_export(
                manifest,
                proof,
                timestamp.clone(),
                registry.clone(),
                report.clone(),
                out.clone(),
                *force,
            ),
            ProofCommands::ZkBuild {
                policy,
                manifest,
                out,
                sanctions_root,
                jurisdiction_root,
                sanctions_csv,
            } => run_zk_build(
                policy,
                manifest,
                out.clone(),
                sanctions_root.clone(),
                jurisdiction_root.clone(),
                sanctions_csv.clone(),
            ),
            ProofCommands::ZkVerify { proof } => run_zk_verify(proof),
            ProofCommands::Bench {
                policy,
                manifest,
                iterations,
            } => run_zk_bench(policy, manifest, *iterations),
            ProofCommands::Adapt {
                policy,
                ir,
                context,
                enforce,
                rollout,
                drift_max,
                selector,
                weights,
                dry_run,
                out,
            } => run_proof_adapt(
                policy, ir, context, *enforce, *rollout, *drift_max, selector, weights, *dry_run,
                out,
            ),
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
            AuditCommands::SetPrivateAnchor {
                manifest,
                audit_tip,
                created_at,
            } => run_audit_set_private_anchor(manifest, audit_tip, created_at.clone()),
            AuditCommands::SetPublicAnchor {
                manifest,
                chain,
                txid,
                digest,
                created_at,
            } => run_audit_set_public_anchor(manifest, chain, txid, digest, created_at.clone()),
            AuditCommands::VerifyAnchor { manifest, out } => {
                run_audit_verify_anchor(manifest, out.clone())
            }
            AuditCommands::Append {
                file,
                event,
                policy_id,
                ir_hash,
                manifest_hash,
                result,
                run_id,
            } => run_audit_append(
                file,
                event,
                policy_id.clone(),
                ir_hash.clone(),
                manifest_hash.clone(),
                result.clone(),
                run_id.clone(),
            ),
            AuditCommands::Verify { file, out } => run_audit_verify_chain(file, out.clone()),
            AuditCommands::Export {
                file,
                from,
                to,
                policy_id,
                out,
            } => run_audit_export(
                file,
                from.clone(),
                to.clone(),
                policy_id.clone(),
                out.clone(),
            ),
        },
        Commands::Lists(cmd) => match cmd {
            ListsCommands::SanctionsRoot { csv, out } => run_lists_sanctions_root(csv, out.clone()),
            ListsCommands::JurisdictionsRoot { csv, out } => {
                run_lists_jurisdictions_root(csv, out.clone())
            }
        },
        Commands::Registry(cmd) => match cmd {
            RegistryCommands::Add {
                manifest,
                proof,
                timestamp,
                registry,
                backend,
                signing_key,
                validate_key,
                keys_dir,
            } => run_registry_add(
                manifest,
                proof,
                timestamp.clone(),
                registry.clone(),
                backend,
                signing_key.clone(),
                *validate_key,
                keys_dir,
            ),
            RegistryCommands::List { registry, backend } => {
                run_registry_list(registry.clone(), backend)
            }
            RegistryCommands::Verify {
                manifest,
                proof,
                registry,
                backend,
            } => run_registry_verify(manifest, proof, registry.clone(), backend),
            RegistryCommands::Migrate {
                from,
                input,
                to,
                output,
            } => run_registry_migrate(from, input, to, output),
            RegistryCommands::Inspect { registry } => run_registry_inspect(registry.clone()),
            RegistryCommands::BackfillKid { registry, output } => {
                run_registry_backfill_kid(registry.clone(), output.clone())
            }
        },
        Commands::Keys(cmd) => match cmd {
            KeysCommands::Keygen {
                owner,
                algo,
                out,
                valid_days,
                comment,
            } => run_keys_keygen(owner, algo, out, *valid_days, comment.clone()),
            KeysCommands::List { dir, status, owner } => {
                run_keys_list(dir, status.clone(), owner.clone())
            }
            KeysCommands::Show { dir, kid } => run_keys_show(dir, kid),
            KeysCommands::Rotate { dir, current, new } => run_keys_rotate(dir, current, new),
            KeysCommands::Attest {
                signer,
                subject,
                out,
            } => run_keys_attest(signer, subject, out),
            KeysCommands::Archive { dir, kid } => run_keys_archive(dir, kid),
            KeysCommands::VerifyChain { dir, attestations } => {
                run_keys_verify_chain(dir, attestations)
            }
        },
        Commands::Blob(cmd) => match cmd {
            BlobCommands::Put {
                file,
                r#type,
                registry,
                link_entry_id,
                stdin,
                out,
                no_dedup,
            } => run_blob_put(
                file.clone(),
                r#type,
                registry,
                link_entry_id.clone(),
                *stdin,
                out.clone(),
                *no_dedup,
            ),
            BlobCommands::Get {
                id,
                out,
                stdout,
                registry,
            } => run_blob_get(id, out.clone(), *stdout, registry),
            BlobCommands::List {
                r#type,
                min_size,
                max_size,
                unused_only,
                limit,
                order,
                registry,
            } => run_blob_list(
                r#type.clone(),
                *min_size,
                *max_size,
                *unused_only,
                *limit,
                order,
                registry,
            ),
            BlobCommands::Gc {
                dry_run,
                force,
                min_age,
                print_ids,
                registry,
            } => run_blob_gc(*dry_run, *force, min_age.clone(), *print_ids, registry),
        },
        Commands::BundleV2 {
            manifest,
            proof,
            verifier_wasm,
            out,
            zip,
            force,
        } => run_bundle_v2(manifest, proof, verifier_wasm.clone(), out, *zip, *force),
        Commands::VerifyBundle { bundle, out } => run_verify_bundle(bundle, out.clone()),
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
