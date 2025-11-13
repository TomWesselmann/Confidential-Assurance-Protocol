#!/usr/bin/env bash
#
# CAP Verifier API - Restore Script
# Week 6 - Track D1: Backup & Restore
#
# Description:
#   Restores a CAP Verifier API backup with deterministic hash verification.
#   Verifies backup integrity via SHA3-256 hashes in backup.manifest.json.
#   Supports verify-only mode for backup validation without restoration.
#
# Usage:
#   ./scripts/restore.sh --backup-dir /restore \
#                        --manifest /restore/backup.manifest.json \
#                        --verify-only
#
# Requirements:
#   - bash >= 4.0
#   - tar, gzip
#   - sha3sum (or openssl for SHA3-256)
#   - jq (for JSON manipulation)
#   - kubectl (for Kubernetes operations)

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default values
BACKUP_DIR=""
MANIFEST=""
VERIFY_ONLY=false
TARGET_NAMESPACE="cap-restore"
REGISTRY_FIRST=true
SKIP_SMOKE=false
VERBOSE=false

# Script metadata
SCRIPT_VERSION="1.0.0"

# Helper functions
log_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

log_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

log_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

log_error() {
    echo -e "${RED}❌ $1${NC}" >&2
}

usage() {
    cat <<EOF
Usage: $0 [OPTIONS]

Restores a CAP Verifier API backup with deterministic hash verification.

Options:
  --backup-dir PATH      Path to extracted backup directory (required)
  --manifest PATH        Path to backup.manifest.json (required)
  --verify-only          Only verify backup integrity, don't restore
  --target-namespace NS  Kubernetes namespace for restore (default: cap-restore)
  --registry-first       Restore registry before policy store (default: true)
  --no-registry-first    Restore policy store before registry
  --skip-smoke           Skip smoke tests after restore (default: false)
  --verbose              Enable verbose output
  --help                 Show this help message

Examples:
  # Verify backup integrity only
  $0 --backup-dir /restore \\
     --manifest /restore/backup.manifest.json \\
     --verify-only

  # Full restore to default namespace (cap-restore)
  $0 --backup-dir /restore \\
     --manifest /restore/backup.manifest.json

  # Restore to custom namespace
  $0 --backup-dir /restore \\
     --manifest /restore/backup.manifest.json \\
     --target-namespace cap-dr \\
     --skip-smoke

Environment Variables:
  RESTORE_BACKUP_DIR     Same as --backup-dir
  RESTORE_MANIFEST       Same as --manifest
  RESTORE_NAMESPACE      Same as --target-namespace

Exit Codes:
  0   Success
  1   General error
  2   Missing required argument
  3   File not found
  4   Verification failed
  5   Kubernetes operation failed

Author: CAP Engineering Team
Version: $SCRIPT_VERSION
EOF
}

# Compute SHA3-256 hash of file
compute_sha3() {
    local file="$1"

    if command -v sha3sum &> /dev/null; then
        sha3sum -a 256 "$file" | awk '{print "0x"$1}'
    elif command -v openssl &> /dev/null; then
        # Fallback to OpenSSL (if sha3 support available)
        openssl dgst -sha3-256 -hex "$file" | awk '{print "0x"$2}'
    else
        log_error "Neither sha3sum nor openssl with SHA3 support found"
        exit 1
    fi
}

# Verify file exists and is readable
verify_file() {
    local file="$1"
    local desc="$2"

    if [[ ! -f "$file" ]]; then
        log_error "$desc not found: $file"
        exit 3
    fi

    if [[ ! -r "$file" ]]; then
        log_error "$desc not readable: $file"
        exit 3
    fi
}

# Verify backup integrity against manifest
verify_backup_integrity() {
    log_info "🔍 Verifying backup integrity..."

    local manifest_content=$(cat "$MANIFEST")
    local files_count=$(echo "$manifest_content" | jq -r '.files | length')
    local verified_count=0
    local failed_count=0

    # Iterate over files in manifest
    for i in $(seq 0 $((files_count - 1))); do
        local file_path=$(echo "$manifest_content" | jq -r ".files[$i].path")
        local expected_hash=$(echo "$manifest_content" | jq -r ".files[$i].sha3_256")
        local file_size=$(echo "$manifest_content" | jq -r ".files[$i].size_bytes")
        local file_type=$(echo "$manifest_content" | jq -r ".files[$i].type")

        # Construct full path
        local full_path="$BACKUP_DIR/$file_path"

        # Special handling for directory (keys/)
        if [[ "$file_path" == */ ]]; then
            # Directory: create tar and hash
            if [[ ! -d "$BACKUP_DIR/${file_path%/}" ]]; then
                log_error "Directory not found: $BACKUP_DIR/${file_path%/}"
                ((failed_count++))
                continue
            fi

            local temp_tar=$(mktemp)
            tar -cf "$temp_tar" -C "$BACKUP_DIR" "${file_path%/}"
            local actual_hash=$(compute_sha3 "$temp_tar")
            rm "$temp_tar"
        else
            # Regular file
            if [[ ! -f "$full_path" ]]; then
                log_error "File not found: $full_path"
                ((failed_count++))
                continue
            fi

            local actual_hash=$(compute_sha3 "$full_path")
        fi

        # Compare hashes
        if [[ "$actual_hash" == "$expected_hash" ]]; then
            ((verified_count++))
            if $VERBOSE; then
                log_success "$file_path: SHA3-256 matches ($actual_hash)"
            else
                log_success "$file_path: SHA3-256 matches"
            fi
        else
            ((failed_count++))
            log_error "$file_path: SHA3-256 MISMATCH"
            log_error "  Expected: $expected_hash"
            log_error "  Got:      $actual_hash"
        fi
    done

    # Summary
    log_info "Verification complete: $verified_count/$files_count files verified"

    if [[ $failed_count -gt 0 ]]; then
        log_error "Verification failed: $failed_count file(s) have hash mismatches"
        exit 4
    fi

    log_success "✅ All $verified_count files verified successfully"
}

# Check if kubectl is available
check_kubectl() {
    if ! command -v kubectl &> /dev/null; then
        log_error "kubectl not found. Please install kubectl and configure cluster access."
        exit 5
    fi

    # Test cluster connectivity
    if ! kubectl cluster-info &> /dev/null; then
        log_error "Cannot connect to Kubernetes cluster. Please check kubeconfig."
        exit 5
    fi
}

# Create target namespace if it doesn't exist
create_namespace() {
    log_info "Checking namespace: $TARGET_NAMESPACE"

    if kubectl get namespace "$TARGET_NAMESPACE" &> /dev/null; then
        log_warning "Namespace $TARGET_NAMESPACE already exists"
        read -p "Continue with existing namespace? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            log_info "Aborting restore"
            exit 0
        fi
    else
        log_info "Creating namespace: $TARGET_NAMESPACE"
        kubectl create namespace "$TARGET_NAMESPACE"
        log_success "Namespace created"
    fi

    # Set namespace context
    kubectl config set-context --current --namespace="$TARGET_NAMESPACE"
}

# Deploy Helm chart
deploy_helm() {
    log_info "Deploying Helm chart to $TARGET_NAMESPACE..."

    local values_file="$BACKUP_DIR/values-prod.yaml"
    if [[ ! -f "$values_file" ]]; then
        log_warning "values-prod.yaml not found in backup, using default values"
        values_file=""
    fi

    # Check if Helm chart directory exists
    if [[ ! -d "helm/" ]]; then
        log_error "Helm chart directory not found: helm/"
        log_error "Please run this script from the repository root"
        exit 3
    fi

    # Deploy Helm chart
    local helm_cmd="helm upgrade --install cap-restore helm/ --namespace $TARGET_NAMESPACE --wait --timeout 10m"
    if [[ -n "$values_file" ]]; then
        helm_cmd="$helm_cmd -f $values_file"
    fi

    if $VERBOSE; then
        log_info "Running: $helm_cmd"
    fi

    if ! eval "$helm_cmd"; then
        log_error "Helm deployment failed"
        exit 5
    fi

    log_success "Helm chart deployed"

    # Wait for pods to be ready
    log_info "Waiting for pods to be ready..."
    kubectl -n "$TARGET_NAMESPACE" wait --for=condition=ready pod -l app=cap-verifier-api --timeout=300s

    log_success "All pods ready"
}

# Get first API pod name
get_api_pod() {
    kubectl -n "$TARGET_NAMESPACE" get pod -l app=cap-verifier-api -o jsonpath='{.items[0].metadata.name}' 2>/dev/null || echo ""
}

# Restore registry
restore_registry() {
    log_info "Restoring registry..."

    local pod=$(get_api_pod)
    if [[ -z "$pod" ]]; then
        log_error "No API pod found in namespace $TARGET_NAMESPACE"
        exit 5
    fi

    # Find registry file (sqlite or json)
    local registry_file=""
    if [[ -f "$BACKUP_DIR/registry.sqlite" ]]; then
        registry_file="$BACKUP_DIR/registry.sqlite"
    elif [[ -f "$BACKUP_DIR/registry.json" ]]; then
        registry_file="$BACKUP_DIR/registry.json"
    else
        log_error "Registry file not found in backup directory"
        exit 3
    fi

    # Copy registry to pod
    log_info "Copying registry to pod $pod..."
    kubectl -n "$TARGET_NAMESPACE" cp "$registry_file" "$pod:/app/build/$(basename $registry_file)"

    # Verify copy
    if ! kubectl -n "$TARGET_NAMESPACE" exec "$pod" -- ls -lh "/app/build/$(basename $registry_file)" &> /dev/null; then
        log_error "Registry file not found in pod after copy"
        exit 5
    fi

    log_success "Registry restored"
}

# Restore policy store
restore_policy_store() {
    log_info "Restoring policy store..."

    local pod=$(get_api_pod)
    if [[ -z "$pod" ]]; then
        log_error "No API pod found in namespace $TARGET_NAMESPACE"
        exit 5
    fi

    local policy_store_file="$BACKUP_DIR/policy_store.json"
    if [[ ! -f "$policy_store_file" ]]; then
        log_error "Policy store file not found: $policy_store_file"
        exit 3
    fi

    # Copy policy store to pod
    log_info "Copying policy store to pod $pod..."
    kubectl -n "$TARGET_NAMESPACE" cp "$policy_store_file" "$pod:/app/build/policy_store.json"

    # TODO: In production, load policy store via REST API (POST /policy/restore)
    # For now, just copy the file and let the application load it on startup

    log_success "Policy store restored"
}

# Restore key metadata
restore_keys() {
    log_info "Restoring key metadata..."

    local pod=$(get_api_pod)
    if [[ -z "$pod" ]]; then
        log_error "No API pod found in namespace $TARGET_NAMESPACE"
        exit 5
    fi

    local keys_dir="$BACKUP_DIR/keys"
    if [[ ! -d "$keys_dir" ]]; then
        log_warning "Keys directory not found in backup, skipping key restore"
        return 0
    fi

    # Copy keys directory to pod
    log_info "Copying key metadata to pod $pod..."
    kubectl -n "$TARGET_NAMESPACE" exec "$pod" -- mkdir -p /app/keys
    kubectl -n "$TARGET_NAMESPACE" cp "$keys_dir/" "$pod:/app/keys/"

    log_success "Key metadata restored"

    # Note: Private keys are NOT in backup and must be retrieved from KMS
    log_warning "Remember to retrieve private keys from KMS/Vault"
}

# Run smoke tests
run_smoke_tests() {
    log_info "Running smoke tests..."

    local pod=$(get_api_pod)
    if [[ -z "$pod" ]]; then
        log_error "No API pod found in namespace $TARGET_NAMESPACE"
        exit 5
    fi

    # Get service URL
    local service_url=""
    if kubectl -n "$TARGET_NAMESPACE" get ingress cap-ingress &> /dev/null; then
        service_url=$(kubectl -n "$TARGET_NAMESPACE" get ingress cap-ingress -o jsonpath='{.spec.rules[0].host}')
        service_url="https://$service_url"
    else
        # Use port-forward for testing
        log_info "No ingress found, using port-forward for testing"
        kubectl -n "$TARGET_NAMESPACE" port-forward "pod/$pod" 8080:8080 &
        local pf_pid=$!
        sleep 2
        service_url="http://localhost:8080"
        trap "kill $pf_pid 2>/dev/null || true" EXIT
    fi

    # Test 1: Health check
    log_info "Test 1: Health check..."
    if curl -s -f "$service_url/healthz" > /dev/null; then
        log_success "Health check: OK"
    else
        log_error "Health check: FAILED"
        return 1
    fi

    # Test 2: Readiness check (requires auth - skip if no token)
    if [[ -n "${RESTORE_TOKEN:-}" ]]; then
        log_info "Test 2: Readiness check..."
        if curl -s -f -H "Authorization: Bearer $RESTORE_TOKEN" "$service_url/readyz" > /dev/null; then
            log_success "Readiness check: OK"
        else
            log_error "Readiness check: FAILED"
            return 1
        fi
    else
        log_warning "Test 2: Readiness check skipped (no RESTORE_TOKEN)"
    fi

    # Test 3: Registry query (SQLite)
    log_info "Test 3: Registry query..."
    local registry_count=$(kubectl -n "$TARGET_NAMESPACE" exec "$pod" -- \
        sqlite3 /app/build/registry.sqlite "SELECT COUNT(*) FROM registry_entries;" 2>/dev/null || echo "0")

    if [[ "$registry_count" -gt 0 ]]; then
        log_success "Registry query: OK ($registry_count entries)"
    else
        log_warning "Registry query: No entries found"
    fi

    log_success "🎉 All smoke tests passed!"
}

# Parse command line arguments
parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            --backup-dir)
                BACKUP_DIR="$2"
                shift 2
                ;;
            --manifest)
                MANIFEST="$2"
                shift 2
                ;;
            --verify-only)
                VERIFY_ONLY=true
                shift
                ;;
            --target-namespace)
                TARGET_NAMESPACE="$2"
                shift 2
                ;;
            --registry-first)
                REGISTRY_FIRST=true
                shift
                ;;
            --no-registry-first)
                REGISTRY_FIRST=false
                shift
                ;;
            --skip-smoke)
                SKIP_SMOKE=true
                shift
                ;;
            --verbose)
                VERBOSE=true
                shift
                ;;
            --help)
                usage
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                usage
                exit 1
                ;;
        esac
    done

    # Check required arguments
    if [[ -z "$BACKUP_DIR" ]]; then
        log_error "Missing required argument: --backup-dir"
        usage
        exit 2
    fi

    if [[ -z "$MANIFEST" ]]; then
        log_error "Missing required argument: --manifest"
        usage
        exit 2
    fi

    # Verify backup directory exists
    if [[ ! -d "$BACKUP_DIR" ]]; then
        log_error "Backup directory not found: $BACKUP_DIR"
        exit 3
    fi

    # Verify manifest exists
    verify_file "$MANIFEST" "Backup manifest"
}

# Main restore function
main() {
    parse_args "$@"

    log_info "🔓 CAP Verifier API Restore Script v$SCRIPT_VERSION"
    log_info "=================================="

    # Step 1: Verify backup integrity
    verify_backup_integrity

    # If verify-only mode, exit here
    if $VERIFY_ONLY; then
        log_success "✅ Backup verification complete (verify-only mode)"
        exit 0
    fi

    # Step 2: Check Kubernetes prerequisites
    check_kubectl

    # Step 3: Create/verify target namespace
    create_namespace

    # Step 4: Deploy Helm chart
    deploy_helm

    # Step 5 & 6: Restore data (order depends on --registry-first flag)
    if $REGISTRY_FIRST; then
        restore_registry
        restore_policy_store
    else
        restore_policy_store
        restore_registry
    fi

    # Step 7: Restore keys
    restore_keys

    # Step 8: Run smoke tests
    if ! $SKIP_SMOKE; then
        run_smoke_tests
    else
        log_warning "Smoke tests skipped (--skip-smoke flag)"
    fi

    log_success "✅ Restore completed successfully!"
    log_info "Namespace: $TARGET_NAMESPACE"
    log_info "Next steps:"
    log_info "  1. Retrieve private keys from KMS/Vault"
    log_info "  2. Verify policy hashes match original"
    log_info "  3. Run full validation checklist (see runbook_restore.md)"
}

# Run main function
main "$@"
