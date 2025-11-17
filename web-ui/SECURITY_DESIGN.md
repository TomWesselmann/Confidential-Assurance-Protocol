# CAP Web UI - Security Design Document

**Version:** 1.0
**Date:** 2025-11-17
**Classification:** Internal - Security Architecture
**Compliance:** OWASP Top 10, NIST CSF, ISO 27001, BSI IT-Grundschutz, DSGVO

---

## Executive Summary

This document defines the security architecture for the CAP (Confidential Assurance Protocol) Web UI, a business-critical application for supply chain compliance verification in accordance with the German LkSG (Lieferkettensorgfaltspflichtengesetz).

**Security Posture:** Defense in Depth with Zero Trust Architecture

**Target Audience:** Enterprise customers (DAX 40, Fortune 500)

**Key Security Objectives:**
1. **Confidentiality**: No PII/sensitive data exposure
2. **Integrity**: Tamper-proof audit trail
3. **Availability**: 99.9% uptime SLO
4. **Compliance**: DSGVO, LkSG, ISO 27001 ready

---

## Table of Contents

1. [Threat Model](#1-threat-model)
2. [Security Architecture](#2-security-architecture)
3. [Authentication & Authorization](#3-authentication--authorization)
4. [Data Protection](#4-data-protection)
5. [Network Security](#5-network-security)
6. [Application Security](#6-application-security)
7. [Cryptography](#7-cryptography)
8. [Audit & Logging](#8-audit--logging)
9. [Dependency Management](#9-dependency-management)
10. [Deployment Security](#10-deployment-security)
11. [Incident Response](#11-incident-response)
12. [Compliance Matrix](#12-compliance-matrix)

---

## 1. Threat Model

### 1.1 Assets

| Asset | Classification | Impact if Compromised |
|-------|---------------|----------------------|
| JWT Authentication Tokens | Critical | Full account takeover |
| Verification Proofs | High | Compliance fraud |
| Policy Files (YAML) | High | Policy manipulation |
| Audit Logs | High | Non-repudiation loss |
| User Sessions | Medium | Session hijacking |
| API Credentials | Critical | Backend compromise |

### 1.2 Threat Actors

1. **External Attackers**
   - Skill Level: Advanced Persistent Threat (APT)
   - Motivation: Corporate espionage, competitive advantage
   - Tactics: XSS, CSRF, SQL Injection, Man-in-the-Middle

2. **Malicious Insiders**
   - Skill Level: Expert (knows system internals)
   - Motivation: Data exfiltration, sabotage
   - Tactics: Privilege escalation, backdoor insertion

3. **Supply Chain Attacks**
   - Skill Level: Nation-state actors
   - Motivation: Widespread compromise
   - Tactics: Dependency poisoning, build process manipulation

### 1.3 Attack Vectors (STRIDE Analysis)

| Threat | Attack Vector | Mitigation |
|--------|--------------|------------|
| **S**poofing | Fake JWT tokens | RS256 signature validation, short lifetime (15 min) |
| **T**ampering | Proof file manipulation | BLAKE3 hash verification, immutable audit log |
| **R**epudiation | Deny verification action | SHA3-256 hash chain, digital signatures |
| **I**nformation Disclosure | XSS data exfiltration | CSP, input sanitization, HttpOnly cookies |
| **D**enial of Service | API flooding | Rate limiting (10 req/min), CDN (Cloudflare) |
| **E**levation of Privilege | Session hijacking | SameSite cookies, CSRF tokens, short sessions |

### 1.4 Risk Assessment (CVSS v3.1)

| Vulnerability | CVSS Score | Severity | Mitigation Priority |
|--------------|-----------|----------|---------------------|
| XSS (Stored) | 8.8 | High | P0 (Critical) |
| CSRF | 6.5 | Medium | P1 (High) |
| Weak JWT | 9.1 | Critical | P0 (Critical) |
| Dependency CVE | 7.3 | High | P1 (High) |
| Info Disclosure | 5.3 | Medium | P2 (Medium) |

---

## 2. Security Architecture

### 2.1 Zero Trust Principles

```
┌─────────────────────────────────────────────────────────────────┐
│                    Zero Trust Architecture                       │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Browser (Untrusted Zone)                                       │
│  ├─ React App (Sandboxed)                                       │
│  ├─ JWT Token (Memory Only, 15 min TTL)                         │
│  └─ CSP: default-src 'self'                                     │
│                                                                  │
│         ↓ HTTPS (TLS 1.3) ↓                                     │
│                                                                  │
│  Reverse Proxy (DMZ)                                            │
│  ├─ Nginx + ModSecurity WAF                                     │
│  ├─ Rate Limiting (10 req/min per IP)                           │
│  ├─ HSTS, X-Frame-Options, CSP Headers                          │
│  └─ TLS Termination + Certificate Pinning                       │
│                                                                  │
│         ↓ mTLS (optional) ↓                                     │
│                                                                  │
│  Backend API (Trusted Zone)                                     │
│  ├─ Axum (Rust) + OAuth2 Middleware                            │
│  ├─ JWT Validation (RS256, aud/iss/exp checks)                 │
│  ├─ RBAC Enforcement (Scopes: verify.read, policy.write)       │
│  └─ Audit Log (SHA3-256 Hash Chain)                            │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### 2.2 Defense in Depth Layers

| Layer | Technology | Purpose |
|-------|-----------|---------|
| 1. Network | Cloudflare CDN + DDoS Protection | Layer 3/4 attacks |
| 2. Transport | TLS 1.3 + HSTS | MITM prevention |
| 3. Application | CSP + CORS | XSS/CSRF prevention |
| 4. Authentication | OAuth2 + JWT (RS256) | Identity verification |
| 5. Authorization | RBAC (Scopes) | Access control |
| 6. Data | BLAKE3 + SHA3-256 | Integrity |
| 7. Audit | Immutable Hash Chain | Non-repudiation |

---

## 3. Authentication & Authorization

### 3.1 OAuth2 Client Credentials Flow (RFC 6749)

**Use Case:** Machine-to-Machine (M2M) or Backend-for-Frontend (BFF)

```typescript
// Token Endpoint (Backend)
POST https://auth.cap-system.com/oauth/token
Content-Type: application/x-www-form-urlencoded

grant_type=client_credentials
&client_id=cap-web-ui-prod
&client_secret=<SECRET>
&scope=verify.read policy.write audit.read
```

**Response:**
```json
{
  "access_token": "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...",
  "token_type": "Bearer",
  "expires_in": 900,  // 15 minutes
  "scope": "verify.read policy.write"
}
```

### 3.2 JWT Structure (RS256)

**Header:**
```json
{
  "alg": "RS256",
  "typ": "JWT",
  "kid": "cap-prod-2025-11"
}
```

**Payload:**
```json
{
  "iss": "https://auth.cap-system.com",
  "sub": "cap-web-ui-prod",
  "aud": "https://api.cap-system.com",
  "exp": 1732206000,  // 15 min from iat
  "iat": 1732205100,
  "nbf": 1732205100,
  "jti": "550e8400-e29b-41d4-a716-446655440000",
  "scope": "verify.read policy.write"
}
```

**Validation (Backend):**
```rust
// Mandatory checks (MUST)
1. Signature verification (RS256 with public key)
2. Expiration (exp > now)
3. Not before (nbf <= now)
4. Issuer (iss == "https://auth.cap-system.com")
5. Audience (aud == "https://api.cap-system.com")

// Recommended checks (SHOULD)
6. JTI uniqueness (prevent replay attacks)
7. Scope validation (required scope present)
```

### 3.3 Token Storage (Frontend)

**❌ NEVER:**
- `localStorage` (XSS vulnerable)
- `sessionStorage` (XSS vulnerable)
- Cookies without HttpOnly (XSS vulnerable)

**✅ BEST PRACTICE:**
```typescript
// Option 1: Memory Only (Recommended for SPA)
const useAuthStore = create<AuthState>((set) => ({
  token: null,  // Cleared on page refresh
  setToken: (token) => set({ token }),
  clearToken: () => set({ token: null })
}));

// Option 2: HttpOnly Cookie (BFF pattern)
Set-Cookie: access_token=<JWT>; HttpOnly; Secure; SameSite=Strict; Max-Age=900
```

### 3.4 Token Refresh Strategy

```typescript
// Auto-refresh 2 minutes before expiry
const TOKEN_REFRESH_BUFFER = 120; // seconds

useEffect(() => {
  const interval = setInterval(() => {
    const expiresAt = parseJwt(token).exp;
    const now = Date.now() / 1000;

    if (expiresAt - now < TOKEN_REFRESH_BUFFER) {
      refreshToken();
    }
  }, 60000); // Check every minute

  return () => clearInterval(interval);
}, [token]);
```

### 3.5 Role-Based Access Control (RBAC)

| Role | Scopes | Permissions |
|------|--------|-------------|
| **Viewer** | `verify.read`, `audit.read` | View verifications, view audit logs |
| **Operator** | `verify.read`, `verify.write`, `audit.read` | Create verifications |
| **Admin** | `policy.read`, `policy.write`, `verify.*`, `audit.*` | Manage policies |
| **Auditor** | `audit.read`, `verify.read` | Read-only access to all |

**Enforcement (Frontend):**
```typescript
const hasScope = (requiredScope: string): boolean => {
  const scopes = parseJwt(token).scope.split(' ');
  return scopes.includes(requiredScope);
};

// Component-level protection
{hasScope('policy.write') && <UploadPolicyButton />}
```

**Enforcement (Backend):**
```rust
#[axum::middleware]
async fn require_scope(req: Request, next: Next, scope: &str) -> Response {
    let claims = extract_jwt(&req)?;

    if !claims.scope.contains(scope) {
        return StatusCode::FORBIDDEN.into_response();
    }

    next.run(req).await
}
```

---

## 4. Data Protection

### 4.1 Data Classification

| Data Type | Classification | Encryption | Retention |
|-----------|---------------|-----------|-----------|
| JWT Token | Secret | In transit (TLS 1.3) | 15 min (memory) |
| Verification Proof | Confidential | BLAKE3 hash | 7 years (audit) |
| Policy Files (YAML) | Internal | TLS 1.3 | Indefinite |
| Audit Logs | Confidential | SHA3-256 hash chain | 10 years |
| User Input | Untrusted | Sanitized + validated | N/A |

### 4.2 PII Handling (DSGVO/GDPR)

**Principles:**
1. **Data Minimization**: Collect only necessary data
2. **Purpose Limitation**: Use data only for stated purpose
3. **Storage Limitation**: Delete after retention period
4. **Integrity & Confidentiality**: Encrypt in transit and at rest

**PII in CAP System:**
- ❌ **NO PII stored in frontend** (JWT contains only client_id, not user names)
- ✅ **Hashed Identifiers**: Supplier IDs hashed with BLAKE3
- ✅ **Right to Erasure**: Audit logs can be anonymized (replace user IDs with pseudonyms)
- ✅ **Data Portability**: Export function for audit logs (JSON format)

### 4.3 Input Validation & Sanitization

**Client-Side Validation (Zod):**
```typescript
import { z } from 'zod';

const VerifyRequestSchema = z.object({
  policy_id: z.string().regex(/^[a-z0-9\.\-]+$/),  // lksg.v1
  proof_file: z.instanceof(File).refine(
    (file) => file.size <= 10 * 1024 * 1024,  // Max 10 MB
    { message: "File too large" }
  ),
  context: z.object({
    supplier_hashes: z.array(z.string().regex(/^0x[a-f0-9]{64}$/)),  // BLAKE3
    supplier_regions: z.array(z.string().length(2))  // ISO 3166-1 alpha-2
  })
});

// Usage
const result = VerifyRequestSchema.safeParse(userInput);
if (!result.success) {
  throw new ValidationError(result.error.issues);
}
```

**Output Escaping (React default):**
```typescript
// ✅ SAFE (React escapes by default)
<div>{userInput}</div>

// ❌ DANGEROUS (XSS vulnerable)
<div dangerouslySetInnerHTML={{ __html: userInput }} />

// ✅ SAFE (DOMPurify for rich text)
import DOMPurify from 'dompurify';
<div dangerouslySetInnerHTML={{
  __html: DOMPurify.sanitize(userInput, {
    ALLOWED_TAGS: ['b', 'i', 'em', 'strong']
  })
}} />
```

### 4.4 File Upload Security

**Proof File Upload:**
```typescript
const ALLOWED_MIME_TYPES = [
  'application/json',
  'application/octet-stream'  // Binary proof files
];

const MAX_FILE_SIZE = 10 * 1024 * 1024; // 10 MB

const validateFile = (file: File): void => {
  // 1. Check MIME type
  if (!ALLOWED_MIME_TYPES.includes(file.type)) {
    throw new Error('Invalid file type');
  }

  // 2. Check file size
  if (file.size > MAX_FILE_SIZE) {
    throw new Error('File too large');
  }

  // 3. Check magic bytes (backend validation)
  // Backend MUST validate file signature, not just extension
};
```

**Backend File Validation (Rust):**
```rust
// Verify JSON structure, not just extension
fn validate_proof_file(bytes: &[u8]) -> Result<Proof> {
    // 1. Check magic bytes
    if bytes.len() < 4 || &bytes[0..4] != b"{\"pr" {
        return Err(Error::InvalidFileFormat);
    }

    // 2. Parse JSON (fails if malformed)
    let proof: Proof = serde_json::from_slice(bytes)?;

    // 3. Validate structure
    if proof.version != "v1" {
        return Err(Error::UnsupportedVersion);
    }

    Ok(proof)
}
```

---

## 5. Network Security

### 5.1 Transport Layer Security (TLS 1.3)

**Nginx Configuration:**
```nginx
server {
    listen 443 ssl http2;
    server_name cap-ui.example.com;

    # TLS 1.3 only
    ssl_protocols TLSv1.3;
    ssl_prefer_server_ciphers on;
    ssl_ciphers 'TLS_AES_256_GCM_SHA384:TLS_CHACHA20_POLY1305_SHA256';

    # Certificates
    ssl_certificate /etc/ssl/certs/cap-ui.crt;
    ssl_certificate_key /etc/ssl/private/cap-ui.key;

    # OCSP Stapling
    ssl_stapling on;
    ssl_stapling_verify on;
    ssl_trusted_certificate /etc/ssl/certs/ca-chain.crt;

    # Session cache
    ssl_session_cache shared:SSL:10m;
    ssl_session_timeout 10m;

    # HSTS (2 years)
    add_header Strict-Transport-Security "max-age=63072000; includeSubDomains; preload" always;
}
```

### 5.2 Content Security Policy (CSP)

**Level 3 CSP (Strictest):**
```http
Content-Security-Policy:
  default-src 'none';
  script-src 'self' 'nonce-{RANDOM}';
  style-src 'self' 'nonce-{RANDOM}';
  img-src 'self' data: https:;
  font-src 'self';
  connect-src 'self' https://api.cap-system.com;
  form-action 'self';
  frame-ancestors 'none';
  base-uri 'self';
  upgrade-insecure-requests;
  block-all-mixed-content;
  report-uri https://cap-system.report-uri.com/r/d/csp/enforce;
```

**React Integration (Nonce):**
```typescript
// vite.config.ts
import { defineConfig } from 'vite';
import cspPlugin from 'vite-plugin-csp';

export default defineConfig({
  plugins: [
    cspPlugin({
      nonce: true,  // Auto-generate nonce for inline scripts
      policy: {
        'script-src': ["'self'", "'nonce-{NONCE}'"],
        'style-src': ["'self'", "'nonce-{NONCE}'"]
      }
    })
  ]
});
```

### 5.3 Cross-Origin Resource Sharing (CORS)

**Backend (Axum):**
```rust
use tower_http::cors::{CorsLayer, AllowOrigin};

let cors = CorsLayer::new()
    .allow_origin(AllowOrigin::exact("https://cap-ui.example.com".parse()?))
    .allow_methods([Method::GET, Method::POST])
    .allow_headers([AUTHORIZATION, CONTENT_TYPE])
    .allow_credentials(true)  // For cookies
    .max_age(Duration::from_secs(3600));

let app = Router::new()
    .layer(cors);
```

### 5.4 Rate Limiting

**Nginx (ngx_http_limit_req_module):**
```nginx
# Define rate limit zones
limit_req_zone $binary_remote_addr zone=api:10m rate=10r/m;  # 10 requests/minute
limit_req_zone $binary_remote_addr zone=auth:10m rate=5r/m;  # 5 requests/minute (stricter)

server {
    location /api/ {
        limit_req zone=api burst=20 nodelay;
        limit_req_status 429;

        proxy_pass http://backend;
    }

    location /oauth/token {
        limit_req zone=auth burst=5 nodelay;
        limit_req_status 429;

        proxy_pass http://auth-server;
    }
}
```

**Error Response:**
```json
HTTP/1.1 429 Too Many Requests
Retry-After: 60
X-RateLimit-Limit: 10
X-RateLimit-Remaining: 0
X-RateLimit-Reset: 1732206060

{
  "error": "rate_limit_exceeded",
  "message": "Too many requests. Please try again in 60 seconds."
}
```

---

## 6. Application Security

### 6.1 OWASP Top 10 (2021) Mitigation

| OWASP Risk | Mitigation | Implementation |
|-----------|-----------|----------------|
| A01: Broken Access Control | RBAC + JWT scopes | `hasScope('verify.write')` |
| A02: Cryptographic Failures | TLS 1.3, BLAKE3, SHA3-256 | All data in transit encrypted |
| A03: Injection | Zod validation, prepared statements | `VerifyRequestSchema.parse()` |
| A04: Insecure Design | Zero Trust, Defense in Depth | Layered security architecture |
| A05: Security Misconfiguration | Security headers, CSP | Nginx + Helmet.js |
| A06: Vulnerable Components | npm audit, Dependabot | CI/CD automated scans |
| A07: Authentication Failures | OAuth2, short JWT lifetime | 15 min expiry, auto-refresh |
| A08: Software/Data Integrity | SBOM, SRI, Audit logs | SHA3-256 hash chain |
| A09: Logging Failures | Immutable audit trail | Loki + Grafana |
| A10: SSRF | Allowlist, DNS validation | Only connect to known APIs |

### 6.2 XSS Prevention

**1. Context-Aware Output Encoding:**
```typescript
// ✅ HTML Context
<div>{userInput}</div>  // React auto-escapes

// ✅ JavaScript Context
<script>
  const data = {JSON.stringify(userInput)};  // JSON encoding
</script>

// ✅ URL Context
<a href={`/verify?id=${encodeURIComponent(verifyId)}`}>View</a>

// ✅ CSS Context (avoid if possible)
<div style={{ color: sanitizeColor(userInput) }}></div>
```

**2. Trusted Types (CSP Level 3):**
```typescript
// vite-env.d.ts
interface Window {
  trustedTypes?: {
    createPolicy: (name: string, policy: any) => any;
  };
}

// Create policy
const escapePolicy = window.trustedTypes?.createPolicy('default', {
  createHTML: (input: string) => DOMPurify.sanitize(input)
});

// Usage
element.innerHTML = escapePolicy.createHTML(userInput);
```

### 6.3 CSRF Prevention

**Double Submit Cookie Pattern:**
```typescript
// Frontend: Send CSRF token in header
const csrfToken = Cookies.get('XSRF-TOKEN');

axios.post('/api/verify', data, {
  headers: {
    'X-XSRF-TOKEN': csrfToken
  }
});

// Backend: Validate token
#[axum::middleware]
async fn csrf_protection(req: Request, next: Next) -> Response {
    if !["GET", "HEAD", "OPTIONS"].contains(&req.method().as_str()) {
        let cookie_token = extract_csrf_cookie(&req);
        let header_token = req.headers().get("X-XSRF-TOKEN");

        if cookie_token != header_token {
            return StatusCode::FORBIDDEN.into_response();
        }
    }

    next.run(req).await
}
```

**Cookie Configuration:**
```http
Set-Cookie: XSRF-TOKEN=<RANDOM>; SameSite=Strict; Secure; Path=/
```

### 6.4 Clickjacking Prevention

**X-Frame-Options:**
```nginx
add_header X-Frame-Options "DENY" always;
```

**CSP frame-ancestors:**
```http
Content-Security-Policy: frame-ancestors 'none';
```

---

## 7. Cryptography

### 7.1 Hash Functions

| Use Case | Algorithm | Rationale |
|---------|-----------|-----------|
| Supplier ID Hashing | BLAKE3 | Fast (Merkle tree), collision-resistant |
| Audit Log Hash Chain | SHA3-256 | NIST approved, quantum-resistant candidate |
| Password Hashing (if needed) | Argon2id | Winner of PHC, memory-hard |
| HMAC (API signing) | HMAC-SHA256 | RFC 2104 |

**BLAKE3 Usage (Supplier IDs):**
```typescript
import { blake3 } from 'blake3-wasm';

const hashSupplier = (id: string, name: string): string => {
  const input = `${id}:${name}`;
  const hash = blake3(input);
  return `0x${hash}`;  // 64 hex chars
};
```

**SHA3-256 Usage (Audit Log):**
```rust
use sha3::{Sha3_256, Digest};

fn append_to_hash_chain(prev_hash: &[u8], event: &AuditEvent) -> Vec<u8> {
    let mut hasher = Sha3_256::new();
    hasher.update(prev_hash);
    hasher.update(&event.timestamp.to_le_bytes());
    hasher.update(event.action.as_bytes());
    hasher.finalize().to_vec()
}
```

### 7.2 Digital Signatures

**Ed25519 (Proof Signing):**
```rust
use ed25519_dalek::{Keypair, Signature, Signer};

// Sign proof
let signature: Signature = keypair.sign(&proof_bytes);

// Verify signature
keypair.verify(&proof_bytes, &signature)?;
```

### 7.3 Key Management

**Rotation Policy:**
- JWT signing keys: Rotate every 90 days
- TLS certificates: Rotate every 365 days (Let's Encrypt auto-renewal)
- API keys: Rotate every 180 days

**Key Storage:**
- Development: Environment variables
- Production: AWS KMS, Azure Key Vault, HashiCorp Vault

---

## 8. Audit & Logging

### 8.1 Audit Events

**What to Log:**
```typescript
interface AuditEvent {
  timestamp: string;        // ISO 8601
  event_id: string;         // UUID v4
  user_id: string;          // client_id from JWT
  action: string;           // "verify.create", "policy.upload"
  resource_type: string;    // "proof", "policy"
  resource_id: string;      // Proof hash, policy ID
  status: "success" | "failure";
  ip_address: string;       // Source IP (anonymized after 90 days)
  user_agent: string;       // Browser info
  metadata: object;         // Additional context
  hash_chain: string;       // SHA3-256 of prev event
}
```

**Example:**
```json
{
  "timestamp": "2025-11-17T18:30:45.123Z",
  "event_id": "550e8400-e29b-41d4-a716-446655440000",
  "user_id": "cap-web-ui-prod",
  "action": "verify.create",
  "resource_type": "proof",
  "resource_id": "0xd847ab7566eab1c8c77417a157648d62483de89b010e955cd9cf298297ff803b",
  "status": "success",
  "ip_address": "203.0.113.42",
  "user_agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64)",
  "metadata": {
    "policy_id": "lksg.v1",
    "proof_size_bytes": 1024768
  },
  "hash_chain": "0x3f7a8c2b1e9d4f6a8c2b1e9d4f6a8c2b1e9d4f6a8c2b1e9d4f6a8c2b1e9d4f6a"
}
```

### 8.2 Log Retention

| Log Type | Retention Period | Storage |
|---------|-----------------|---------|
| Audit Logs | 10 years | Loki + S3 Glacier |
| Access Logs | 1 year | Loki |
| Error Logs | 90 days | Loki |
| Debug Logs | 7 days | Loki (dev only) |

### 8.3 SIEM Integration

**Export to SIEM (Splunk, ELK, Azure Sentinel):**
```nginx
# Nginx access log (JSON format)
log_format json_combined escape=json
  '{'
    '"timestamp":"$time_iso8601",'
    '"remote_addr":"$remote_addr",'
    '"request":"$request",'
    '"status":$status,'
    '"body_bytes_sent":$body_bytes_sent,'
    '"http_user_agent":"$http_user_agent",'
    '"request_time":$request_time'
  '}';

access_log /var/log/nginx/access.log json_combined;
```

---

## 9. Dependency Management

### 9.1 Supply Chain Security

**npm Audit (CI/CD):**
```bash
# Fail build on moderate+ vulnerabilities
npm audit --audit-level=moderate

# Auto-fix (with review)
npm audit fix
```

**Dependabot Configuration (.github/dependabot.yml):**
```yaml
version: 2
updates:
  - package-ecosystem: "npm"
    directory: "/web-ui"
    schedule:
      interval: "weekly"
    open-pull-requests-limit: 10
    reviewers:
      - "security-team"
    labels:
      - "dependencies"
      - "security"
```

### 9.2 SBOM Generation

**CycloneDX SBOM:**
```bash
# Generate SBOM
npx @cyclonedx/cyclonedx-npm --output-file sbom.json

# Validate SBOM
cyclonedx-cli validate --input-file sbom.json
```

**SBOM Format (CycloneDX 1.5):**
```json
{
  "bomFormat": "CycloneDX",
  "specVersion": "1.5",
  "serialNumber": "urn:uuid:3e671687-395b-41f5-a30f-a58921a69b79",
  "version": 1,
  "metadata": {
    "component": {
      "type": "application",
      "name": "cap-web-ui",
      "version": "1.0.0"
    }
  },
  "components": [
    {
      "type": "library",
      "name": "react",
      "version": "18.3.1",
      "purl": "pkg:npm/react@18.3.1",
      "licenses": [{ "license": { "id": "MIT" } }]
    }
  ]
}
```

### 9.3 License Compliance

**Allowed Licenses:**
- MIT, Apache-2.0, BSD-3-Clause, ISC

**Forbidden Licenses:**
- GPL, AGPL, SSPL (copyleft)

**Check Script:**
```bash
npx license-checker --summary --onlyAllow "MIT;Apache-2.0;BSD-3-Clause;ISC"
```

---

## 10. Deployment Security

### 10.1 Docker Security

**Dockerfile (Multi-stage, Non-root):**
```dockerfile
# Stage 1: Build
FROM node:20-alpine AS builder
WORKDIR /app
COPY package*.json ./
RUN npm ci --only=production
COPY . .
RUN npm run build

# Stage 2: Production
FROM nginx:1.25-alpine
COPY --from=builder /app/dist /usr/share/nginx/html
COPY nginx.conf /etc/nginx/nginx.conf

# Non-root user
RUN addgroup -g 1001 -S appuser && \
    adduser -u 1001 -S appuser -G appuser && \
    chown -R appuser:appuser /usr/share/nginx/html

USER appuser
EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD wget --quiet --tries=1 --spider http://localhost:8080/healthz || exit 1

CMD ["nginx", "-g", "daemon off;"]
```

**Image Scanning (Trivy):**
```bash
# Scan for vulnerabilities
trivy image --severity HIGH,CRITICAL cap-web-ui:latest

# Fail CI/CD if vulnerabilities found
trivy image --exit-code 1 --severity CRITICAL cap-web-ui:latest
```

### 10.2 Kubernetes Security

**Pod Security Policy:**
```yaml
apiVersion: v1
kind: Pod
metadata:
  name: cap-web-ui
spec:
  securityContext:
    runAsNonRoot: true
    runAsUser: 1001
    fsGroup: 1001
    seccompProfile:
      type: RuntimeDefault
  containers:
  - name: nginx
    image: cap-web-ui:latest
    securityContext:
      allowPrivilegeEscalation: false
      readOnlyRootFilesystem: true
      capabilities:
        drop:
          - ALL
    resources:
      limits:
        memory: "512Mi"
        cpu: "500m"
      requests:
        memory: "256Mi"
        cpu: "250m"
```

### 10.3 Environment Variables

**Secrets Management:**
```bash
# ❌ NEVER commit secrets
API_KEY=secret123  # BAD

# ✅ Use Kubernetes Secrets
kubectl create secret generic cap-web-ui-secrets \
  --from-literal=api-key=<SECRET>

# ✅ Or use external secret managers
# - AWS Secrets Manager
# - Azure Key Vault
# - HashiCorp Vault
```

---

## 11. Incident Response

### 11.1 Security Incident Categories

| Severity | Examples | Response Time |
|---------|---------|--------------|
| **Critical** | Data breach, RCE | 1 hour |
| **High** | XSS, CSRF, Auth bypass | 4 hours |
| **Medium** | DoS, Info disclosure | 24 hours |
| **Low** | Missing security header | 7 days |

### 11.2 Incident Response Plan

**Phase 1: Detection**
- SIEM alerts (Splunk, Azure Sentinel)
- WAF logs (ModSecurity)
- User reports (security@cap-system.com)

**Phase 2: Containment**
- Isolate affected systems (Kubernetes pod deletion)
- Revoke compromised credentials (JWT blacklist)
- Enable read-only mode (prevent further damage)

**Phase 3: Eradication**
- Patch vulnerability (deploy hotfix)
- Remove backdoors (audit codebase)
- Update firewall rules (block malicious IPs)

**Phase 4: Recovery**
- Restore from backup (PostgreSQL PITR)
- Verify system integrity (hash verification)
- Re-enable write access

**Phase 5: Lessons Learned**
- Post-mortem document
- Update runbooks
- Security awareness training

### 11.3 Breach Notification (DSGVO Art. 33)

**Timeline:**
- **72 hours** to notify supervisory authority (BfDI)
- **Without undue delay** to notify affected individuals

**Notification Template:**
```
Subject: Security Incident Notification - [Date]

Dear [Supervisory Authority],

We are writing to inform you of a personal data breach that occurred on [Date].

1. Nature of the breach: [XSS vulnerability exploited]
2. Categories of data affected: [User emails, verification timestamps]
3. Approximate number of data subjects: [~500 users]
4. Consequences: [Potential session hijacking]
5. Measures taken: [Patched XSS, revoked all sessions, notified users]

Contact: security@cap-system.com
```

---

## 12. Compliance Matrix

### 12.1 OWASP ASVS 4.0 (Level 2)

| Category | Requirement | Status |
|---------|------------|--------|
| V1 (Architecture) | Threat modeling performed | ✅ |
| V2 (Authentication) | OAuth2 with short-lived tokens | ✅ |
| V3 (Session) | Tokens in memory, not localStorage | ✅ |
| V4 (Access Control) | RBAC enforced | ✅ |
| V5 (Validation) | Zod input validation | ✅ |
| V6 (Cryptography) | TLS 1.3, BLAKE3, SHA3-256 | ✅ |
| V7 (Error Handling) | No stack traces in production | ✅ |
| V8 (Data Protection) | No PII in frontend | ✅ |
| V9 (Communications) | CSP, HSTS, CORS | ✅ |
| V10 (Malicious Code) | npm audit, SBOM | ✅ |
| V13 (API) | Rate limiting, authentication | ✅ |
| V14 (Configuration) | Security headers | ✅ |

### 12.2 NIST Cybersecurity Framework

| Function | Category | Implementation |
|---------|---------|----------------|
| **Identify** | Asset Management | SBOM, dependency tracking |
| **Protect** | Access Control | OAuth2, RBAC |
| **Detect** | Anomaly Detection | SIEM, WAF logs |
| **Respond** | Incident Response | 1-hour critical response |
| **Recover** | Backup & Recovery | PostgreSQL PITR, immutable logs |

### 12.3 ISO 27001:2022

| Annex A Control | Implementation |
|----------------|----------------|
| A.5.1 (Policies) | Security Design Document (this doc) |
| A.8.1 (Asset Management) | SBOM generation |
| A.8.23 (Web Filtering) | CSP, CORS, WAF |
| A.8.24 (Cryptography) | TLS 1.3, BLAKE3, SHA3-256 |
| A.12 (Logging) | Immutable audit trail (SHA3-256) |

### 12.4 BSI IT-Grundschutz

| Baustein | Maßnahme | Status |
|---------|---------|--------|
| CON.10 (Development) | Secure SDLC | ✅ |
| OPS.1.1.5 (Logging) | Centralized logging (Loki) | ✅ |
| APP.3.1 (Web Apps) | CSP, XSS prevention | ✅ |
| NET.3.2 (Firewalls) | WAF (ModSecurity) | ✅ |

### 12.5 DSGVO/GDPR

| Article | Requirement | Implementation |
|---------|------------|----------------|
| Art. 5 (Principles) | Data minimization | No PII in frontend |
| Art. 17 (Right to Erasure) | Delete user data | Anonymize audit logs |
| Art. 25 (Data Protection by Design) | Privacy by default | JWT in memory only |
| Art. 32 (Security) | Encryption | TLS 1.3, BLAKE3 |
| Art. 33 (Breach Notification) | 72-hour notification | Incident response plan |

---

## 13. Security Testing

### 13.1 Automated Security Testing (CI/CD)

```yaml
# .github/workflows/security.yml
name: Security Scan

on: [push, pull_request]

jobs:
  security:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: npm audit
        run: npm audit --audit-level=moderate

      - name: Trivy scan
        uses: aquasecurity/trivy-action@master
        with:
          scan-type: 'fs'
          severity: 'HIGH,CRITICAL'

      - name: OWASP Dependency Check
        uses: dependency-check/Dependency-Check_Action@main

      - name: Snyk scan
        uses: snyk/actions/node@master
        env:
          SNYK_TOKEN: ${{ secrets.SNYK_TOKEN }}
```

### 13.2 Manual Penetration Testing

**Scope:**
- OWASP Top 10 testing
- Authentication bypass attempts
- Authorization flaws
- Business logic vulnerabilities

**Frequency:** Annually (before major releases)

**Provider:** External security firm (e.g., Cure53, NCC Group)

---

## 14. Security Checklist (Pre-Production)

### 14.1 Deployment Readiness

- [ ] All dependencies updated (`npm audit` clean)
- [ ] SBOM generated and reviewed
- [ ] TLS 1.3 configured
- [ ] CSP headers enforced
- [ ] HSTS enabled (2-year max-age)
- [ ] CORS allowlist configured
- [ ] Rate limiting enabled (10 req/min)
- [ ] JWT lifetime set to 15 minutes
- [ ] No secrets in environment variables (use vault)
- [ ] Docker image scanned (Trivy CRITICAL = 0)
- [ ] Non-root user in Docker container
- [ ] Read-only root filesystem
- [ ] Health checks configured
- [ ] Audit logging enabled
- [ ] SIEM integration tested
- [ ] Incident response plan documented
- [ ] Backup & recovery tested

### 14.2 Code Review Checklist

- [ ] No `dangerouslySetInnerHTML` usage
- [ ] All user input validated (Zod schemas)
- [ ] No secrets in code (API keys, tokens)
- [ ] Error messages don't leak info
- [ ] Logging doesn't contain PII
- [ ] CSRF tokens on all state-changing requests
- [ ] Authorization checks on protected routes
- [ ] File uploads validated (size, type, magic bytes)

---

## 15. Glossary

| Term | Definition |
|------|-----------|
| **APT** | Advanced Persistent Threat |
| **BfDI** | Bundesbeauftragte für den Datenschutz und die Informationsfreiheit (German DPA) |
| **BSI** | Bundesamt für Sicherheit in der Informationstechnik (German Federal Office for Information Security) |
| **CSP** | Content Security Policy |
| **CSRF** | Cross-Site Request Forgery |
| **CVSS** | Common Vulnerability Scoring System |
| **DSGVO** | Datenschutz-Grundverordnung (German GDPR) |
| **HSTS** | HTTP Strict Transport Security |
| **JWT** | JSON Web Token |
| **OWASP** | Open Web Application Security Project |
| **PII** | Personally Identifiable Information |
| **RBAC** | Role-Based Access Control |
| **SBOM** | Software Bill of Materials |
| **SIEM** | Security Information and Event Management |
| **TLS** | Transport Layer Security |
| **XSS** | Cross-Site Scripting |

---

## 16. References

1. OWASP Top 10 (2021): https://owasp.org/Top10/
2. OWASP ASVS 4.0: https://owasp.org/www-project-application-security-verification-standard/
3. NIST Cybersecurity Framework: https://www.nist.gov/cyberframework
4. ISO/IEC 27001:2022: https://www.iso.org/standard/27001
5. BSI IT-Grundschutz: https://www.bsi.bund.de/EN/Topics/IT-Grundschutz/
6. DSGVO (GDPR): https://eur-lex.europa.eu/eli/reg/2016/679/oj
7. JWT Best Practices (RFC 8725): https://datatracker.ietf.org/doc/html/rfc8725
8. CSP Level 3: https://www.w3.org/TR/CSP3/
9. OAuth 2.0 (RFC 6749): https://datatracker.ietf.org/doc/html/rfc6749
10. CycloneDX SBOM: https://cyclonedx.org/

---

**Document Control:**
- **Version:** 1.0
- **Author:** CAP Security Team
- **Reviewed By:** CISO
- **Next Review:** 2026-05-17 (6 months)
- **Classification:** Internal - Security Architecture

**Change Log:**
- 2025-11-17: Initial version (v1.0)

---

**Approval:**

| Role | Name | Signature | Date |
|------|------|-----------|------|
| Security Architect | [Name] | [Signature] | 2025-11-17 |
| CISO | [Name] | [Signature] | 2025-11-17 |
| Legal/DPO | [Name] | [Signature] | 2025-11-17 |

---

*END OF DOCUMENT*
