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

Das System besteht aus **80+ spezialisierten Komponenten**, organisiert in **17 Kategorien**:

| Kategorie | Anzahl Module | Analogie | Zweck |
|-----------|---------------|----------|-------|
| **API Layer** | 8 | Empfangsschalter | Nimmt Anfragen entgegen (inkl. Upload, Rate Limiting) |
| **Core Processing** | 9 | Produktionshalle | Erstellt Nachweise |
| **Verification** | 3 | Prüfstelle | Prüft Nachweise |
| **Registry** | 5 | Archiv | Speichert Nachweise-Liste |
| **Key Management** | 1 | Tresor | Verwaltet Schlüssel |
| **BLOB Store** | 1 | Lager | Speichert große Dateien |
| **Cryptography** | 1 | Verschlüsselungsmaschine | Hashes & Signaturen |
| **Policy V2** | 7 | Regelwerk-Verwaltung | Verwaltet Compliance-Regeln |
| **Policy Store** | 4 | Policy-Datenbank | Persistente Policy-Speicherung (InMemory + SQLite) |
| **Orchestrator** | 6 | Dirigent | Koordiniert Abläufe |
| **WASM** | 2 | Plugin-System | Erweiterungen |
| **Proof Format** | 2 | Verpackung | Standardisiert Nachweise (cap-bundle.v1 mit SHA3-256 Hashes) |
| **Key Providers** | 4 | Schlüssel-Speicher | Verschiedene Speicherorte |
| **Lists** | 3 | Referenz-Listen | Sanktionslisten etc. |
| **Support** | 6 | Hilfssysteme | Logging, Metrics |
| **Web UI** | 7 | Benutzeroberfläche | React-basierte grafische Oberfläche (v0.11.0) |
| **Monitoring & Observability** | 8 | Überwachungszentrale | Production Monitoring Stack (Week 2) |
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

### api/upload.rs
**Zweck:** Multipart File Upload Handler für Proof Packages
**Hauptstrukturen:**
```rust
struct UploadResponse {
    manifest: Manifest,
    proof_base64: String,
    package_info: PackageInfo,
}

struct PackageInfo {
    total_size: usize,
    file_count: usize,
}
```

**Hauptfunktionen:**
- `handle_proof_upload(multipart: Multipart) -> Result<UploadResponse>`
- `extract_zip_contents(zip_data: &[u8]) -> Result<(Manifest, String)>`

**Request Flow:**
```
1. Receive multipart/form-data with ZIP file
2. Extract ZIP contents (manifest.json + proof.dat)
3. Parse manifest.json
4. Base64-encode proof.dat
5. Return UploadResponse with package info
```

**Integration:** Wird von WebUI verwendet für Drag & Drop Upload

---

### api/rate_limit.rs
**Zweck:** IP-basierte Rate Limiting Middleware (✅ Production Ready)
**Hauptstrukturen:**
```rust
struct RateLimitConfig {
    requests_per_minute: u32,
    burst_size: u32,
}
```

**Presets:**
- `default_global()` – 100 req/min, burst 120 (allgemeine API-Nutzung)
- `strict()` – 10 req/min, burst 15 (Policy Compilation)
- `moderate()` – 20 req/min, burst 25 (Verification)

**Hauptfunktionen:**
- `rate_limiter_layer(config: RateLimitConfig) -> impl tower::Layer`
- `handle_rate_limit_error() -> Response<StatusCode::TOO_MANY_REQUESTS>`

**Features:**
- Token Bucket Algorithm (GCRA via tower_governor)
- IP-basierte Limits (via X-Forwarded-For oder Socket Address)
- Standard Rate Limit Headers (X-RateLimit-Limit, X-RateLimit-Remaining, Retry-After)
- Per-Endpoint Rate Limits

**CLI Flags:**
- `--rate-limit <number>` – Requests pro Minute (default: 100)
- `--rate-limit-burst <number>` – Burst Size (default: 120)

**HTTP Response Headers:**
```
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 42
Retry-After: 15  (bei 429)
```

**Analogie (Management):** Wie ein Türsteher - lässt nur eine bestimmte Anzahl Kunden pro Minute rein, verhindert Überlastung

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

## 8a. Policy Store System (✅ v0.11.0)

**Management-Zusammenfassung:** Persistenter Speicher für Compliance-Policies mit automatischer Deduplizierung und Lifecycle-Management (Active/Deprecated/Draft).

### policy/metadata.rs
**Zweck:** Policy Metadata Strukturen
**Hauptstrukturen:**
```rust
enum PolicyStatus {
    Active,      // Policy ist aktiv und kann verwendet werden
    Deprecated,  // Policy ist veraltet, aber noch zugänglich
    Draft,       // Policy ist im Entwurfszustand
}

struct PolicyMetadata {
    id: Uuid,                    // UUID v4 Policy Identifier
    name: String,
    version: String,
    hash: String,                // SHA3-256 (0x-präfixiert, 64 hex)
    status: PolicyStatus,
    created_at: String,          // ISO 8601 Timestamp
    updated_at: String,          // ISO 8601 Timestamp
    description: Option<String>,
}

struct CompiledPolicy {
    metadata: PolicyMetadata,
    policy: Policy,              // Original Policy Definition
    compiled_bytes: Option<Vec<u8>>, // Optional compiled IR
}
```

**Verwendung:**
```rust
let metadata = PolicyMetadata {
    id: Uuid::new_v4(),
    name: "LkSG Demo Policy".to_string(),
    version: "lksg.v1".to_string(),
    hash: "0x1da941f7...".to_string(),
    status: PolicyStatus::Active,
    created_at: "2025-11-18T10:00:00Z".to_string(),
    updated_at: "2025-11-18T10:00:00Z".to_string(),
    description: Some("Test policy".to_string()),
};
```

---

### policy/store.rs
**Zweck:** PolicyStore Trait Interface
**Hauptfunktionen:**
```rust
#[async_trait]
trait PolicyStore: Send + Sync {
    // Speichert Policy, gibt Metadata zurück (deduplication via hash)
    async fn save(&self, policy: Policy) -> Result<PolicyMetadata>;

    // Ruft Policy nach UUID ab
    async fn get(&self, id: &str) -> Result<Option<CompiledPolicy>>;

    // Ruft Policy nach SHA3-256 Hash ab
    async fn get_by_hash(&self, hash: &str) -> Result<Option<CompiledPolicy>>;

    // Listet Policies auf, optional gefiltert nach Status
    async fn list(&self, status_filter: Option<PolicyStatus>) -> Result<Vec<PolicyMetadata>>;

    // Setzt Policy Status (Active → Deprecated → Draft)
    async fn set_status(&self, id: &str, status: PolicyStatus) -> Result<()>;
}
```

**Helper-Funktionen:**
```rust
// SHA3-256 Hash-Berechnung für Policy (deterministisch)
fn compute_policy_hash(policy: &Policy) -> Result<String>;

// RFC3339 Timestamp-Generierung (UTC)
fn now_iso8601() -> String;
```

**Verwendung:**
```rust
let store = InMemoryPolicyStore::new();
let policy = Policy { /* ... */ };

// Save (deduplication automatisch)
let metadata = store.save(policy).await?;

// Get by UUID
let compiled = store.get(&metadata.id.to_string()).await?;

// Get by Hash
let compiled = store.get_by_hash(&metadata.hash).await?;

// List active policies
let active = store.list(Some(PolicyStatus::Active)).await?;

// Deprecate policy
store.set_status(&metadata.id.to_string(), PolicyStatus::Deprecated).await?;
```

---

### policy/in_memory.rs
**Zweck:** Thread-Safe In-Memory Policy Store
**Hauptstrukturen:**
```rust
struct InMemoryPolicyStore {
    policies: Arc<Mutex<HashMap<String, CompiledPolicy>>>, // UUID → Policy
    hash_index: Arc<Mutex<HashMap<String, String>>>,        // Hash → UUID
}
```

**Features:**
- ✅ Thread-Safe via `Arc<Mutex<HashMap>>`
- ✅ O(1) Lookups (UUID + Hash)
- ✅ Automatische Deduplizierung via Policy-Hash
- ✅ Block Scoping für Mutex Guards (no deadlocks)
- ✅ Status Lifecycle Management

**Verwendung:**
```rust
let store = InMemoryPolicyStore::new();
// ... siehe PolicyStore trait
```

**Testabdeckung:**
- ✅ 7 Unit-Tests (save/get, hash lookup, deduplication, list, status, not_found)
- ✅ 6 API Integration Tests (compile, get, not_found, validation, concurrent access)

---

### policy/sqlite.rs
**Zweck:** Production-Ready SQLite Policy Store
**Hauptstrukturen:**
```rust
struct SqlitePolicyStore {
    conn: Arc<Mutex<Connection>>, // Thread-Safe SQLite Connection
}
```

**SQLite Schema:**
```sql
CREATE TABLE policies (
    id TEXT PRIMARY KEY,              -- UUID v4
    name TEXT NOT NULL,
    version TEXT NOT NULL,
    hash TEXT NOT NULL UNIQUE,        -- SHA3-256 (deduplication)
    status TEXT NOT NULL,             -- 'active', 'deprecated', 'draft'
    created_at TEXT NOT NULL,         -- ISO 8601
    updated_at TEXT NOT NULL,         -- ISO 8601
    description TEXT,
    policy_json TEXT NOT NULL,        -- Original Policy JSON
    compiled_bytes BLOB               -- Optional compiled IR
);

CREATE INDEX idx_policies_hash ON policies(hash);
CREATE INDEX idx_policies_status ON policies(status);
CREATE INDEX idx_policies_created_at ON policies(created_at DESC);
```

**Features:**
- ✅ WAL Mode (Write-Ahead Logging) für Concurrent Access
- ✅ PRAGMA synchronous=NORMAL (Performance-Optimierung)
- ✅ ACID-Transaktionen mit Rollback-Support
- ✅ Automatische Deduplizierung via UNIQUE constraint
- ✅ Persistent Storage mit Datenbank-Datei
- ✅ Thread-Safe via `Arc<Mutex<Connection>>`

**Migration:**
```sql
-- migrations/001_create_policies_table.sql
-- Automatisch angewendet bei Initialisierung
```

**Verwendung:**
```rust
let store = SqlitePolicyStore::new("/data/policies.sqlite")?;
// ... siehe PolicyStore trait
```

**Testabdeckung:**
- ✅ 7 Unit-Tests (save/get, hash lookup, deduplication, list, status, persistence, not_found)

---

### Integration Tests (tests/test_policy_store.rs)
**Zweck:** End-to-End Tests für Policy Store System
**Testabdeckung:**
- **InMemory Tests (7):**
  - `test_inmemory_save_and_get` - CRUD Operations
  - `test_inmemory_get_by_hash` - Hash-based Lookup
  - `test_inmemory_deduplication` - Content Deduplication
  - `test_inmemory_list` - Status Filtering
  - `test_inmemory_set_status` - Lifecycle Management
  - `test_inmemory_not_found` - Error Handling

- **SQLite Tests (7):**
  - `test_sqlite_save_and_get` - CRUD Operations
  - `test_sqlite_get_by_hash` - Hash-based Lookup
  - `test_sqlite_deduplication` - Content Deduplication
  - `test_sqlite_list` - Status Filtering
  - `test_sqlite_set_status` - Lifecycle Management
  - `test_sqlite_persistence` - Multi-Instance Persistence
  - `test_sqlite_not_found` - Error Handling

- **API Integration Tests (6):**
  - `test_api_policy_compile_and_get` - Compile + Get (UUID/Hash)
  - `test_api_policy_not_found` - 404 Error Handling
  - `test_api_policy_invalid_policy` - 400 Bad Request Validation
  - `test_api_policy_deduplication` - Content Deduplication
  - `test_api_sqlite_backend` - SQLite Backend Integration
  - `test_api_concurrent_access` - Thread-Safety (10 parallel requests)

**Status:** ✅ 19/19 Tests passed (0.02s execution time)

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

## 13. Proof Format Layer (cap-bundle.v1) ✨

**Management-Zusammenfassung:** Das cap-bundle.v1 Format ist das standardisierte Proof-Package-Format mit strukturierten Metadaten und SHA3-256 Hashes für alle Dateien. Es löst das Kompatibilitätsproblem zwischen `proof export` und `verifier run` und ermöglicht offline-verifizierbare Compliance-Nachweise.

**Problem (vorher):**
- `proof export` erstellte Pakete im alten Format (cap-proof.v1.0)
- `verifier run` erwartete neues Format (cap-bundle.v1)
- **Ergebnis:** Inkompatibilität, Tests schlugen fehl

**Lösung (jetzt):**
- Beide Tools sprechen die gleiche "Sprache" (cap-bundle.v1)
- Strukturierte Metadaten für jede Datei
- SHA3-256 Hashes für Integritätsprüfung
- Automatische Policy-Information-Extraktion aus Manifest

**Analogie (Management):** Wie ein standardisiertes Versandpaket mit detailliertem Lieferschein - vorher wusste man nur "ein Dokument ist drin", jetzt steht auf jedem Paket genau: "Dokument X, Größe 1.2KB, Prüfsumme ABC123, Rolle: Manifest"

---

### main.rs (run_proof_export) - Bundle v1 Implementation

**Zweck:** Exportiert Proof-Pakete im standardisierten cap-bundle.v1 Format
**Datei:** agent/src/main.rs (Zeilen 921-1555)

**BundleMeta Struktur:**
```rust
#[derive(Debug, Serialize, Deserialize)]
struct BundleMeta {
    pub schema: String,           // "cap-bundle.v1"
    pub bundle_id: String,        // "bundle-<timestamp>"
    pub created_at: String,       // RFC3339
    pub files: HashMap<String, BundleFileMeta>,
    pub proof_units: Vec<ProofUnit>,
}
```

**BundleFileMeta Struktur:**
```rust
#[derive(Debug, Serialize, Deserialize)]
struct BundleFileMeta {
    pub role: String,                   // "manifest", "proof", "timestamp", "registry"
    pub hash: String,                   // SHA3-256 (0x-präfixiert)
    pub size: usize,                    // Dateigröße in Bytes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,   // MIME-Type
    pub optional: bool,                 // Pflichtdatei?
}
```

**ProofUnit Struktur:**
```rust
#[derive(Debug, Serialize, Deserialize)]
struct ProofUnit {
    pub id: String,
    pub manifest_file: String,
    pub proof_file: String,
    pub policy_id: String,       // Automatisch aus Manifest extrahiert
    pub policy_hash: String,     // Automatisch aus Manifest extrahiert
    pub backend: String,         // "mock", "zkvm", "halo2"
}
```

**Bundle-Struktur:**
```
cap-proof/
├─ manifest.json         # Compliance manifest (role: "manifest", optional: false)
├─ proof.dat             # Zero-knowledge proof (role: "proof", optional: false)
├─ _meta.json            # Bundle metadata (schema: cap-bundle.v1)
├─ timestamp.tsr         # Optional: Timestamp (role: "timestamp", optional: true)
├─ registry.json         # Optional: Registry (role: "registry", optional: true)
├─ verification.report.json  # Verification report (role: "report", optional: false)
└─ README.txt            # Human-readable instructions
```

**Hauptfunktionen:**
- `run_proof_export()` – Orchestriert Bundle-Erstellung
  1. Lädt Manifest-Datei
  2. Extrahiert Policy-Informationen (name, hash)
  3. Erstellt Output-Verzeichnis
  4. Kopiert alle Dateien (manifest, proof, optional files)
  5. Berechnet SHA3-256 Hashes für jede Datei
  6. Erstellt BundleMeta mit files Map und proof_units Array
  7. Schreibt _meta.json
  8. Erstellt README.txt mit Verifikationsanleitung
  9. Audit-Log-Eintrag "proof_package_exported"

**Features:**
- SHA3-256 Hashes für jede Datei (Integritätsprüfung)
- Strukturierte Metadaten (Rolle, Typ, Größe, optional Flag)
- Policy-Informationen automatisch aus Manifest extrahiert
- Flexible Proof-Units (Multi-Proof-Support vorbereitet)
- Backend-Typ wird in Metadata gespeichert
- Sicherheit: Path Traversal Prevention, Cycle Detection, TOCTOU Mitigation
- Bundle Type Detection (Modern vs Legacy)

**CLI-Integration:**
```bash
cargo run -- proof export \
  --manifest build/manifest.json \
  --proof build/proof.dat \
  --out build/cap-proof \
  --force
```

**Test-Coverage:**
- `test_cli_complete_workflow` (tests/test_cli_e2e_workflow.rs) – End-to-End Test
  - Step 7: Proof Export mit --force Flag
  - Step 8: Verifier Run auf exportiertem Package
  - Verifiziert: manifest.json, proof.dat, _meta.json, README.txt existieren

**Migration von v1.0 zu cap-bundle.v1:**
- Alte Pakete (ohne _meta.json) werden noch unterstützt (Backward-Compatibility)
- Neue Pakete verwenden cap-bundle.v1 Format
- Verifier prüft zuerst auf _meta.json Existenz, Fallback auf Legacy-Format

**Analogie (Management):** Wie der Upgrade von einfachen Versandtaschen zu standardisierten Paketen mit Barcode, Tracking-Nummer und Inhaltsliste - jede Datei hat einen eindeutigen "Fingerabdruck" (SHA3-256 Hash)

---

### package_verifier.rs (Legacy Verifier)

**Zweck:** Legacy Package-Verifier (wird durch verifier::core ersetzt)
**Status:** Deprecated, nur noch für Binary-Kompatibilität
**Datei:** agent/src/package_verifier.rs

**Hinweis:** Für moderne Verifikation siehe `verifier::core` (I/O-frei, portable).

---

## 14. Lists Layer

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

## 16. Web UI Layer (v0.11.0) ✨

**Management-Zusammenfassung:** Moderne React-basierte Benutzeroberfläche für nicht-technische Nutzer. Ermöglicht Drag & Drop Upload von Proof Packages und Ein-Klick-Verifikation ohne CLI-Kenntnisse.

**Technologie-Stack:**
- React 18.x (UI Framework)
- TypeScript 5.x (Type-Safe JavaScript)
- Vite 5.x (Build Tool)
- TailwindCSS 3.x (Styling)
- Axios 1.x (HTTP Client)

**Deployment:**
- Dev Server: `npm run dev` (Port 5173)
- Production Build: `npm run build` → `dist/`
- Backend API: http://localhost:8080

**Analogie (Management):** Wie ein Bankautomat - bietet einfachen Zugang zu komplexen Backend-Funktionen

---

### webui/src/App.tsx
**Zweck:** Main Application Component & State Management
**Hauptfunktionen:**
- Orchestriert Upload & Verification Workflow
- Verwaltet globalen Zustand (apiUrl, bearerToken, manifest)
- Koordiniert Komponenten (BundleUploader, ManifestViewer, VerificationView)

**State Management:**
```typescript
const [manifest, setManifest] = useState<Manifest | null>(null);
const [verifyResult, setVerifyResult] = useState<VerifyResponse | null>(null);
const [apiUrl, setApiUrl] = useState('http://localhost:8080');
const [bearerToken, setBearerToken] = useState('admin-tom'); // Dev only!
```

**Workflow:**
```
1. User lädt Proof Package ZIP hoch
2. Backend extrahiert manifest.json + proof.dat
3. App zeigt Manifest-Daten an
4. User klickt "Proof Verifizieren"
5. Backend verifiziert gegen Policy
6. App zeigt Verification Result
```

**⚠️ Security Note:** `admin-tom` Token nur für Development! Production muss echten OAuth2 Provider nutzen.

---

### webui/src/core/api/client.ts
**Zweck:** Axios-basierter HTTP Client für REST API
**Hauptfunktionen:**
```typescript
class CapApiClient {
  private client: AxiosInstance;

  // Configuration
  setBaseURL(url: string): void
  setBearerToken(token: string): void

  // API Methods
  async uploadProofPackage(file: File): Promise<UploadResponse>
  async verifyProofBundle(request: VerifyRequest): Promise<VerifyResponse>
  async compilePolicy(request: PolicyCompileRequest): Promise<PolicyCompileResponse>
}
```

**Features:**
- Bearer Token Authentication
- Automatic JSON Serialization
- Error Handling mit Axios Interceptors
- CORS-compatible

**Request Headers:**
```
Authorization: Bearer admin-tom
Content-Type: multipart/form-data (Upload)
Content-Type: application/json (Verify)
```

---

### webui/src/core/api/types.ts
**Zweck:** TypeScript Type Definitions für API
**Hauptstrukturen:**
```typescript
interface Manifest {
  version: string;
  created_at: string;
  company_commitment_root: string;
  policy: PolicyInfo;
  audit: AuditInfo;
}

interface UploadResponse {
  manifest: Manifest;
  proof_base64: string;
  package_info: PackageInfo;
}

interface VerifyRequest {
  policy_id: string;
  context: VerifyContext;
  backend: string;          // "mock" | "zkvm" | "halo2"
  options: VerifyRequestOptions;
}

interface VerifyResponse {
  result: string;           // "ok" | "warn" | "fail"
  manifest_hash: string;
  proof_hash: string;
  report: VerifyReport;
}
```

**Type Safety:** Alle API-Requests und Responses sind typsicher durch TypeScript

---

### webui/src/hooks/useBundleUploader.ts
**Zweck:** React Hook für Bundle Upload Logic
**Hauptfunktionen:**
```typescript
function useBundleUploader() {
  const [isUploading, setIsUploading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const uploadFile = async (file: File) => {
    setIsUploading(true);
    try {
      const response = await capApiClient.uploadProofPackage(file);
      return response;
    } catch (err) {
      setError(err.message);
      throw err;
    } finally {
      setIsUploading(false);
    }
  };

  return { uploadFile, isUploading, error };
}
```

**State Management:** Verwaltet Upload-State (loading, error) für UI Feedback

---

### webui/src/components/upload/BundleUploader.tsx
**Zweck:** Drag & Drop File Upload Component
**Features:**
- Drag & Drop Zone
- File Type Validation (nur .zip)
- Progress Indicator während Upload
- Error Display

**UI Flow:**
```
1. User draggt ZIP-Datei über Drop Zone
2. Drop Zone hebt sich hervor (visuelles Feedback)
3. User dropped Datei
4. Upload startet (Spinner angezeigt)
5. Bei Erfolg: Manifest wird angezeigt
6. Bei Fehler: Error Message angezeigt
```

**Analogie (Management):** Wie E-Mail-Anhang hochladen

---

### webui/src/components/manifest/ManifestViewer.tsx
**Zweck:** Visuelle Anzeige von Manifest-Daten
**Angezeigte Felder:**
- Company Commitment Root
- Policy Name, Version, Hash
- Audit Event Count
- Created At Timestamp

**UI Design:**
- Card-basiertes Layout
- Badge für Policy Version
- Monospace Font für Hashes
- Copy-to-Clipboard Buttons

**Analogie (Management):** Wie ein "Lieferschein" - zeigt, was im Paket enthalten ist

---

### webui/src/components/verification/VerificationView.tsx
**Zweck:** Anzeige von Verification Results
**Angezeigte Informationen:**
- Status Badge (OK/WARN/FAIL) - farbcodiert
- Manifest Hash
- Proof Hash
- Signature Status
- Detailed Report (falls verfügbar)

**Status Colors:**
- OK: Grün (✅)
- WARN: Gelb (⚠️)
- FAIL: Rot (❌)

**Features:**
- Expandable Details Accordion
- Copy-to-Clipboard für Hashes
- Export Button für Report (zukünftig)

**Analogie (Management):** Wie ein TÜV-Zertifikat - zeigt, ob Nachweis bestanden hat

---

## 17. Monitoring & Observability Layer (Week 2) 📊

**Management-Zusammenfassung:** Production-Ready Monitoring Stack nach Google SRE Prinzipien. Bietet 360°-Sicht auf System-Gesundheit mit Metriken, Logs und Traces - vollständig korreliert für schnelle Problemanalyse.

**Die drei Säulen der Observability:**
1. **Metrics (Prometheus)** - Was passiert? (Request Rate, Error Rate, Latency)
2. **Logs (Loki)** - Warum passiert es? (Fehler-Details, Events)
3. **Traces (Jaeger)** - Wo passiert es? (Request-Flow durch System)

**Deployment:** 8 Container via Docker Compose, 5/5 Health Checks passing
**Status:** ✅ Production-Ready (erfolgreich deployed und getestet)

**Analogie (Management):** Wie ein modernes Flugzeug-Cockpit - alle wichtigen Instrumente auf einen Blick, Warnlampen bei Problemen, Black-Box-Recorder für Incident-Analyse

---

### monitoring/prometheus/prometheus.yml
**Zweck:** Metrics Collection Configuration
**Hauptfunktionen:**
- Scrapes Metrics von CAP Verifier API (`/metrics` endpoint)
- Scrapes Node Exporter (Host Metrics: CPU, Memory, Disk)
- Scrapes cAdvisor (Container Metrics)
- Evaluiert Alert Rules alle 15s

**Scrape Targets:**
```yaml
scrape_configs:
  - job_name: 'cap-verifier-api'
    scrape_interval: 10s
    static_configs:
      - targets: ['cap-verifier-api:8080']

  - job_name: 'node-exporter'
    static_configs:
      - targets: ['node-exporter:9100']

  - job_name: 'cadvisor'
    static_configs:
      - targets: ['cadvisor:8080']
```

**Configuration:**
- Retention: 30 Tage
- Scrape Interval: 15s (global), 10s (API)
- Evaluation Interval: 15s (für Alert Rules)

**Service URL:** http://localhost:9090 (Prometheus UI)

**Analogie (Management):** Wie ein Datenlogger - sammelt alle Messwerte im 15-Sekunden-Takt

---

### monitoring/prometheus/alerts/cap-verifier-rules.yml
**Zweck:** Alert Rule Definitions
**Alert-Kategorien:**

**Critical Alerts (3):**
- `CAPVerifierAPIDown` - API antwortet nicht (>1 Minute)
- `CAPVerifierHighErrorRate` - Error Rate > 5% (kritisch)
- `CAPVerifierAuthFailureSpike` - Auth Failures > 10 in 5min

**Warning Alerts (4):**
- `CAPVerifierElevatedErrorRate` - Error Rate > 1%
- `CAPVerifierLowCacheHit` - Cache Hit Ratio < 50%
- `CAPVerifierAuthFailuresIncreasing` - Auth Failures > 5 in 5min
- `CAPVerifierNoTraffic` - Keine Requests seit 10 Minuten

**Info Alerts (2):**
- `CAPVerifierHighRequestRate` - Request Rate > 100 RPS (Capacity Planning)
- `CAPVerifierCacheDegradation` - Cache Hit Ratio < 70%

**SLO-Based Alerts (1):**
- `CAPVerifierErrorBudgetBurning` - Error Budget wird zu schnell verbraucht (99.9% SLO violation)

**Prometheus Queries:**
```promql
# Error Rate
rate(cap_verifier_requests_total{result="fail"}[5m]) > 0.05

# Auth Failures
increase(cap_auth_token_validation_failures_total[5m]) > 10

# Cache Hit Ratio
cap_cache_hit_ratio < 0.5
```

**Analogie (Management):** Wie Rauchmelder im Haus - verschiedene Sensoren für verschiedene Gefahren

---

### monitoring/grafana/dashboards/cap-verifier-api.json
**Zweck:** Main Production Dashboard (UID: `cap-verifier-api`)
**Panels:** 13 Panels in 4 Kategorien

**Overview (4 Panels):**
- Total Requests (1h) - Stat Panel mit Gesamtzahl
- Request Rate - Stat Panel mit Sparkline (Trend)
- Error Rate - Stat Panel mit Thresholds (>1% Yellow, >5% Red)
- Cache Hit Ratio - Stat Panel mit Gauge (0-100%)

**Request Metrics (2 Panels):**
- Request Rate by Result - Timeseries mit Stacking (ok/warn/fail)
- Request Distribution - Pie Chart (ok vs. fail Prozentual)

**Authentication & Security (2 Panels):**
- Auth Failures Timeline - Timeseries (Spikes erkennen)
- Total Auth Failures - Counter (Gesamtzahl)

**Cache Performance (2 Panels):**
- Cache Hit Ratio (Timeline) - Timeseries mit 70%-Threshold-Linie
- Cache Misses - Counter

**System Health (3 Panels):**
- CPU Usage - Timeseries (Node Exporter Metric)
- Memory Usage - Timeseries mit Used/Available
- Uptime - Stat Panel (Sekunden seit Start)

**Template Variables:**
- `$namespace` - Namespace Filter für Multi-Tenancy (zukünftig)

**Auto-Refresh:** 30 Sekunden
**Service URL:** http://localhost:3000 (admin/admin)

**Analogie (Management):** Wie ein KPI-Dashboard für Geschäftsführer - alle wichtigen Kennzahlen auf einen Blick

---

### monitoring/grafana/dashboards/slo-monitoring.json
**Zweck:** SLO/SLI Monitoring Dashboard (UID: `slo-monitoring`)
**Panels:** 17 Panels in 4 Kategorien

**SLO Compliance Overview (4 Panels):**
- Availability SLO (99.9%) - Stat Panel (Current: 99.95%)
- Error Rate SLO (< 0.1%) - Stat Panel (Current: 0.05%)
- Auth Success SLO (99.95%) - Stat Panel (Current: 99.98%)
- Cache Hit Rate SLO (> 70%) - Stat Panel (Current: 75%)

**Error Budget Status (3 Panels):**
- Availability Error Budget Remaining - Gauge (0-100%, Red bei <25%)
- Error Rate Budget Remaining - Gauge
- Auth Success Budget Remaining - Gauge

**Error Budget Burn Rate (4 Panels):**
- Availability Burn Rate (1h) - Timeseries (sollte <14.4x sein)
- Availability Burn Rate (6h) - Timeseries (sollte <6.0x sein)
- Error Rate Burn Rate (1h) - Timeseries
- Error Rate Burn Rate (6h) - Timeseries

**SLI Trends (6 Panels):**
- Availability Trend (30d) - Timeseries (99-100% Range) mit 99.9%-Linie
- Error Rate Trend (30d) - Timeseries mit 0.1%-Linie
- Auth Success Rate Trend - Timeseries
- Cache Hit Rate Trend - Timeseries
- 30-Day Error Budget Consumption - Timeseries (kumulativ)
- SLO Violations Timeline - Bar Chart (zeigt Zeitpunkte von Violations)

**Formulae:**
```
Error Budget Remaining = 1 - ((1 - Current SLI) / (1 - SLO Target))
Burn Rate = (Error Rate / Error Budget) * Time Window Multiplier
```

**Beispiel:**
- SLO: 99.9% Availability → Error Budget: 0.1% = 43.2 min/Monat
- Current Availability: 99.95% → Verbraucht: 50% Error Budget
- Remaining: 50% = ~21.6 min noch verfügbar

**Analogie (Management):** Wie ein Jahres-Budget-Dashboard - zeigt, wie viel "Fehlerbudget" noch übrig ist

---

### monitoring/loki/loki-config.yml
**Zweck:** Log Aggregation Configuration
**Hauptfunktionen:**
- Sammelt Logs von Promtail (Docker + Kubernetes Service Discovery)
- Speichert Logs in Filesystem (boltdb-shipper)
- Ermöglicht LogQL-Queries (wie SQL für Logs)

**Configuration:**
```yaml
schema_config:
  configs:
    - from: 2025-11-01
      store: boltdb-shipper
      object_store: filesystem
      schema: v11

storage_config:
  boltdb_shipper:
    active_index_directory: /loki/boltdb-shipper-active
    cache_location: /loki/boltdb-shipper-cache
  filesystem:
    directory: /loki/chunks

limits_config:
  retention_period: 744h  # 31 Tage
  max_query_length: 721h  # 30 Tage
  ingestion_rate_mb: 10
```

**Compactor:**
- Retention Deletion: Enabled
- Compaction Interval: 10 Minuten
- Löscht Logs älter als 31 Tage automatisch

**Query Features:**
- Query Results Cache: 100 MB embedded cache
- Unordered Writes: Unterstützt (für High-Throughput)

**Service URL:** http://localhost:3100 (Loki API)

**Analogie (Management):** Wie ein Archiv - alle Logbuch-Einträge durchsuchbar, ältere werden automatisch vernichtet

---

### monitoring/promtail/promtail-config.yml
**Zweck:** Log Collection Agent Configuration
**Hauptfunktionen:**
- Scraped Logs von Docker Containern
- Scraped Logs von Kubernetes Pods
- Parsed JSON Logs automatisch
- Extrahiert Trace IDs für Korrelation

**Scrape Job 1: cap-verifier-api (Docker)**
```yaml
- job_name: cap-verifier-api
  docker_sd_configs:
    - host: unix:///var/run/docker.sock
  relabel_configs:
    - source_labels: ['__meta_docker_container_label_app']
      regex: 'cap-verifier-api'
      action: keep
  pipeline_stages:
    - json:
        expressions:
          timestamp: timestamp
          level: level
          message: message
    - labels:
        level:
    - timestamp:
        source: timestamp
        format: RFC3339Nano
```

**Scrape Job 2: kubernetes-pods**
```yaml
- job_name: kubernetes-pods
  kubernetes_sd_configs:
    - role: pod
  relabel_configs:
    - source_labels: [__meta_kubernetes_pod_label_app]
      regex: 'cap-verifier-api'
      action: keep
  pipeline_stages:
    - cri: {}
    - json:
        expressions:
          trace_id: trace_id
          span_id: span_id
    - metrics:
        log_lines_total:
          type: Counter
          description: "Total log lines"
          source: level
          config:
            action: inc
```

**Features:**
- Automatisches JSON Parsing
- Label Extraction (level, target, trace_id)
- Metrics Extraction (`log_lines_total`, `auth_failures_total`)
- Timestamp Parsing (RFC3339Nano)

**Service URL:** http://localhost:9080 (Promtail API)

**Analogie (Management):** Wie ein Postbote - sammelt alle Logbuch-Einträge von verschiedenen Orten und bringt sie ins Archiv

---

### monitoring/jaeger/jaeger-config.yml
**Zweck:** Distributed Tracing Configuration
**Hauptfunktionen:**
- Sammelt Traces von CAP Verifier API (OTLP Protocol)
- Visualisiert Request Flow durch System
- Korreliert Traces mit Logs & Metrics

**Configuration:**
```yaml
sampling:
  strategies:
    - type: probabilistic
      param: 1.0  # 100% sampling (für Dev/Testing)

storage:
  type: memory
  memory:
    max-traces: 10000
```

**Ports:**
- 16686 - Jaeger UI
- 14268 - jaeger.thrift (HTTP)
- 4317 - OTLP gRPC
- 4318 - OTLP HTTP
- 14269 - Health Check

**Grafana Integration:**
```yaml
jsonData:
  tracesToLogs:
    datasourceUid: loki
    tags: ['trace_id']
    mappedTags:
      - key: service.name
        value: app
    filterByTraceID: true
    filterBySpanID: false
    spanStartTimeShift: '-1m'
    spanEndTimeShift: '1m'

  tracesToMetrics:
    datasourceUid: prometheus
    queries:
      - name: 'Request Rate'
        query: 'rate(cap_verifier_requests_total{app="$__tags"}[5m])'
      - name: 'Error Rate'
        query: 'rate(cap_verifier_requests_total{app="$__tags",result="fail"}[5m])'

  nodeGraph:
    enabled: true
```

**Features:**
- **Traces → Logs:** Klick auf Trace-ID öffnet Loki mit gefilterten Logs
- **Traces → Metrics:** Zeigt Request/Error Rate für getraced Service
- **Node Graph:** Visualisiert Service Dependencies

**Service URL:** http://localhost:16686 (Jaeger UI)

**Analogie (Management):** Wie ein GPS-Tracker - zeigt exakte Route eines Requests durch das System

---

### monitoring/slo/slo-config.yml
**Zweck:** SLO/SLI Definitions (Google SRE Workbook)
**Defined SLOs:**

| SLO Name | Target | Time Window | Error Budget | Burn Rate Alerts |
|----------|--------|-------------|--------------|------------------|
| availability_999 | 99.9% | 30 days | 43.2 min/month | Fast: 14.4x, Slow: 6.0x |
| error_rate_001 | < 0.1% | 30 days | 0.1% | Fast: 14.4x, Slow: 6.0x |
| auth_success_9995 | 99.95% | 30 days | 0.05% | Fast: 14.4x, Slow: 6.0x |
| cache_hit_rate_70 | > 70% | 7 days | 30% | Threshold: < 60% |

**SLI Formulas:**
```yaml
availability_sli:
  formula: "ok_requests / total_requests"
  prometheus_query: |
    sum(rate(cap_verifier_requests_total{result="ok"}[5m])) /
    sum(rate(cap_verifier_requests_total[5m]))

error_rate_sli:
  formula: "fail_requests / total_requests"
  prometheus_query: |
    sum(rate(cap_verifier_requests_total{result="fail"}[5m])) /
    sum(rate(cap_verifier_requests_total[5m]))

auth_success_sli:
  formula: "(total_requests - auth_failures) / total_requests"
  prometheus_query: |
    (sum(rate(cap_verifier_requests_total[5m])) -
     rate(cap_auth_token_validation_failures_total[5m])) /
    sum(rate(cap_verifier_requests_total[5m]))
```

**Error Budget Policies:**

**Policy 1: Slow Rollout (< 25% Error Budget remaining)**
- Pause automated deployments
- Require manual approval für alle Changes
- Increase monitoring cadence (15min → 5min checks)
- Root Cause Analysis required

**Policy 2: Emergency Freeze (< 5% Error Budget remaining)**
- **FREEZE** all deployments (außer Hotfixes)
- Activate incident response team
- Daily Status Meetings mit Management
- Post-Incident Review mandatory

**Analogie (Management):** Wie ein Jahres-Wartungsbudget - wenn 95% verbraucht sind, werden nur noch Notfall-Reparaturen durchgeführt

---

### monitoring/test-monitoring.sh
**Zweck:** Automated Health Check Script
**Hauptfunktionen:**
- Prüft Health von allen 8 Services
- Zeigt Container Status
- Gibt Service URLs aus
- Bietet Test-Request-Beispiele

**Checks:**
```bash
# CAP Verifier API
curl -s http://localhost:8080/healthz | jq

# Prometheus
curl -s http://localhost:9090/-/healthy

# Grafana
curl -s http://localhost:3000/api/health | jq

# Loki
curl -s http://localhost:3100/ready

# Jaeger
curl -s http://localhost:14269/ # Health Check Port
```

**Output:**
```
✅ CAP Verifier API: {"status":"OK","version":"0.1.0"}
✅ Prometheus: Prometheus is Healthy
✅ Grafana: {"commit":"...","database":"ok","version":"..."}
✅ Loki: ready
✅ Jaeger: {"status":"Server available"}

📊 Container Status:
NAME                    STATUS                  HEALTH
cap-verifier-api        Up 2 hours              healthy
prometheus              Up 2 hours              healthy
grafana                 Up 2 hours              healthy
loki                    Up 2 hours              healthy
promtail                Up 2 hours              -
jaeger                  Up 2 hours              healthy
node-exporter           Up 2 hours              -
cadvisor                Up 2 hours              -
```

**Verwendung:**
```bash
cd monitoring
chmod +x test-monitoring.sh
./test-monitoring.sh
```

**Analogie (Management):** Wie ein Wartungstechniker - geht alle Systeme durch und prüft, ob sie laufen

---

## Zusammenfassung

Der LsKG-Agent besteht aus:
- **17 Kategorien** mit klaren Verantwortlichkeiten
- **80+ Module** für verschiedene Funktionen (Backend + Frontend + Monitoring)
- **Trait-basierte Abstraktion** für Erweiterbarkeit
- **Type-Safe** durch Rust's starkes Typsystem (Backend) und TypeScript (Frontend)
- **Testbar** durch klare Schnittstellen
- **Production-Ready** mit vollständigem Monitoring Stack (Metrics, Logs, Traces)
- **User-Friendly** durch React-basierte Web UI für nicht-technische Nutzer

**Neue Komponenten in v0.11.0:**
- ✨ **Web UI** (7 Komponenten) - React + TypeScript für grafische Oberfläche
- 📊 **Monitoring Stack** (8 Services) - Prometheus, Grafana, Loki, Jaeger, Node Exporter, cAdvisor
- 🗄️ **Policy Store** (4 Module) - InMemory + SQLite Backends mit Content Deduplication
- 🚦 **Rate Limiting** - IP-basierte Request Throttling
- 📤 **File Upload** - Multipart Upload Handler für Proof Packages

**Production-Ready Features:**
- SLO/SLI Monitoring mit Error Budget Tracking
- 11 Alert Rules in 3 Severities (Critical, Warning, Info)
- 2 Grafana Dashboards mit 30 Panels
- Full Observability (Logs ↔ Traces ↔ Metrics Correlation)
- 8/8 Docker Containers running, 5/5 Health Checks passing
