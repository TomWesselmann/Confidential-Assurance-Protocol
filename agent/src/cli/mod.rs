//! CLI Command Definitions für CAP-Agent
//!
//! Dieses Modul enthält alle Clap-basierten Subcommand-Enums und Handler.
//! Extrahiert aus main.rs für bessere Wartbarkeit.

pub mod audit;
pub mod blob;
pub mod bundle;
pub mod keys;
pub mod manifest;
pub mod output;
pub mod policy;
pub mod prepare;
pub mod proof;
pub mod registry;
pub mod sign;
pub mod verifier;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// LkSG Proof Agent - Confidential Assurance Protocol (CAP)
///
/// CLI-Tool zur Erzeugung kryptografisch prüfbarer Commitments
/// für Lieferketten- und Sanktionsprüfungen.
#[derive(Parser)]
#[command(name = "cap-agent")]
#[command(version = super::VERSION)]
#[command(about = "LkSG Proof Agent (Proof & Verifier Layer MVP)", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
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
pub enum PolicyCommands {
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
pub enum ManifestCommands {
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
pub enum ProofCommands {
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
        ir: Option<PathBuf>,

        /// Pfad zur Context-JSON-Datei
        #[arg(long)]
        context: PathBuf,

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
        weights: Option<PathBuf>,

        /// Dry-Run-Modus (keine Seiteneffekte)
        #[arg(long, default_value_t = false)]
        dry_run: bool,

        /// Output-Pfad für Execution Plan (JSON)
        #[arg(short = 'o', long)]
        out: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
pub enum SignCommands {
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
pub enum VerifierCommands {
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
pub enum AuditCommands {
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
pub enum ListsCommands {
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
pub enum RegistryCommands {
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
pub enum KeysCommands {
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
pub enum BlobCommands {
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
