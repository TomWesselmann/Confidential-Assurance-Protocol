# Backup & Restore Runbook (Week 6 - Track D1)

## Overview

This runbook describes the procedures for backing up and restoring the CAP Verifier API system to ensure business continuity and disaster recovery. The backup/restore process is designed to be **deterministic** and **evidence-preserving**, meaning that restored systems produce identical hashes and ETags as the original system.

## Objectives

- **RTO (Recovery Time Objective):** < 30 minutes for full restore
- **RPO (Recovery Point Objective):** < 1 hour (hourly backups)
- **Determinism:** Restored system produces identical `ir_hash`, `policy_hash`, `manifest_hash`, and ETags
- **Privacy:** No PII in backups (only commitments, hashes, configurations)
- **Automation:** Fully scripted backup and restore procedures

## Backup Scope

### What is Backed Up ✅

1. **IR Registry** (Intermediate Representation)
   - Format: JSON or SQLite
   - Location: `build/registry.json` or `build/registry.sqlite`
   - Content: Policy hashes, manifest hashes, proof hashes, signatures, KIDs
   - Size: ~10-100 MB (depending on number of entries)

2. **Policy Store** (In-Memory → Serialized)
   - Format: JSON (exported snapshot)
   - Location: `build/policy_store.json`
   - Content: Compiled policies with hashes
   - Size: ~1-10 MB

3. **Configuration Files**
   - `helm/values-prod.yaml` - Kubernetes deployment configuration
   - `openapi/openapi.yaml` - API specification
   - `grafana/dashboards/*.json` - Monitoring dashboards
   - `prometheus/alerts.yml` - Alerting rules
   - Size: ~1-5 MB

4. **Key Metadata** (Public keys only)
   - Format: JSON (`cap-key.v1`)
   - Location: `keys/*.v1.json`, `keys/trusted/*.pub`
   - Content: KIDs, public keys, metadata (NOT private keys!)
   - Size: ~100 KB

5. **Documentation**
   - `docs/*.md` - Operational documentation
   - `runbooks/*.md` - This runbook and others
   - Size: ~5-10 MB

### What is NOT Backed Up ❌

1. **Private Keys** (`keys/*.ed25519`)
   - Reason: Stored separately in KMS/Vault (HashiCorp Vault, AWS KMS, etc.)
   - Recovery: Retrieved from KMS during restore

2. **Secrets** (passwords, tokens)
   - Reason: Stored in Kubernetes Secrets or external vault
   - Recovery: Re-applied from secure storage during restore

3. **PII** (Personally Identifiable Information)
   - Reason: GDPR compliance, unnecessary for system restore
   - Content: All data in backups are hashes/commitments only

4. **Temporary Files** (`/tmp`, logs)
   - Reason: Ephemeral data, not needed for restore
   - Recovery: Recreated automatically post-restore

## Backup Manifest Format

All backups include a `backup.manifest.json` file with SHA3-256 hashes of all included files for integrity verification.

```json
{
  "version": "backup.manifest.v1",
  "created_at": "2025-11-10T12:00:00Z",
  "created_by": "cap-backup-automation",
  "backup_id": "backup-20251110-120000",
  "system_version": "0.11.0",
  "files": [
    {
      "path": "registry.sqlite",
      "sha3_256": "0x1da941f7026bae3cf8b1bcdc3a8e01e76ea678c32ec6bc2c374fb67b3744571f",
      "size_bytes": 10485760,
      "type": "database"
    },
    {
      "path": "policy_store.json",
      "sha3_256": "0x83a8779ddef456789012345678901234567890123456789012345678901234567890",
      "size_bytes": 1048576,
      "type": "policy_store"
    },
    {
      "path": "keys/metadata/",
      "sha3_256": "0xabc123def456789012345678901234567890123456789012345678901234567890",
      "size_bytes": 102400,
      "type": "key_metadata"
    },
    {
      "path": "config/helm/values-prod.yaml",
      "sha3_256": "0xdef456789012345678901234567890123456789012345678901234567890123456",
      "size_bytes": 8192,
      "type": "configuration"
    }
  ],
  "total_files": 15,
  "total_size_bytes": 12582912,
  "compression": "gzip",
  "encryption": "aes-256-gcm"
}
```

## Backup Procedure

### Automated Backup (Recommended)

**Schedule:** Hourly via Kubernetes CronJob

**Command:**
```bash
./scripts/backup.sh \
  --output /backup/cap-backup-$(date +%Y%m%d-%H%M%S).tar.gz \
  --registry build/registry.sqlite \
  --policy-store build/policy_store.json \
  --keys keys/ \
  --config helm/values-prod.yaml \
  --manifest /backup/backup.manifest.json
```

**CronJob Configuration** (`k8s/cronjob-backup.yaml`):
```yaml
apiVersion: batch/v1
kind: CronJob
metadata:
  name: cap-backup
  namespace: cap
spec:
  schedule: "0 * * * *"  # Hourly at minute 0
  jobTemplate:
    spec:
      template:
        spec:
          serviceAccountName: cap-backup-sa
          containers:
          - name: backup
            image: cap-agent:0.11.0
            command:
            - /scripts/backup.sh
            args:
            - --output
            - /backup/cap-backup-$(date +%Y%m%d-%H%M%S).tar.gz
            volumeMounts:
            - name: backup-volume
              mountPath: /backup
            - name: registry-volume
              mountPath: /app/build
            - name: keys-volume
              mountPath: /app/keys
              readOnly: true
          volumes:
          - name: backup-volume
            persistentVolumeClaim:
              claimName: cap-backup-pvc
          - name: registry-volume
            persistentVolumeClaim:
              claimName: cap-registry-pvc
          - name: keys-volume
            secret:
              secretName: cap-keys-public
          restartPolicy: OnFailure
```

### Manual Backup

**Prerequisites:**
- kubectl access to production cluster
- Read access to `/backup` PVC

**Steps:**

1. **Connect to Cluster**
   ```bash
   kubectl config use-context prod-cluster
   kubectl -n cap get pods
   ```

2. **Execute Backup Script**
   ```bash
   POD=$(kubectl -n cap get pod -l app=cap-verifier-api -o jsonpath='{.items[0].metadata.name}')
   kubectl -n cap exec -it $POD -- /scripts/backup.sh \
     --output /backup/manual-backup-$(date +%Y%m%d-%H%M%S).tar.gz
   ```

3. **Download Backup (Optional)**
   ```bash
   kubectl -n cap cp $POD:/backup/manual-backup-*.tar.gz ./local-backup.tar.gz
   ```

4. **Verify Backup Integrity**
   ```bash
   tar -tzf ./local-backup.tar.gz | head -20
   jq '.files[].sha3_256' < backup.manifest.json
   ```

### Backup Storage

**Primary:** Kubernetes PersistentVolume (`cap-backup-pvc`)
- Retention: 7 days (168 backups at hourly frequency)
- Storage Class: `fast-ssd` (AWS EBS gp3, Azure Premium SSD)
- Access Mode: `ReadWriteMany` (NFS or CSI driver)

**Secondary:** S3/Azure Blob/GCS
- Retention: 30 days
- Lifecycle Policy: Transition to Glacier after 7 days
- Replication: Cross-region (us-east-1 → eu-central-1)

**Offsite:** Tape/Offline Storage
- Retention: 1 year (monthly snapshots)
- Encryption: AES-256 (keys stored separately)

## Restore Procedure

### Prerequisites

1. **Empty Target Namespace**
   ```bash
   kubectl create namespace cap-restore
   kubectl config set-context --current --namespace=cap-restore
   ```

2. **Backup Archive Available**
   ```bash
   ls -lh /backup/cap-backup-20251110-120000.tar.gz
   ```

3. **Secrets/Keys in KMS**
   ```bash
   # Verify key access
   vault kv get secret/cap/keys/company
   ```

### Restore Steps

#### Step 1: Extract Backup Archive

```bash
cd /restore
tar -xzf /backup/cap-backup-20251110-120000.tar.gz
```

**Expected Output:**
```
backup.manifest.json
registry.sqlite
policy_store.json
keys/metadata/
config/helm/values-prod.yaml
...
```

#### Step 2: Verify Backup Integrity

```bash
./scripts/restore.sh --verify-only \
  --backup-dir /restore \
  --manifest /restore/backup.manifest.json
```

**Expected Output:**
```
🔍 Verifying backup integrity...
✅ registry.sqlite: SHA3-256 matches (0x1da941f7...)
✅ policy_store.json: SHA3-256 matches (0x83a8779d...)
✅ keys/metadata/: SHA3-256 matches (0xabc123def...)
✅ config/helm/values-prod.yaml: SHA3-256 matches (0xdef45678...)
✅ All 15 files verified successfully
```

#### Step 3: Deploy Helm Chart (Empty State)

```bash
helm upgrade --install cap-restore helm/ \
  -f /restore/config/helm/values-prod.yaml \
  --namespace cap-restore \
  --wait --timeout 10m
```

**Validation:**
```bash
kubectl -n cap-restore get deploy,po,svc
```

**Expected:** All pods running, service exposed

#### Step 4: Restore Registry

```bash
# Copy registry to pod
POD=$(kubectl -n cap-restore get pod -l app=cap-verifier-api -o jsonpath='{.items[0].metadata.name}')
kubectl -n cap-restore cp /restore/registry.sqlite $POD:/app/build/registry.sqlite

# Verify registry loaded
kubectl -n cap-restore exec $POD -- ls -lh /app/build/registry.sqlite
```

#### Step 5: Restore Policy Store

```bash
# Load policy store via API (POST /policy/restore)
curl -s -k -X POST \
  -H "Authorization: Bearer $RESTORE_TOKEN" \
  -H "Content-Type: application/json" \
  -d @/restore/policy_store.json \
  https://cap-restore.example.com/api/v1/policy/restore
```

**Expected Response:**
```json
{
  "status": "ok",
  "policies_restored": 25,
  "total_size_bytes": 1048576
}
```

#### Step 6: Restore Key Metadata

```bash
# Copy key metadata to pod
kubectl -n cap-restore cp /restore/keys/metadata/ $POD:/app/keys/

# Retrieve private keys from KMS
vault kv get -format=json secret/cap/keys/company | \
  jq -r '.data.data.private_key' | \
  base64 -d > /tmp/company.ed25519

kubectl -n cap-restore cp /tmp/company.ed25519 $POD:/app/keys/company.ed25519
rm /tmp/company.ed25519  # Clean up local copy
```

#### Step 7: Verify Restore Completeness

**Test 1: Health Check**
```bash
curl -s https://cap-restore.example.com/healthz
```

**Expected:**
```json
{"status": "OK", "version": "0.11.0", "build_hash": null}
```

**Test 2: Readiness Check**
```bash
curl -s -H "Authorization: Bearer $RESTORE_TOKEN" \
  https://cap-restore.example.com/readyz
```

**Expected:**
```json
{
  "status": "OK",
  "checks": [
    {"name": "verifier_core", "status": "OK"},
    {"name": "crypto", "status": "OK"},
    {"name": "registry", "status": "OK"},
    {"name": "policy_store", "status": "OK"}
  ]
}
```

**Test 3: Policy Hash Verification (Critical!)**
```bash
# Get policy from restored system
POLICY_HASH="0x0afcb402e74ff6a11601863fc4ae2f2d756124db71bc703f5889ecefcd371ff4"
curl -s -H "Authorization: Bearer $RESTORE_TOKEN" \
  https://cap-restore.example.com/api/v1/policy/$POLICY_HASH \
  | jq '.policy_hash'
```

**Expected:** Identical policy_hash as original
```
"0x0afcb402e74ff6a11601863fc4ae2f2d756124db71bc703f5889ecefcd371ff4"
```

**Test 4: ETag Verification**
```bash
curl -sI -H "Authorization: Bearer $RESTORE_TOKEN" \
  https://cap-restore.example.com/api/v1/policy/$POLICY_HASH \
  | grep -i etag
```

**Expected:** Identical ETag as original
```
etag: "0afcb402e74ff6a1"
```

**Test 5: Registry Entry Verification**
```bash
# Query registry for known entry
MANIFEST_HASH="0xd490be94abc123def456789012345678901234567890123456789012345678901234"
kubectl -n cap-restore exec $POD -- \
  sqlite3 /app/build/registry.sqlite \
  "SELECT manifest_hash, proof_hash, kid FROM registry_entries WHERE manifest_hash='$MANIFEST_HASH';"
```

**Expected:** Entry exists with identical hashes and KID

#### Step 8: Smoke Tests

```bash
# Run smoke test suite
./scripts/smoke_restore.sh https://cap-restore.example.com/api/v1
```

**Expected Output:**
```
🧪 Running smoke tests on restored system...
✅ Health check: OK
✅ Readiness check: OK
✅ Policy retrieval: OK (hash matches)
✅ Verify endpoint: OK
✅ ETag matches original: OK
✅ Registry query: OK (25 entries)
🎉 All smoke tests passed!
```

### Rollback Procedure

If restore fails or produces incorrect hashes:

1. **Stop Restored System**
   ```bash
   kubectl -n cap-restore scale deploy/cap-verifier-api --replicas=0
   ```

2. **Analyze Logs**
   ```bash
   kubectl -n cap-restore logs deploy/cap-verifier-api --tail=1000
   ```

3. **Compare Hashes** (Manual Verification)
   ```bash
   # Original system
   curl -s https://cap-prod.example.com/api/v1/policy/$POLICY_HASH | jq '.policy_hash'

   # Restored system
   curl -s https://cap-restore.example.com/api/v1/policy/$POLICY_HASH | jq '.policy_hash'
   ```

4. **Delete Restore Namespace**
   ```bash
   kubectl delete namespace cap-restore
   ```

5. **Escalate to Engineering**
   - Contact: cap-engineering@example.com
   - PagerDuty: Restore Failure Alert
   - Slack: #cap-incidents

## Validation Checklist

After restore, verify the following (DoD):

### Functional Checks
- [ ] All pods running (`kubectl -n cap-restore get pods`)
- [ ] Health check returns 200 (`/healthz`)
- [ ] Readiness check returns 200 (`/readyz`)
- [ ] Registry accessible (SQLite query succeeds)
- [ ] Policy store loaded (25+ policies)

### Hash Integrity Checks (Critical!)
- [ ] `policy_hash` matches original (SHA3-256)
- [ ] `ir_hash` matches original (SHA3-256)
- [ ] `manifest_hash` matches original (SHA3-256)
- [ ] `proof_hash` matches original (SHA3-256)
- [ ] ETag matches original (first 16 chars of policy_hash)

### Signature Checks
- [ ] Registry entries have valid KIDs
- [ ] Signatures verify with restored public keys
- [ ] Private keys retrieved from KMS (not in backup)

### API Functionality Checks
- [ ] `/policy/compile` works (create new policy)
- [ ] `/policy/:id` works (retrieve policy)
- [ ] `/verify` works (verify compliance)
- [ ] Authentication works (OAuth2 JWT)

### Compliance Checks
- [ ] No PII in backup archive (verify with grep)
- [ ] Audit logs not included (ephemeral)
- [ ] Secrets not in backup (verify with strings)

## Disaster Recovery Scenarios

### Scenario 1: Complete Cluster Loss

**Impact:** All data in Kubernetes cluster lost
**Recovery:** Restore from S3 backup to new cluster

**Steps:**
1. Provision new Kubernetes cluster
2. Download latest backup from S3
3. Follow restore procedure (Steps 1-8)
4. Update DNS to point to new cluster
5. Validate with smoke tests

**RTO:** 30-45 minutes
**RPO:** < 1 hour (last backup)

### Scenario 2: Corrupted Registry

**Impact:** Registry data corrupted, hashes don't match
**Recovery:** Restore registry.sqlite only

**Steps:**
1. Stop API pods (`kubectl scale deploy --replicas=0`)
2. Download latest backup
3. Extract registry.sqlite only
4. Copy to pod (`kubectl cp`)
5. Restart API pods (`kubectl scale deploy --replicas=3`)
6. Verify hashes match

**RTO:** 5-10 minutes
**RPO:** < 1 hour

### Scenario 3: Lost Policy Store

**Impact:** In-memory policy store cleared (pod restart)
**Recovery:** Restore from backup or rebuild from registry

**Steps:**
1. Option A: Restore policy_store.json from backup
2. Option B: Rebuild from registry entries (slower)
3. Verify with `/policy/:id` calls
4. Check ETags match

**RTO:** 2-5 minutes
**RPO:** < 1 hour

## Monitoring & Alerting

### Backup Monitoring

**Prometheus Metrics:**
```promql
# Backup success/failure
cap_backup_attempts_total{status="success|failure"}

# Backup duration
cap_backup_duration_seconds

# Backup size
cap_backup_size_bytes

# Last successful backup timestamp
cap_backup_last_success_timestamp
```

**Alerts:**
```yaml
- alert: BackupFailed
  expr: rate(cap_backup_attempts_total{status="failure"}[1h]) > 0
  for: 5m
  labels:
    severity: critical
  annotations:
    summary: "CAP Backup failed"

- alert: BackupStale
  expr: time() - cap_backup_last_success_timestamp > 7200
  for: 10m
  labels:
    severity: warning
  annotations:
    summary: "CAP Backup stale (no successful backup in 2h)"
```

### Restore Monitoring

**Grafana Dashboard:**
- Restore Duration (histogram)
- Hash Verification Success Rate (%)
- Restored Registry Entry Count
- Smoke Test Pass Rate (%)

## Security Considerations

### Backup Security
- ✅ Encrypt backups at rest (AES-256-GCM)
- ✅ Encrypt backups in transit (TLS 1.3)
- ✅ No private keys in backups
- ✅ No secrets in backups
- ✅ RBAC: Only backup service account can read
- ✅ Audit log: All backup operations logged

### Restore Security
- ✅ Verify backup integrity (SHA3-256 hashes)
- ✅ Verify backup signature (signed by backup service)
- ✅ Retrieve private keys from KMS (not backup)
- ✅ Temporary namespace (isolated from prod)
- ✅ Audit log: All restore operations logged

### Access Control
- **Backup Creation:** `cap-backup-sa` service account
- **Backup Download:** SRE team (PagerDuty escalation)
- **Restore Execution:** SRE + Engineering Manager approval
- **KMS Access:** Multi-party approval (2 of 3 keys)

## Troubleshooting

### Issue 1: Hash Mismatch After Restore

**Symptom:** `policy_hash` or `ir_hash` differs from original

**Possible Causes:**
1. Backup corrupted during storage
2. Registry entries modified during backup
3. Incorrect restore order (policy store before registry)

**Solution:**
```bash
# Verify backup integrity
jq '.files[] | select(.path=="registry.sqlite") | .sha3_256' backup.manifest.json
sha3sum /restore/registry.sqlite

# If mismatch, download backup again from S3
aws s3 cp s3://cap-backups/latest/registry.sqlite /restore/

# Restore registry first, then policy store
./scripts/restore.sh --registry-first
```

### Issue 2: ETag Mismatch

**Symptom:** ETag header differs from original

**Cause:** ETag is computed from policy content, not just hash

**Solution:**
```bash
# Retrieve full policy and recompute ETag
curl -s https://cap-restore.example.com/api/v1/policy/$POLICY_HASH | \
  jq -r '.policy | tostring' | \
  sha3sum | \
  cut -c1-16
```

### Issue 3: Missing Registry Entries

**Symptom:** Registry query returns 0 results

**Cause:** Registry not loaded or SQLite file corrupted

**Solution:**
```bash
# Check file exists
kubectl -n cap-restore exec $POD -- ls -lh /app/build/registry.sqlite

# Check file readable
kubectl -n cap-restore exec $POD -- sqlite3 /app/build/registry.sqlite ".tables"

# If corrupted, restore from backup again
kubectl -n cap-restore cp /restore/registry.sqlite $POD:/app/build/registry.sqlite

# Restart pod to reload
kubectl -n cap-restore delete pod -l app=cap-verifier-api
```

## Testing Strategy

### Unit Tests
- `tests/backup_restore.rs::test_backup_manifest_generation`
- `tests/backup_restore.rs::test_restore_hash_integrity`
- `tests/backup_restore.rs::test_no_pii_in_backup`

### Integration Tests
- `tests/backup_restore.rs::test_full_backup_restore_cycle` (ignored, requires cluster)
- `tests/backup_restore.rs::test_restored_ir_hash_matches` (ignored)
- `tests/backup_restore.rs::test_smoke_ready_after_restore` (ignored)

### Manual Tests
- DR drill (quarterly): Full restore to new cluster
- Hash verification: Compare all critical hashes
- Performance: RTO < 30 minutes

## References

- [Kubernetes Backup Best Practices](https://kubernetes.io/docs/tasks/administer-cluster/configure-upgrade-etcd/#backing-up-an-etcd-cluster)
- [Velero Backup Tool](https://velero.io/)
- [PostgreSQL Backup & Recovery](https://www.postgresql.org/docs/current/backup.html)
- [AWS Backup for EBS](https://docs.aws.amazon.com/aws-backup/latest/devguide/whatisbackup.html)

## Appendix A: Backup Script Usage

```bash
./scripts/backup.sh --help

Usage: backup.sh [OPTIONS]

Options:
  --output PATH          Output tar.gz file path (required)
  --registry PATH        Path to registry.sqlite or registry.json (required)
  --policy-store PATH    Path to policy_store.json (required)
  --keys PATH            Path to keys directory (public keys only)
  --config PATH          Path to helm/values-prod.yaml
  --manifest PATH        Output path for backup.manifest.json
  --compress             Enable gzip compression (default: true)
  --encrypt              Enable AES-256-GCM encryption (default: false)
  --encryption-key PATH  Path to encryption key (if --encrypt enabled)
  --help                 Show this help message

Examples:
  # Basic backup
  ./scripts/backup.sh \
    --output /backup/cap-backup.tar.gz \
    --registry build/registry.sqlite \
    --policy-store build/policy_store.json

  # Encrypted backup
  ./scripts/backup.sh \
    --output /backup/cap-backup-encrypted.tar.gz \
    --registry build/registry.sqlite \
    --policy-store build/policy_store.json \
    --encrypt \
    --encryption-key /keys/backup.key
```

## Appendix B: Restore Script Usage

```bash
./scripts/restore.sh --help

Usage: restore.sh [OPTIONS]

Options:
  --backup-dir PATH      Path to extracted backup directory (required)
  --manifest PATH        Path to backup.manifest.json (required)
  --verify-only          Only verify backup integrity, don't restore
  --target-namespace NS  Kubernetes namespace for restore (default: cap-restore)
  --registry-first       Restore registry before policy store (default: true)
  --skip-smoke           Skip smoke tests after restore (default: false)
  --help                 Show this help message

Examples:
  # Verify backup only
  ./scripts/restore.sh \
    --backup-dir /restore \
    --manifest /restore/backup.manifest.json \
    --verify-only

  # Full restore
  ./scripts/restore.sh \
    --backup-dir /restore \
    --manifest /restore/backup.manifest.json \
    --target-namespace cap-restore
```

---

**Document Version:** 1.0.0
**Last Updated:** 2025-11-10
**Author:** CAP Engineering Team
**Reviewed By:** SRE Team, Security Team
**Next Review:** 2025-12-10 (Monthly)
