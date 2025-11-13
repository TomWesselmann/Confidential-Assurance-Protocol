# WEEK7_S — HSM/TPM/KMS Key‑Management

**Ziel:** Signaturen aus Hardware/KMS, KID‑Politik, Rotation sicher integriert. Software‑Signer bleibt als Fallback.

## Architektur
- Trait `KeyProvider`:
  ```rust
  pub trait KeyProvider: Send + Sync {
      fn provider_id(&self) -> &'static str;
      fn current_kid(&self) -> Result<String, KeyError>;
      fn sign(&self, kid: Option<&str>, msg: &[u8]) -> Result<Vec<u8>, KeyError>;
      fn public_key(&self, kid: &str) -> Result<Vec<u8>, KeyError>;
      fn list_kids(&self) -> Result<Vec<String>, KeyError>;
  }
  ```
- Provider: `software` (ed25519), `pkcs11` (HSM/SoftHSM2), `cloudkms` (GCP/AWS/Azure)
- **KID:** `blake3(pubkey || provider_id || key_name)` (deterministisch)

## Config (Beispiel `config/key.yaml`)
```yaml
key:
  provider: pkcs11     # software|pkcs11|cloudkms
  pkcs11:
    module: /usr/lib/softhsm/libsofthsm2.so
    slot: 0
    pin_env: PKCS11_PIN
    key_label: cap-signing
  cloudkms:
    project: my-proj
    location: eu
    keyring: cap-ring
    key: cap-signer
    version: latest
```

## CLI
- `cap keys info` → Provider/KIDs
- `cap keys sign --in msg.bin --kid <opt>`
- `cap verify-signature --in msg.bin --sig sig.bin --kid KID`

## Integration
- Registry‑Signaturen mit `kid`; Verifier akzeptiert gemäß Rotation‑Politik (Dual‑Accept → Decom)

## Akzeptanzkriterien (DoD)
1. **Provider‑Parity:** Signatur‑Verifikation über alle Provider identisch/äquivalent
2. **KID‑Stabilität:** Gleiches Key‑Material ⇒ gleicher KID
3. **Rotation:** Vor T1 alt+neu OK; nach T1 alt FAIL; Rollback 2→1 / 3→2 OK
4. **Fehlpfade:** Token locked/Pin falsch/KMS‑Timeout → fail‑closed, klare Fehlercodes, keine PII
5. **Latenz:** p95 Sign ≤ 50ms (CloudKMS ≤ 150ms), p99 ≤ 200ms

## Tests & Befehle
```bash
cargo test --test key_provider_unit -- --nocapture

# PKCS11 (SoftHSM2, benötigt Env)
SOFTHSM2_CONF=tests/softhsm2.conf PKCS11_PIN=1234       cargo test --test key_provider_pkcs11_it -- --ignored --nocapture

# CloudKMS (env‑driven, ignored)
GCP_PROJECT=... GCP_KEY=...       cargo test --test key_provider_cloudkms_it -- --ignored --nocapture

# Rotation (bestehende Suite)
cargo test --test rotation -- --nocapture
```

## Dateien (neu/ändern)
```
src/providers/key_provider.rs
src/providers/software.rs
src/providers/pkcs11.rs
src/providers/cloudkms.rs
src/bin/cap.rs        # keys‑CLI
tests/key_provider_unit.rs
tests/key_provider_pkcs11_it.rs
tests/key_provider_cloudkms_it.rs
```