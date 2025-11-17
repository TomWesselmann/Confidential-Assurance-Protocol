# 03 - Komponenten & Module

## 📖 Über dieses Kapitel

Nachdem Sie in [Kapitel 02](./02-architecture.md) den **Aufbau** des Systems kennengelernt haben (die "Stockwerke"), zeigt dieses Kapitel den **detaillierten Inventar** aller Teile.

**Für wen ist dieses Kapitel?**
- **Management:** Die Management-Zusammenfassung am Anfang jeder Kategorie
- **IT-Leiter:** Überblick über technische Komponenten
- **Entwickler:** Detaillierte Modul-Dokumentation mit Funktionen und Datenstrukturen

**Was Sie lernen werden:**
1. Welche 65+ Module es gibt
2. Was jedes Modul macht (in einfachen Worten)
3. Wie die Module zusammenarbeiten

**Analogie:** Stellen Sie sich vor, Sie haben einen Bauplan eines Hauses gesehen (Kapitel 02). Jetzt sehen Sie die **Teile-Liste**: Welche Türen, Fenster, Rohre, Kabel verbaut sind.

---

## 👔 Für Management: Die große Inventur

Das System besteht aus **65+ spezialisierten Komponenten**, organisiert in **15 Kategorien**:

| Kategorie | Anzahl Module | Analogie | Zweck |
|-----------|---------------|----------|-------|
| **API Layer** | 7 | Empfangsschalter | Nimmt Anfragen entgegen |
| **Core Processing** | 9 | Produktionshalle | Erstellt Nachweise |
| **Verification** | 3 | Prüfstelle | Prüft Nachweise |
| **Registry** | 5 | Archiv | Speichert Nachweise-Liste |
| **Key Management** | 1 | Tresor | Verwaltet Schlüssel |
| **BLOB Store** | 1 | Lager | Speichert große Dateien |
| **Cryptography** | 1 | Verschlüsselungsmaschine | Hashes & Signaturen |
| **Policy V2** | 7 | Regelwerk-Verwaltung | Verwaltet Compliance-Regeln |
| **Orchestrator** | 6 | Dirigent | Koordiniert Abläufe |
| **WASM** | 2 | Plugin-System | Erweiterungen |
| **Proof Format** | 1 | Verpackung | Standardisiert Nachweise |
| **Key Providers** | 4 | Schlüssel-Speicher | Verschiedene Speicherorte |
| **Lists** | 3 | Referenz-Listen | Sanktionslisten etc. |
| **Support** | 6 | Hilfssysteme | Logging, Metrics |
| **CLI Binary** | 1 | Kommandozentrale | Befehlseingabe |

**Warum so viele Komponenten?**
- **Spezialisierung:** Jedes Modul macht eine Sache richtig gut
- **Wartbarkeit:** Defekte Module können einzeln ersetzt werden
- **Sicherheit:** Kleinere Module sind einfacher zu prüfen
- **Wiederverwendung:** Module können in anderen Projekten genutzt werden

**Kritische Komponenten** (die "Herzstücke"):
1. **crypto/mod.rs** - Alle Verschlüsselung (ohne dies geht nichts)
2. **core/proof_engine.rs** - Erstellt Nachweise
3. **verifier/core.rs** - Prüft Nachweise
4. **api/auth.rs** - Sichert den Zugang

---

## Übersicht aller Rust-Module

**Technischer Hinweis:** "Rust" ist die Programmiersprache, "Module" sind einzelne Code-Dateien.

Der LsKG-Agent besteht aus **65+ Rust-Modulen** in verschiedenen Kategorien.

## 1. API Layer (REST API v0.11.0)

### api/mod.rs
**Zweck:** Module aggregation für API-Layer
**Hauptfunktionen:**
- Re-exports aller API-Module
- API-Versionskonstanten

### api/auth.rs
**Zweck:** OAuth2 JWT-Validierung
**Hauptstrukturen:**
```rust
struct Claims {
    sub: String,        // Subject (user/client ID)
    iss: String,        // Issuer
    aud: String,        // Audience
    exp: usize,         // Expiration (Unix timestamp)
    iat: usize,         // Issued at
    scope: String,      // Space-separated scopes
}

struct OAuth2Config {
    issuer: String,
    audience: String,
    decode_key: DecodingKey,
    required_scopes: Vec<String>,
}
```

**Hauptfunktionen:**
- `validate_token(token: &str, config: &OAuth2Config) -> Result<Claims>` - JWT RS256 validation
- `extract_bearer_token(auth_header: &str) -> Result<String>` - Extract token from "Bearer xxx"
- `load_oauth2_config(path: &Path) -> Result<OAuth2Config>` - Load config from YAML

**Verwendung:**
```rust
let claims = validate_token(&token, &config)?;
if claims.exp < now() {
    return Err(AuthError::TokenExpired);
}
```

---

### api/verify.rs
**Zweck:** REST-Handler für Proof-Verifikation
**Hauptstrukturen:**
```rust
struct VerifyRequest {
    policy_id: String,
    context: VerifyContext,
    backend: String,              // "mock", "zkvm", "halo2"
    options: Option<VerifyRequestOptions>,
}

struct VerifyContext {
    manifest: Manifest,
    proof: Option<Proof>,
}

struct VerifyResponse {
    result: String,               // "ok", "fail", "warn"
    manifest_hash: String,
    proof_hash: String,
    report: VerifyReport,
}
```

**Hauptfunktionen:**
- `handle_verify(req: VerifyRequest) -> Result<VerifyResponse>` - Main handler
- `extract_and_hash(manifest: &Manifest) -> String` - Compute manifest hash

**Request Flow:**
```
1. Parse JSON request
2. Load manifest & proof
3. Call verifier::core::verify()
4. Update Prometheus metrics
5. Return JSON response
```

---

### api/policy.rs
**Zweck:** Policy Management REST-Handlers
**Hauptstrukturen:**
```rust
struct PolicyCompileRequest {
    policy: Policy,
}

struct PolicyCompileResponse {
    policy_hash: String,
    policy_info: PolicyInfo,
    status: String,
}

struct PolicyGetResponse {
    policy_hash: String,
    policy: Policy,
}
```

**Endpoints:**
- `POST /policy/compile` - Validate & hash policy
- `GET /policy/:id` - Retrieve policy by hash

---

### api/policy_compiler.rs
**Zweck:** PolicyV2 Compiler Integration
**Hauptfunktionen:**
- `compile_policy(policy: &Policy) -> Result<PolicyInfo>` - Compile & validate
- `compute_policy_hash(policy: &Policy) -> String` - SHA3-256 hash

---

### api/metrics_middleware.rs
**Zweck:** Prometheus Metrics Collection
**Metrics:**
```
cap_verifier_requests_total{result="ok|fail|warn"} counter
cap_verifier_request_duration_seconds histogram
cap_auth_token_validation_failures_total counter
cap_cache_hit_ratio gauge
```

**Middleware:**
```rust
pub fn metrics_middleware() -> impl tower::Layer {
    // Wraps all handlers
    // Increments counters on success/failure
    // Records latency histograms
}
```

---

### api/tls.rs
**Zweck:** TLS/mTLS Configuration
**Hauptstrukturen:**
```rust
enum TlsMode {
    Disabled,
    Tls,      // Server cert only
    Mtls,     // Mutual TLS
}

struct TlsConfig {
    mode: TlsMode,
    cert_path: PathBuf,
    key_path: PathBuf,
    ca_path: Option<PathBuf>,  // For mTLS
}
```

**Hauptfunktionen:**
- `load_tls_config(path: &Path) -> Result<TlsConfig>`
- `build_rustls_config(config: &TlsConfig) -> Result<ServerConfig>`

---

### bin/verifier_api.rs
**Zweck:** REST API Server Binary
**Hauptfunktionen:**
- Axum router setup
- OAuth2 middleware installation
- TLS/mTLS configuration
- Health/readiness/metrics endpoints
- Route handlers

**Server Setup:**
```rust
#[tokio::main]
async fn main() {
    // Load config
    let config = load_config("config/app.yaml")?;

    // Build router
    let app = Router::new()
        .route("/healthz", get(health_check))
        .route("/readyz", get(readiness_check))
        .route("/metrics", get(prometheus_metrics))
        .route("/verify", post(handle_verify))
        .route("/policy/compile", post(handle_policy_compile))
        .route("/policy/:id", get(handle_policy_get))
        .layer(auth_middleware())
        .layer(metrics_middleware());

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], 8443));
    axum_server::bind_rustls(addr, tls_config)
        .serve(app.into_make_service())
        .await?;
}
```

---

## 2. Core Processing Layer

### core/io.rs
**Zweck:** CSV Import für Supplier & UBO Daten
**Hauptstrukturen:**
```rust
struct Supplier {
    name: String,
    jurisdiction: String,
    tier: String,
}

struct Ubo {
    name: String,
    birthdate: String,
    citizenship: String,
}
```

**Hauptfunktionen:**
- `read_suppliers_csv(path: &Path) -> Result<Vec<Supplier>>`
- `read_ubos_csv(path: &Path) -> Result<Vec<Ubo>>`

**Validierung:**
- Pflichtfelder prüfen
- UTF-8 encoding
- Delimiter: `,` (Komma)

---

### core/commitment.rs
**Zweck:** BLAKE3 Merkle Root Berechnung
**Hauptstrukturen:**
```rust
struct Commitments {
    supplier_root: String,              // 0x + 64 hex
    ubo_root: String,
    company_commitment_root: String,
    supplier_count: Option<usize>,
    ubo_count: Option<usize>,
}
```

**Hauptfunktionen:**
- `compute_supplier_root(suppliers: &[Supplier]) -> Result<String>`
- `compute_ubo_root(ubos: &[Ubo]) -> Result<String>`
- `compute_company_root(company_data: &str) -> Result<String>`

**Algorithmus:**
```
1. For each record: hash = BLAKE3(json_serialize(record))
2. Merkle root = BLAKE3(hash_1 || hash_2 || ... || hash_N)
3. Format: "0x" + hex(root)
```

---

### core/audit/mod.rs
**Zweck:** Audit Controller
**Hauptstrukturen:**
```rust
struct AuditLog {
    events: Vec<AuditEntry>,
    tail_digest: String,
}
```

**Hauptfunktionen:**
- `log_event(event_type: &str, payload: serde_json::Value) -> Result<()>`
- `get_tail_digest() -> String`
- `load_audit_log(path: &Path) -> Result<AuditLog>`

---

### core/audit/v1_0.rs
**Zweck:** Audit Entry Schema v1.0
**Hauptstrukturen:**
```rust
struct AuditEntry {
    timestamp: String,              // RFC3339
    event_type: String,             // "prepare", "manifest_build", etc.
    payload: serde_json::Value,
    previous_digest: String,        // SHA3-256 of previous entry
}
```

**Event Types:**
- `prepare` - CSV import
- `manifest_build` - Manifest creation
- `proof_build` - Proof generation
- `sign` - Signature creation
- `export` - Package export
- `registry_add` - Registry insertion

---

### core/audit/hash_chain.rs
**Zweck:** SHA3-256 Hash-Chain für Audit Trail
**Hauptstrukturen:**
```rust
struct HashChain {
    chain: Vec<String>,             // Digests
}
```

**Hauptfunktionen:**
- `append(event: &AuditEntry) -> String` - Add event, return new digest
- `verify() -> bool` - Verify chain integrity

**Hash Calculation:**
```rust
digest = SHA3(previous_digest || timestamp || event_type || payload)
```

---

### core/policy.rs
**Zweck:** Policy Schema & Validation
**Hauptstrukturen:**
```rust
struct Policy {
    version: String,                    // "lksg.v1"
    name: String,
    created_at: String,                 // RFC3339
    constraints: PolicyConstraints,
    notes: String,
}

struct PolicyConstraints {
    require_at_least_one_ubo: bool,
    supplier_count_max: u32,
    ubo_count_min: Option<u32>,
    require_statement_roots: Option<Vec<String>>,
}

struct PolicyInfo {
    name: String,
    version: String,
    hash: String,                       // SHA3-256
}
```

**Hauptfunktionen:**
- `load_policy(path: &Path) -> Result<Policy>`
- `validate_policy(policy: &Policy) -> Result<()>`
- `compute_policy_hash(policy: &Policy) -> String`

---

### core/manifest.rs
**Zweck:** Manifest Builder
**Hauptstrukturen:**
```rust
struct Manifest {
    version: String,                    // "manifest.v1.0"
    created_at: String,                 // RFC3339
    supplier_root: String,
    ubo_root: String,
    company_commitment_root: String,
    policy: PolicyInfo,
    audit: AuditInfo,
    proof: ProofInfo,
    signatures: Vec<SignatureInfo>,
    time_anchor: Option<TimeAnchor>,
}
```

**Hauptfunktionen:**
- `build_manifest(commitments: &Commitments, policy: &Policy, audit: &AuditLog) -> Result<Manifest>`
- `add_time_anchor(manifest: &mut Manifest, anchor: TimeAnchor)`
- `compute_manifest_hash(manifest: &Manifest) -> String`

---

### core/sign.rs
**Zweck:** Ed25519 Signing & Verification
**Hauptfunktionen:**
- `generate_keypair() -> (SecretKey, PublicKey)`
- `sign_manifest(manifest: &str, secret_key: &[u8; 32]) -> Result<String>`
- `verify_signature(manifest: &str, signature: &str, public_key: &[u8; 32]) -> Result<bool>`

**Signature Format:**
- Ed25519 (64 bytes)
- Encoding: "0x" + hex

---

### core/proof_engine.rs
**Zweck:** Proof Generation
**Hauptstrukturen:**
```rust
struct Proof {
    version: String,                    // "proof.v0"
    type: String,                       // "mock", "zkvm", "halo2"
    statement: String,
    manifest_hash: String,
    policy_hash: String,
    proof_data: ProofData,
    status: String,
}

struct ProofData {
    checked_constraints: Vec<ConstraintCheck>,
}
```

**Hauptfunktionen:**
- `build_proof(manifest: &Manifest, policy: &Policy, backend: &str) -> Result<Proof>`
- `verify_proof(proof: &Proof, manifest: &Manifest) -> Result<bool>`

---

### core/proof_mock.rs
**Zweck:** Mock Proof Backend (Legacy)
**Hauptfunktionen:**
- `generate_mock_proof(statement: &str) -> ProofData`

---

### core/zk_system.rs
**Zweck:** ZK Backend Abstraction
**Trait:**
```rust
trait ProofSystem {
    fn backend_name(&self) -> &str;
    fn verify(&self, proof_data: &ProofData, statement: &str) -> Result<bool>;
}
```

**Implementierungen:**
- `MockZK` - Mock backend (Phase 1)
- `Halo2ZK` - Halo2 backend (Phase 3, stub)
- `SpartanZK` - Spartan backend (Phase 4, planned)

**Factory:**
```rust
fn backend_factory(backend: &str) -> Box<dyn ProofSystem>
```

---

## 3. Verification Layer

### verifier/core.rs
**Zweck:** I/O-freier Verifikationskern
**Hauptfunktionen:**
- `verify(manifest_hash: &str, proof_hash: &str, signature: Option<&str>, public_key: Option<&str>) -> VerifyReport`
- `extract_statement_from_manifest(manifest: &Manifest) -> String`

**VerifyReport:**
```rust
struct VerifyReport {
    status: String,                     // "ok", "fail", "warn"
    manifest_hash: String,
    proof_hash: String,
    signature_valid: bool,
    details: Vec<String>,
}
```

---

### verifier/mod.rs
**Zweck:** Package Verifier (I/O-basiert)
**Hauptstrukturen:**
```rust
struct Verifier {
    package_path: PathBuf,
}
```

**Hauptfunktionen:**
- `check_package_integrity(path: &Path) -> Result<()>`
- `show_package_summary(path: &Path) -> Result<()>`
- `verify_package(path: &Path) -> Result<VerifyReport>`

**Package Structure:**
```
cap-proof/
├── _meta.json              # Hashes of all files
├── manifest.json
├── proof.dat
├── proof.json
├── timestamp.txt (optional)
├── registry.json (optional)
└── README.md
```

---

### package_verifier.rs
**Zweck:** CLI Package Verifier (Binary-only)
Verwendet `verifier/mod.rs` für I/O-basierte Verifikation.

---

## 4. Registry Layer

### registry/mod.rs
**Zweck:** Registry Backend Abstraction
**Trait:**
```rust
trait RegistryStore {
    fn add(&mut self, entry: RegistryEntry) -> Result<()>;
    fn find_by_hashes(&self, manifest_hash: &str, proof_hash: &str) -> Result<Option<RegistryEntry>>;
    fn list(&self, limit: usize, offset: usize) -> Result<Vec<RegistryEntry>>;
}
```

**Implementierungen:**
- `JsonRegistryStore` - JSON file backend (simple)
- `SqliteRegistryStore` - SQLite backend (performant)

---

### registry/v1_0.rs
**Zweck:** Registry Entry Schema v1.0
**Hauptstrukturen:**
```rust
struct RegistryEntry {
    id: String,                         // UUID
    manifest_hash: String,
    proof_hash: String,
    timestamp: String,                  // RFC3339
    status: String,
    signature: Option<String>,          // Base64 Ed25519
    public_key: Option<String>,
    kid: Option<String>,                // 32 hex chars (v0.10)
    signature_scheme: Option<String>,   // "ed25519"
}
```

---

### registry/schema.rs
**Zweck:** SQLite Schema Definition
**Tables:**
```sql
CREATE TABLE IF NOT EXISTS registry_entries (
    id TEXT PRIMARY KEY,
    manifest_hash TEXT NOT NULL,
    proof_hash TEXT NOT NULL,
    timestamp TEXT NOT NULL,
    status TEXT NOT NULL,
    signature TEXT,
    public_key TEXT,
    kid TEXT,
    signature_scheme TEXT
);

CREATE INDEX idx_manifest_proof ON registry_entries(manifest_hash, proof_hash);
CREATE INDEX idx_timestamp ON registry_entries(timestamp);
CREATE INDEX idx_kid ON registry_entries(kid);
```

**Configuration:**
- WAL mode (Write-Ahead Logging)
- PRAGMA synchronous = NORMAL
- PRAGMA journal_mode = WAL

---

### registry/migrate.rs
**Zweck:** Backend Migration (JSON ↔ SQLite)
**Hauptfunktionen:**
- `migrate_json_to_sqlite(json_path: &Path, sqlite_path: &Path) -> Result<()>`
- `migrate_sqlite_to_json(sqlite_path: &Path, json_path: &Path) -> Result<()>`

---

### registry/api.rs
**Zweck:** Registry REST API Handlers
**Endpoints:**
- `POST /registry/add` - Add entry
- `GET /registry/find?manifest_hash=X&proof_hash=Y` - Find by hashes
- `GET /registry/list?limit=10&offset=0` - List entries

---

## 5. Key Management Layer (v0.10)

### keys.rs
**Zweck:** Key Management & KID Derivation
**Hauptstrukturen:**
```rust
struct KeyMetadata {
    schema: String,                     // "cap-key.v1"
    kid: String,                        // 32 hex chars
    owner: String,
    created_at: String,
    valid_from: String,
    valid_to: String,
    algorithm: String,                  // "ed25519"
    status: String,                     // "active", "retired", "revoked"
    usage: Vec<String>,                 // ["signing", "registry"]
    public_key: String,                 // Base64
    fingerprint: String,                // SHA-256 (first 16 bytes)
    comment: Option<String>,
}

struct KeyStoreEntry {
    kid: String,
    metadata_path: String,
    private_key_path: String,
    public_key_path: String,
    status: String,
}

struct Attestation {
    attestation: AttestationData,
    signature: String,
    signer_public_key: String,
}
```

**Hauptfunktionen:**
- `derive_kid(public_key_base64: &str) -> String` - BLAKE3-based KID
- `load_key_metadata(path: &Path) -> Result<KeyMetadata>`
- `rotate_key(current: &Path, new: &Path) -> Result<()>`
- `attest_key(signer: &Path, subject: &Path) -> Result<Attestation>`
- `verify_chain(attestations: &[Attestation]) -> Result<bool>`

**KID Derivation:**
```rust
kid = hex(BLAKE3(base64(public_key))[0:16])  // First 128 bits
```

---

## 6. BLOB Store Layer

### blob_store.rs
**Zweck:** Content-Addressable BLOB Storage
**Trait:**
```rust
trait BlobStore {
    fn put(&mut self, data: &[u8]) -> Result<String>;           // Returns BLAKE3 hash
    fn get(&self, hash: &str) -> Result<Option<Vec<u8>>>;
    fn delete(&mut self, hash: &str) -> Result<()>;
    fn gc(&mut self) -> Result<usize>;                          // Garbage collection
}
```

**Implementierungen:**
- `FileBlobStore` - Filesystem-based (default)
- `SqliteBlobStore` - SQLite-based (planned)

**Features:**
- Content-addressable (BLAKE3)
- Deduplication (same data → same hash)
- Reference counting
- Garbage collection (removes unreferenced blobs)

---

## 7. Cryptography Layer

### crypto/mod.rs
**Zweck:** Zentralisierte Krypto-API
**Hauptfunktionen:**

**Hashing:**
```rust
fn sha3_256(data: &[u8]) -> [u8; 32]
fn blake3_256(data: &[u8]) -> [u8; 32]
fn blake3_hash_str(data: &str) -> String     // Returns "0x..."
```

**Ed25519:**
```rust
struct Ed25519SecretKey([u8; 32]);
struct Ed25519PublicKey([u8; 32]);
struct Ed25519Signature([u8; 64]);

fn ed25519_generate() -> (Ed25519SecretKey, Ed25519PublicKey)
fn ed25519_sign(secret: &Ed25519SecretKey, msg: &[u8]) -> Ed25519Signature
fn ed25519_verify(public: &Ed25519PublicKey, msg: &[u8], sig: &Ed25519Signature) -> bool
```

**Encoding:**
```rust
fn hex_lower_prefixed32(bytes: &[u8; 32]) -> String  // "0x..."
fn hex_to_32b(hex: &str) -> Result<[u8; 32]>
fn base64_encode(data: &[u8]) -> String
fn base64_decode(data: &str) -> Result<Vec<u8>>
```

---

## 8. Policy V2 Layer

### policy_v2/types.rs
**Zweck:** PolicyV2 AST Types
**Hauptstrukturen:**
```rust
struct PolicyV2 {
    version: String,
    name: String,
    rules: Vec<Rule>,
}

struct Rule {
    name: String,
    condition: Expression,
    action: Action,
}

enum Expression {
    BinaryOp { left: Box<Expression>, op: Operator, right: Box<Expression> },
    Literal(Value),
    Variable(String),
}
```

---

### policy_v2/ir.rs
**Zweck:** Intermediate Representation (IR)
**Hauptstrukturen:**
```rust
struct IRProgram {
    statements: Vec<IRStatement>,
}

enum IRStatement {
    CheckConstraint { name: String, expression: IRExpression },
    Return(IRValue),
}
```

---

### policy_v2/linter.rs
**Zweck:** Policy Linting Engine
**Hauptfunktionen:**
- `lint_policy(policy: &PolicyV2) -> Vec<LintWarning>`

**Checks:**
- Unused variables
- Unreachable rules
- Type mismatches
- Constraint conflicts

---

### policy_v2/yaml_parser.rs
**Zweck:** YAML → PolicyV2 Parser
**Hauptfunktionen:**
- `parse_policy_yaml(path: &Path) -> Result<PolicyV2>`

---

### policy_v2/hasher.rs
**Zweck:** Policy Hash Calculation
**Hauptfunktionen:**
- `compute_policy_hash(policy: &PolicyV2) -> String` - SHA3-256

---

### policy_v2/cli.rs
**Zweck:** PolicyV2 CLI Integration
**Commands:**
- `policy compile <file>` - Compile & validate
- `policy lint <file>` - Run linter
- `policy hash <file>` - Compute hash

---

## 9. Orchestrator Layer

### orchestrator/mod.rs
**Zweck:** Adaptive Proof Orchestrator
**Hauptstrukturen:**
```rust
struct Orchestrator {
    selector: ProofSelector,
    planner: ExecutionPlanner,
    enforcer: PolicyEnforcer,
}
```

**Hauptfunktionen:**
- `select_backend(policy: &Policy, context: &Context) -> String`
- `plan_execution(policy: &Policy) -> ExecutionPlan`
- `enforce_constraints(policy: &Policy, data: &Data) -> Result<()>`

---

### orchestrator/selector.rs
**Zweck:** Risk-Based Proof Backend Selection
**Hauptfunktionen:**
- `select_backend(risk_level: u8) -> String`

**Logic:**
```
if risk_level >= 80 → "halo2" (real ZK)
else if risk_level >= 50 → "zkvm"
else → "mock"
```

---

### orchestrator/planner.rs
**Zweck:** Execution Graph Planning
**Hauptstrukturen:**
```rust
struct ExecutionPlan {
    steps: Vec<ExecutionStep>,
}

struct ExecutionStep {
    name: String,
    dependencies: Vec<String>,
}
```

---

### orchestrator/enforcer.rs
**Zweck:** Policy Constraint Enforcement
**Hauptfunktionen:**
- `enforce(policy: &Policy, data: &Data) -> Result<()>`

---

### orchestrator/drift_analysis.rs
**Zweck:** Drift Detection (Policy vs Data)
**Hauptfunktionen:**
- `analyze_drift(expected: &Policy, actual: &Data) -> DriftReport`

---

### orchestrator/metrics.rs
**Zweck:** Orchestrator Performance Metrics
**Metrics:**
- Backend selection duration
- Execution plan size
- Constraint enforcement failures

---

## 10. WASM Layer

### wasm/loader.rs
**Zweck:** WASM Module Loader
**Hauptstrukturen:**
```rust
struct WasmVerifier {
    engine: wasmtime::Engine,
    limits: WasmLimits,
}

struct WasmLimits {
    max_memory_bytes: u64,      // 100 MB
    max_execution_time_ms: u64, // 5000 ms
}
```

**Hauptfunktionen:**
- `load_wasm_module(path: &Path) -> Result<WasmModule>`
- `execute_verifier(module: &WasmModule, input: &[u8]) -> Result<bool>`

---

### wasm/executor.rs
**Zweck:** Bundle Executor
**Hauptstrukturen:**
```rust
struct BundleExecutor {
    config: ExecutorConfig,
}

struct ExecutorConfig {
    timeout: Duration,
    memory_limit: usize,
}
```

---

## 11. Proof Format Layer

### proof/capz.rs
**Zweck:** CAPZ v2 Binary Container Format
**Hauptstrukturen:**
```rust
struct CapzHeader {
    magic: [u8; 4],             // b"CAPZ"
    version: u16,               // 0x0002
    backend: u8,                // 0=mock, 1=zkvm, 2=halo2
    reserved: u8,
    vk_hash: [u8; 32],
    params_hash: [u8; 32],
    payload_len: u32,
}

struct CapzContainer {
    header: CapzHeader,
    payload: Vec<u8>,
}
```

**Format:**
```
Offset | Size | Field
-------|------|-------------
0      | 4    | magic (b"CAPZ")
4      | 2    | version (0x0002)
6      | 1    | backend
7      | 1    | reserved
8      | 32   | vk_hash
40     | 32   | params_hash
72     | 4    | payload_len
76     | 2    | padding
78     | N    | payload
```

**Hauptfunktionen:**
- `encode(proof: &Proof) -> Result<Vec<u8>>`
- `decode(data: &[u8]) -> Result<CapzContainer>`

---

## 12. Key Provider Layer

### providers/key_provider.rs
**Zweck:** Key Provider Trait
**Trait:**
```rust
trait KeyProvider {
    fn sign(&self, data: &[u8]) -> Result<Vec<u8>>;
    fn public_key(&self) -> Result<Vec<u8>>;
    fn provider_name(&self) -> &str;
}
```

---

### providers/software.rs
**Zweck:** Software Key Provider (In-Memory)
**Implementierung:**
```rust
struct SoftwareProvider {
    secret_key: Ed25519SecretKey,
    public_key: Ed25519PublicKey,
}
```

---

### providers/pkcs11.rs
**Zweck:** HSM/TPM Provider (PKCS#11)
**Status:** Phase 3 (stub)

---

### providers/cloudkms.rs
**Zweck:** Google Cloud KMS Provider
**Status:** Phase 3 (stub)

---

## 13. Lists Layer

### lists/sanctions.rs
**Zweck:** Sanctions Lists (OFAC, EU, UN)
**Hauptfunktionen:**
- `load_sanctions_list(source: &str) -> Result<Vec<Entity>>`
- `check_entity(name: &str, list: &[Entity]) -> bool`

---

### lists/jurisdictions.rs
**Zweck:** Jurisdiction Mappings
**Hauptfunktionen:**
- `load_jurisdictions() -> Result<HashMap<String, Jurisdiction>>`
- `get_jurisdiction_info(code: &str) -> Option<Jurisdiction>`

---

## 14. Support Modules

### auth/mod.rs & auth/errors.rs
**Zweck:** Auth Utilities & Errors
**Error Types:**
```rust
enum AuthError {
    MissingToken,
    InvalidToken,
    TokenExpired,
    InvalidClaims,
    InsufficientScopes,
}
```

---

### http/mod.rs & http/middleware/auth.rs
**Zweck:** HTTP Foundation & Middleware

---

### metrics/mod.rs
**Zweck:** Prometheus Metrics Export
**Metric Types:**
- Counter
- Histogram
- Gauge

---

### tls/mod.rs
**Zweck:** TLS/mTLS Configuration

---

## 15. CLI Binary

### main.rs
**Zweck:** CLI Entry Point
**Commands:**
```
cap prepare         - Import CSV data
cap policy          - Policy management
cap manifest        - Manifest operations
cap proof           - Proof generation/verification
cap sign            - Signing operations
cap export          - Package export
cap registry        - Registry management
cap keys            - Key management
cap verifier        - Verification operations
```

**Argument Parsing:** clap 4.5 with derive macros

---

## Zusammenfassung

Der LsKG-Agent besteht aus:
- **10+ Layer** mit klaren Verantwortlichkeiten
- **65+ Module** für verschiedene Funktionen
- **Trait-basierte Abstraktion** für Erweiterbarkeit
- **Type-Safe** durch Rust's starkes Typsystem
- **Testbar** durch klare Schnittstellen
