# Key Rotation Runbook (Week 6 - Track D2)

## Overview

This runbook describes the procedure for rotating Ed25519 signing keys used by the CAP Verifier API. The rotation process is designed to be **zero-downtime**, **deterministic**, and **backwards-compatible** using a phased approach with **KID (Key Identifier)** based dual-acceptance.

## Objectives

- **Zero Downtime:** No service interruption during rotation
- **Dual-Accept Period:** Both old and new keys accepted during transition (T1)
- **Deterministic:** Same key rotation produces same KIDs and hashes
- **Audit Trail:** All rotation events logged with timestamps
- **Security:** Old private keys securely decommissioned after rotation

## Key Identifier (KID) System

The CAP Verifier API uses **KID-based key identification** for Ed25519 signatures:

**KID Derivation:**
```
kid = blake3(base64(public_key))[0:16]  // 128 bits = 32 hex chars
```

**Properties:**
- ✅ Deterministic: Same public key → same KID
- ✅ Collision-resistant: BLAKE3 truncated to 128 bits
- ✅ Globally unique: KID ties signature to specific key
- ✅ Rotation-friendly: Old and new KIDs coexist during dual-accept

**Example KID:**
```
a010ac65166984697b93b867c36e9c94
```

## Rotation Phases

### Phase 0: Preparation

**Duration:** 1-2 days before rotation
**Goal:** Generate new key pair, prepare configuration

**Steps:**

1. **Generate New Key Pair**
   ```bash
   cargo run -- keys keygen \
     --owner "CompanyName" \
     --out keys/company_new.v1.json \
     --valid-days 730 \
     --comment "Rotation $(date +%Y-%m-%d)"
   ```

   **Output:**
   ```
   ✅ Key generated: kid=b234de78901ab567cdef1234567890ab
   📄 Metadata: keys/company_new.v1.json
   🔐 Private key: keys/company_new.v1.ed25519
   🔓 Public key: keys/company_new.v1.pub
   ```

2. **Verify New Key**
   ```bash
   cargo run -- keys show \
     --dir keys \
     --kid b234de78901ab567cdef1234567890ab
   ```

   **Expected:**
   ```json
   {
     "schema": "cap-key.v1",
     "kid": "b234de78901ab567cdef1234567890ab",
     "owner": "CompanyName",
     "status": "active",
     "algorithm": "ed25519",
     "created_at": "2025-11-10T10:00:00Z",
     "valid_from": "2025-11-10T10:00:00Z",
     "valid_to": "2027-11-10T10:00:00Z"
   }
   ```

3. **Attest New Key with Old Key (Chain of Trust)**
   ```bash
   cargo run -- keys attest \
     --signer keys/company.v1.json \
     --subject keys/company_new.v1.json \
     --out keys/attestation_$(date +%Y%m%d).json
   ```

   **Output:**
   ```json
   {
     "attestation": {
       "schema": "cap-attestation.v1",
       "signer_kid": "a010ac65166984697b93b867c36e9c94",
       "subject_kid": "b234de78901ab567cdef1234567890ab",
       "attested_at": "2025-11-10T10:05:00Z"
     },
     "signature": "base64...",
     "signer_public_key": "base64..."
   }
   ```

4. **Store New Key in KMS/Vault**
   ```bash
   # Store private key in vault (DO NOT commit to git!)
   vault kv put secret/cap/keys/company_new \
     private_key=@keys/company_new.v1.ed25519 \
     kid=b234de78901ab567cdef1234567890ab

   # Store public key metadata in version control
   git add keys/company_new.v1.json keys/company_new.v1.pub
   git commit -m "feat(keys): Add new key for rotation (KID: b234de78...)"
   ```

5. **Update Kubernetes Secret (Dual-Key Config)**
   ```bash
   # Create dual-key secret (both old and new)
   kubectl -n cap create secret generic cap-keys-dual \
     --from-file=old=keys/company.v1.ed25519 \
     --from-file=new=keys/company_new.v1.ed25519 \
     --from-file=old-metadata=keys/company.v1.json \
     --from-file=new-metadata=keys/company_new.v1.json \
     --dry-run=client -o yaml | kubectl apply -f -
   ```

**Validation Checklist:**
- [ ] New key pair generated successfully
- [ ] KID derived correctly (32 hex chars)
- [ ] Attestation created and verified
- [ ] Private key stored in KMS/Vault
- [ ] Public key metadata in version control
- [ ] Kubernetes secret updated with dual keys

---

### Phase 1: Dual-Accept (T1 Start)

**Duration:** 7-14 days (configurable grace period)
**Goal:** Accept signatures from **both** old and new keys

**Steps:**

1. **Update API Configuration (Dual-Accept Mode)**

   Update `helm/values-prod.yaml`:
   ```yaml
   verifier:
     signing:
       mode: dual-accept
       keys:
         - kid: "a010ac65166984697b93b867c36e9c94"  # Old key
           status: active
           path: /keys/old
         - kid: "b234de78901ab567cdef1234567890ab"  # New key
           status: active
           path: /keys/new
       dual_accept_until: "2025-11-24T10:00:00Z"  # T1 end date
   ```

2. **Deploy Updated Configuration**
   ```bash
   helm upgrade cap helm/ \
     -f helm/values-prod.yaml \
     --namespace cap \
     --wait --timeout 10m
   ```

3. **Verify Dual-Accept Active**

   **Test with Old Key:**
   ```bash
   # Sign test entry with old key
   cargo run -- registry add \
     --manifest build/test_manifest.json \
     --proof build/test_proof.dat \
     --signing-key keys/company.v1.ed25519 \
     --registry build/registry.sqlite

   # Should succeed
   echo "Expected: Entry signed with old KID (a010ac65...)"
   ```

   **Test with New Key:**
   ```bash
   # Sign test entry with new key
   cargo run -- registry add \
     --manifest build/test_manifest2.json \
     --proof build/test_proof2.dat \
     --signing-key keys/company_new.v1.ed25519 \
     --registry build/registry.sqlite

   # Should succeed
   echo "Expected: Entry signed with new KID (b234de78...)"
   ```

4. **Monitor Metrics**

   **Grafana Dashboard:**
   - `cap_signatures_verified_total{kid="a010ac65..."}` - Old key usage (should decline)
   - `cap_signatures_verified_total{kid="b234de78..."}` - New key usage (should increase)
   - `cap_signature_verification_failures_total` - Should remain 0

5. **Notify Stakeholders**

   **Email/Slack Announcement:**
   ```
   📢 Key Rotation Alert: Dual-Accept Phase Started

   Duration: 2025-11-10 to 2025-11-24 (14 days)
   Old KID: a010ac65166984697b93b867c36e9c94
   New KID: b234de78901ab567cdef1234567890ab

   Impact: NONE - Both keys accepted
   Action Required: None (informational only)

   Next Phase: Sign-Switch on 2025-11-17 (7 days)
   ```

**Validation Checklist:**
- [ ] Dual-accept mode enabled in API
- [ ] Both old and new keys accepted
- [ ] Test signatures verify successfully (both KIDs)
- [ ] Metrics show dual-key usage
- [ ] No signature verification failures
- [ ] Stakeholders notified

---

### Phase 2: Sign-Switch (New Key Active)

**Duration:** 7 days (during dual-accept window)
**Goal:** Switch to signing with **new key** while still accepting **old signatures**

**Steps:**

1. **Update Default Signing Key**

   Update `helm/values-prod.yaml`:
   ```yaml
   verifier:
     signing:
       mode: dual-accept  # Still accepting old
       default_key: /keys/new  # Now signing with new
       keys:
         - kid: "a010ac65166984697b93b867c36e9c94"  # Old: verify only
           status: active
           path: /keys/old
         - kid: "b234de78901ab567cdef1234567890ab"  # New: sign + verify
           status: active
           path: /keys/new
       dual_accept_until: "2025-11-24T10:00:00Z"
   ```

2. **Deploy Configuration**
   ```bash
   helm upgrade cap helm/ \
     -f helm/values-prod.yaml \
     --namespace cap \
     --wait
   ```

3. **Verify New Key Signing**

   **Create New Registry Entry:**
   ```bash
   cargo run -- registry add \
     --manifest build/new_manifest.json \
     --proof build/new_proof.dat \
     --signing-key keys/company_new.v1.ed25519 \
     --registry build/registry.sqlite
   ```

   **Verify Entry:**
   ```bash
   sqlite3 build/registry.sqlite \
     "SELECT kid FROM registry_entries ORDER BY id DESC LIMIT 1;"
   # Expected: b234de78901ab567cdef1234567890ab (new KID)
   ```

4. **Verify Old Signatures Still Accepted**

   **Retrieve Old Entry:**
   ```bash
   curl -s -H "Authorization: Bearer $TOKEN" \
     "$API_BASE/registry/entry/old_manifest_hash" | jq '.kid'
   # Expected: a010ac65166984697b93b867c36e9c94 (old KID, still valid)
   ```

5. **Monitor Transition**

   **Grafana Metrics:**
   - `cap_signatures_created_total{kid="b234de78..."}` → Should be 100% of new signatures
   - `cap_signatures_verified_total{kid="a010ac65..."}` → Should be declining (old entries only)
   - `cap_signatures_verified_total{kid="b234de78..."}` → Should be increasing (new + old entries)

**Validation Checklist:**
- [ ] All new signatures use new KID
- [ ] Old signatures still verify successfully
- [ ] No verification failures
- [ ] Metrics show new key dominance
- [ ] Old entries still accessible

---

### Phase 3: Decommission (T1 End)

**Duration:** After dual-accept period expires
**Goal:** Reject old key signatures, full transition to new key

**Steps:**

1. **Update Configuration (Decommission Old Key)**

   Update `helm/values-prod.yaml`:
   ```yaml
   verifier:
     signing:
       mode: single-key  # Only new key accepted
       default_key: /keys/new
       keys:
         - kid: "a010ac65166984697b93b867c36e9c94"  # Old: DECOMMISSIONED
           status: retired
           path: /keys/old
         - kid: "b234de78901ab567cdef1234567890ab"  # New: active
           status: active
           path: /keys/new
   ```

2. **Deploy Configuration**
   ```bash
   helm upgrade cap helm/ \
     -f helm/values-prod.yaml \
     --namespace cap \
     --wait
   ```

3. **Verify Old Key Rejected**

   **Test Old Signature Verification (Should Fail):**
   ```bash
   # Try to verify old entry
   curl -s -H "Authorization: Bearer $TOKEN" \
     "$API_BASE/registry/entry/old_manifest_hash" | jq '.error'
   # Expected: "Signature verification failed: key retired"
   ```

4. **Archive Old Key**
   ```bash
   cargo run -- keys archive \
     --dir keys \
     --kid a010ac65166984697b93b867c36e9c94
   ```

   **Expected:**
   ```
   ✅ Key a010ac65166984697b93b867c36e9c94 archived
   📁 Moved to: keys/archive/company.v1.json
   ```

5. **Remove Old Private Key from KMS**
   ```bash
   # Mark as retired in vault (do NOT delete immediately for audit)
   vault kv metadata put secret/cap/keys/company \
     custom_metadata="status=retired,retired_at=$(date -Iseconds)"

   # Optional: Move to archive path
   vault kv put secret/cap/keys/archive/company_$(date +%Y%m%d) \
     @secret/cap/keys/company
   ```

6. **Update Key Rotation Log**
   ```bash
   echo "$(date -Iseconds),a010ac65166984697b93b867c36e9c94,b234de78901ab567cdef1234567890ab,decommissioned" \
     >> docs/key_rotation.log
   ```

**Validation Checklist:**
- [ ] Old key signatures rejected
- [ ] New key signatures accepted
- [ ] Old key archived (status: retired)
- [ ] Old private key removed from active KMS
- [ ] Rotation logged in audit trail
- [ ] No service disruption

---

## Rollback Procedure

If issues occur during rotation, rollback to previous phase:

### Rollback from Sign-Switch → Dual-Accept

**Scenario:** New key has issues, need to revert to old key for signing

**Steps:**
1. Update `helm/values-prod.yaml`: Set `default_key: /keys/old`
2. Deploy: `helm upgrade cap helm/ -f helm/values-prod.yaml`
3. Verify: Old key now used for new signatures
4. Investigate: Review logs for new key issues

### Rollback from Decommission → Sign-Switch

**Scenario:** Old entries need to be re-verified after decommission

**Steps:**
1. Update `helm/values-prod.yaml`:
   ```yaml
   signing:
     mode: dual-accept
     keys:
       - kid: "a010ac65..."
         status: active  # Re-activate old key
   ```
2. Deploy: `helm upgrade cap helm/ -f helm/values-prod.yaml`
3. Verify: Old signatures now verify again
4. Extend T1 deadline: Update `dual_accept_until` to new date

### Emergency Rollback (Full Revert)

**Scenario:** Critical issue with new key, need to fully revert

**Steps:**
1. Stop rotation immediately
2. Update configuration to use old key exclusively
3. Deploy configuration
4. Remove new key from production
5. Conduct post-mortem investigation
6. Plan new rotation with fixes

---

## Testing Strategy

### Unit Tests

**File:** `tests/rotation.rs`

**Test Cases:**
1. `test_accepts_old_and_new_before_T1` - Dual-accept phase validation
2. `test_rejects_old_after_T1` - Decommission validation
3. `test_new_key_signs_correctly` - Sign-switch validation
4. `test_kid_derivation_deterministic` - KID consistency
5. `test_attestation_chain_valid` - Chain of trust

### Integration Tests

**Environment:** Staging cluster with test keys

**Scenarios:**
1. Full rotation cycle (Prep → Dual → Switch → Decom)
2. Rollback from each phase
3. Expired dual-accept period (automatic decommission)
4. Multiple key rotations (A → B → C)

### Load Tests

**Scenario:** Rotation under production load

**Metrics:**
- Latency impact (should be < 5ms increase)
- Error rate (should remain 0%)
- Throughput (should not decrease)

---

## Monitoring & Alerting

### Prometheus Metrics

```promql
# Signatures created by KID
cap_signatures_created_total{kid="$kid"}

# Signatures verified by KID
cap_signatures_verified_total{kid="$kid",status="success|failure"}

# Key rotation phase
cap_key_rotation_phase{phase="preparation|dual_accept|sign_switch|decommission"}

# Dual-accept deadline
cap_dual_accept_deadline_timestamp
```

### Grafana Dashboard

**Panels:**
1. **Key Usage Over Time**
   - Line chart: `cap_signatures_created_total` by KID
   - Shows transition from old to new key

2. **Verification Success Rate**
   - Gauge: `cap_signatures_verified_total{status="success"}` / total
   - Should remain 100% during rotation

3. **Rotation Phase**
   - State timeline: Current rotation phase
   - Alerts if stuck in a phase too long

4. **Dual-Accept Countdown**
   - Time until T1 deadline
   - Warning if deadline approaching

### Alerts

```yaml
- alert: KeyRotationStalled
  expr: cap_key_rotation_phase{phase="dual_accept"} == 1 for 21d
  labels:
    severity: warning
  annotations:
    summary: "Key rotation in dual-accept for >21 days"

- alert: DualAcceptDeadlineApproaching
  expr: cap_dual_accept_deadline_timestamp - time() < 86400
  labels:
    severity: warning
  annotations:
    summary: "Dual-accept deadline in <24 hours"

- alert: OldKeyUsageAfterDecommission
  expr: rate(cap_signatures_created_total{kid="$old_kid"}[5m]) > 0 and cap_key_rotation_phase{phase="decommission"} == 1
  labels:
    severity: critical
  annotations:
    summary: "Old key still being used after decommission!"
```

---

## Security Considerations

### Private Key Handling

- ✅ **Generation:** Generate keys on secure hardware (HSM or Vault transit engine)
- ✅ **Storage:** Store in KMS/Vault with access logging
- ✅ **Transport:** Never transmit over unencrypted channels
- ✅ **Access:** Multi-party approval for key access (2 of 3)
- ✅ **Rotation:** Rotate every 12-24 months minimum
- ✅ **Decommission:** Securely delete from active systems, archive in cold storage

### Audit Trail

Every rotation event is logged:
```jsonl
{"timestamp":"2025-11-10T10:00:00Z","event":"key_generated","kid":"b234de78...","owner":"CompanyName"}
{"timestamp":"2025-11-10T10:05:00Z","event":"key_attested","signer_kid":"a010ac65...","subject_kid":"b234de78..."}
{"timestamp":"2025-11-10T12:00:00Z","event":"dual_accept_started","old_kid":"a010ac65...","new_kid":"b234de78...","deadline":"2025-11-24T10:00:00Z"}
{"timestamp":"2025-11-17T10:00:00Z","event":"sign_switch","new_kid":"b234de78..."}
{"timestamp":"2025-11-24T10:00:00Z","event":"key_decommissioned","old_kid":"a010ac65..."}
```

### Compliance

- **SOC 2:** Key rotation procedures documented and tested quarterly
- **ISO 27001:** Cryptographic key management policy enforced
- **GDPR:** No PII in key metadata or audit logs
- **LkSG:** Audit trail for all compliance-related signatures

---

## Troubleshooting

### Issue 1: KID Mismatch After Rotation

**Symptom:** New key has different KID than expected

**Cause:** Public key encoding or hash algorithm mismatch

**Solution:**
```bash
# Verify KID derivation
cargo run -- keys show --dir keys --kid b234de78...

# Recompute KID manually
echo -n "$(base64 keys/company_new.v1.pub)" | blake3sum | cut -c1-32
```

### Issue 2: Old Signatures Fail During Dual-Accept

**Symptom:** Old entries fail verification during Phase 1

**Cause:** Old key not loaded or status incorrect

**Solution:**
```bash
# Check loaded keys
kubectl -n cap exec $POD -- ls -lh /keys/

# Verify old key status
cargo run -- keys show --dir keys --kid a010ac65...
# Status should be "active" during dual-accept
```

### Issue 3: New Key Not Signing After Sign-Switch

**Symptom:** New entries still use old KID after Phase 2

**Cause:** Default key configuration not updated

**Solution:**
```bash
# Check current signing key
kubectl -n cap exec $POD -- cat /app/config/signing.yaml

# Verify default_key points to /keys/new
# If not, update helm values and redeploy
```

---

## References

- [Ed25519 Signature Scheme](https://ed25519.cr.yp.to/)
- [BLAKE3 Hash Function](https://github.com/BLAKE3-team/BLAKE3)
- [Key Rotation Best Practices (NIST SP 800-57)](https://csrc.nist.gov/publications/detail/sp/800-57-part-1/rev-5/final)
- [Zero-Downtime Key Rotation (Google Cloud)](https://cloud.google.com/kms/docs/key-rotation)

---

**Document Version:** 1.0.0
**Last Updated:** 2025-11-10
**Author:** CAP Engineering Team
**Reviewed By:** Security Team
**Next Review:** 2025-12-10 (Monthly)
**Rotation Frequency:** Every 12 months (or upon compromise)
