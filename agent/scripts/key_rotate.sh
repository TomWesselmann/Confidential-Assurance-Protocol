#!/usr/bin/env bash
#
# CAP Verifier API - Key Rotation Script
# Week 6 - Track D2: Key Rotation
#
# Description:
#   Automates zero-downtime Ed25519 key rotation using KID-based dual-accept phases.
#   Implements 4-phase rotation: Preparation → Dual-Accept → Sign-Switch → Decommission
#
# Usage:
#   ./scripts/key_rotate.sh --phase <0|1|2|3> \
#                           --old-key keys/old.v1.json \
#                           --new-key keys/new.v1.json \
#                           --namespace cap-production
#
# Requirements:
#   - bash >= 4.0
#   - kubectl (for Kubernetes operations)
#   - jq (for JSON manipulation)
#   - cargo (for KID derivation via cap-agent CLI)

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default values
PHASE=""
OLD_KEY=""
NEW_KEY=""
NAMESPACE="cap-production"
DUAL_ACCEPT_DURATION="168h"  # 7 days
DRY_RUN=false
FORCE=false
ROLLBACK=false
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

Automates zero-downtime Ed25519 key rotation using KID-based dual-accept phases.

Options:
  --phase <0|1|2|3>      Rotation phase (required)
                         0 = Preparation (generate + attest)
                         1 = Dual-Accept (accept both keys)
                         2 = Sign-Switch (sign with new, accept both)
                         3 = Decommission (retire old key)
  --old-key PATH         Path to old key metadata (required)
  --new-key PATH         Path to new key metadata (required for phase 0-2)
  --namespace NS         Kubernetes namespace (default: cap-production)
  --duration DURATION    Dual-accept duration (default: 168h = 7 days)
  --dry-run              Preview changes without applying
  --force                Skip confirmation prompts
  --rollback             Rollback current phase (phase-specific)
  --verbose              Enable verbose output
  --help                 Show this help message

Phases:
  Phase 0: Preparation
    - Generate new key (if not exists)
    - Attest new key with old key (chain of trust)
    - Store new key in KMS/Vault
    - Update Kubernetes secret with dual-key config

  Phase 1: Dual-Accept (T1 Start)
    - Activate dual-accept mode
    - Both old and new keys accepted for verification
    - Still signing with old key
    - Set dual_accept_until timestamp

  Phase 2: Sign-Switch
    - Switch default signing key to new key
    - Still accepting both keys for verification
    - Monitor signature distribution

  Phase 3: Decommission (T1 End)
    - Retire old key (status: retired)
    - Only new key accepted
    - Archive old key

Rollback:
  --rollback --phase 2  → Revert to Phase 1 (sign with old, accept both)
  --rollback --phase 3  → Revert to Phase 2 (accept both, extend T1)

Examples:
  # Phase 0: Generate and attest new key
  $0 --phase 0 \\
     --old-key keys/company.v1.json \\
     --new-key keys/company.v2.json

  # Phase 1: Activate dual-accept mode
  $0 --phase 1 \\
     --old-key keys/company.v1.json \\
     --new-key keys/company.v2.json \\
     --namespace cap-production \\
     --duration 168h

  # Phase 2: Switch to signing with new key
  $0 --phase 2 \\
     --old-key keys/company.v1.json \\
     --new-key keys/company.v2.json \\
     --namespace cap-production

  # Phase 3: Decommission old key
  $0 --phase 3 \\
     --old-key keys/company.v1.json \\
     --namespace cap-production

  # Rollback Phase 2 to Phase 1
  $0 --rollback --phase 2 \\
     --old-key keys/company.v1.json \\
     --namespace cap-production

Environment Variables:
  ROTATE_OLD_KEY         Same as --old-key
  ROTATE_NEW_KEY         Same as --new-key
  ROTATE_NAMESPACE       Same as --namespace

Exit Codes:
  0   Success
  1   General error
  2   Missing required argument
  3   File not found
  4   Validation failed
  5   Kubernetes operation failed
  6   KID derivation failed

Author: CAP Engineering Team
Version: $SCRIPT_VERSION
EOF
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

# Check if jq is available
check_jq() {
    if ! command -v jq &> /dev/null; then
        log_error "jq not found. Please install jq for JSON manipulation."
        exit 1
    fi
}

# Check if cargo is available (for KID derivation)
check_cargo() {
    if ! command -v cargo &> /dev/null; then
        log_error "cargo not found. Please install Rust toolchain."
        exit 1
    fi
}

# Derive KID from key metadata using cap-agent CLI
derive_kid() {
    local key_file="$1"

    if [[ ! -f "$key_file" ]]; then
        log_error "Key metadata file not found: $key_file"
        exit 6
    fi

    # Extract KID from key metadata JSON
    local kid=$(jq -r '.kid' "$key_file" 2>/dev/null || echo "")

    if [[ -z "$kid" || "$kid" == "null" ]]; then
        log_error "KID not found in key metadata: $key_file"
        exit 6
    fi

    echo "$kid"
}

# Phase 0: Preparation (generate new key, attest)
phase_0_preparation() {
    log_info "🔧 Phase 0: Preparation"
    log_info "======================="

    # Check if new key already exists
    if [[ -f "$NEW_KEY" ]]; then
        log_warning "New key metadata already exists: $NEW_KEY"
        read -p "Regenerate key? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            log_info "Skipping key generation"
        else
            # Generate new key
            log_info "Generating new key: $NEW_KEY"
            if $DRY_RUN; then
                log_info "DRY RUN: Would generate new key"
            else
                cargo run -- keys keygen \
                    --owner "$(jq -r '.owner' $OLD_KEY)" \
                    --out "$NEW_KEY" \
                    --valid-days 730
                log_success "New key generated"
            fi
        fi
    else
        # Generate new key
        log_info "Generating new key: $NEW_KEY"
        if $DRY_RUN; then
            log_info "DRY RUN: Would generate new key"
        else
            cargo run -- keys keygen \
                --owner "$(jq -r '.owner' $OLD_KEY)" \
                --out "$NEW_KEY" \
                --valid-days 730
            log_success "New key generated"
        fi
    fi

    # Derive KIDs
    local old_kid=$(derive_kid "$OLD_KEY")
    local new_kid=$(derive_kid "$NEW_KEY")

    log_info "Old KID: $old_kid"
    log_info "New KID: $new_kid"

    # Attest new key with old key
    local attestation_file="${NEW_KEY%.json}.attestation.json"
    log_info "Attesting new key with old key: $attestation_file"

    if $DRY_RUN; then
        log_info "DRY RUN: Would create attestation"
    else
        cargo run -- keys attest \
            --signer "$OLD_KEY" \
            --subject "$NEW_KEY" \
            --out "$attestation_file"
        log_success "Attestation created: $attestation_file"
    fi

    # Update Kubernetes secret with dual-key config
    log_info "Updating Kubernetes secret: cap-signing-keys"

    if $DRY_RUN; then
        log_info "DRY RUN: Would update Kubernetes secret"
    else
        # Read private keys
        local old_privkey=$(cat "${OLD_KEY%.json}.ed25519" | base64)
        local new_privkey=$(cat "${NEW_KEY%.json}.ed25519" | base64)

        # Create or update secret
        kubectl -n "$NAMESPACE" create secret generic cap-signing-keys \
            --from-literal=old_key="$old_privkey" \
            --from-literal=new_key="$new_privkey" \
            --from-literal=old_kid="$old_kid" \
            --from-literal=new_kid="$new_kid" \
            --dry-run=client -o yaml | kubectl -n "$NAMESPACE" apply -f -

        log_success "Kubernetes secret updated"
    fi

    log_success "✅ Phase 0 complete: Keys prepared and attested"
    log_info "Next step: Run Phase 1 to activate dual-accept mode"
}

# Phase 1: Dual-Accept (activate dual-accept mode)
phase_1_dual_accept() {
    log_info "🔓 Phase 1: Dual-Accept (T1 Start)"
    log_info "=================================="

    # Derive KIDs
    local old_kid=$(derive_kid "$OLD_KEY")
    local new_kid=$(derive_kid "$NEW_KEY")

    log_info "Old KID: $old_kid"
    log_info "New KID: $new_kid"

    # Calculate dual_accept_until timestamp
    local dual_accept_until=$(date -u -v "+${DUAL_ACCEPT_DURATION}" +"%Y-%m-%dT%H:%M:%SZ" 2>/dev/null || \
                              date -u -d "+${DUAL_ACCEPT_DURATION}" +"%Y-%m-%dT%H:%M:%SZ")

    log_info "Dual-accept mode will expire: $dual_accept_until"

    if ! $FORCE; then
        read -p "Activate dual-accept mode? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            log_info "Aborting"
            exit 0
        fi
    fi

    # Update Helm values with dual-accept config
    log_info "Updating Helm values: helm/values-prod.yaml"

    if $DRY_RUN; then
        log_info "DRY RUN: Would update Helm values"
        cat <<EOF
verifier:
  signing:
    mode: dual-accept
    keys:
      - kid: "$old_kid"
        path: /keys/old
        status: active
      - kid: "$new_kid"
        path: /keys/new
        status: active
    default_key: /keys/old
    dual_accept_until: "$dual_accept_until"
EOF
    else
        # Create temporary YAML patch
        local temp_patch=$(mktemp)
        cat > "$temp_patch" <<EOF
verifier:
  signing:
    mode: dual-accept
    keys:
      - kid: "$old_kid"
        path: /keys/old
        status: active
      - kid: "$new_kid"
        path: /keys/new
        status: active
    default_key: /keys/old
    dual_accept_until: "$dual_accept_until"
EOF

        # Apply Helm upgrade
        log_info "Applying Helm upgrade..."
        helm upgrade --install cap-verifier-api helm/ \
            --namespace "$NAMESPACE" \
            --values helm/values-prod.yaml \
            --values "$temp_patch" \
            --wait --timeout 10m

        rm "$temp_patch"
        log_success "Helm upgrade complete"
    fi

    # Verify pods are ready
    log_info "Waiting for pods to be ready..."
    if ! $DRY_RUN; then
        kubectl -n "$NAMESPACE" wait --for=condition=ready pod -l app=cap-verifier-api --timeout=300s
        log_success "All pods ready"
    fi

    log_success "✅ Phase 1 complete: Dual-accept mode activated"
    log_info "Next step: Monitor metrics, then run Phase 2 to switch signing key"
    log_info "Dual-accept expires: $dual_accept_until"
}

# Phase 2: Sign-Switch (switch to signing with new key)
phase_2_sign_switch() {
    log_info "🔄 Phase 2: Sign-Switch"
    log_info "======================"

    # Derive KIDs
    local old_kid=$(derive_kid "$OLD_KEY")
    local new_kid=$(derive_kid "$NEW_KEY")

    log_info "Old KID: $old_kid"
    log_info "New KID: $new_kid"

    if ! $FORCE; then
        read -p "Switch to signing with new key? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            log_info "Aborting"
            exit 0
        fi
    fi

    # Update Helm values to switch default_key
    log_info "Updating Helm values: switch default_key to /keys/new"

    if $DRY_RUN; then
        log_info "DRY RUN: Would update default_key to /keys/new"
    else
        # Update default_key in current Helm values
        kubectl -n "$NAMESPACE" get configmap cap-verifier-config -o json | \
            jq '.data."verifier.yaml" | fromjson | .verifier.signing.default_key = "/keys/new"' | \
            kubectl -n "$NAMESPACE" patch configmap cap-verifier-config --type merge -p '{"data":{"verifier.yaml":"'$(cat)'"}}'

        # Rolling restart to pick up new config
        log_info "Rolling restart to pick up new config..."
        kubectl -n "$NAMESPACE" rollout restart deployment cap-verifier-api
        kubectl -n "$NAMESPACE" rollout status deployment cap-verifier-api --timeout=5m

        log_success "Default signing key switched to new key"
    fi

    log_success "✅ Phase 2 complete: Signing with new key"
    log_info "Next step: Monitor signature distribution, then run Phase 3 to decommission old key"
}

# Phase 3: Decommission (retire old key)
phase_3_decommission() {
    log_info "🗑️  Phase 3: Decommission (T1 End)"
    log_info "================================="

    # Derive old KID
    local old_kid=$(derive_kid "$OLD_KEY")

    log_info "Old KID: $old_kid"

    if ! $FORCE; then
        read -p "Retire old key and end dual-accept mode? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            log_info "Aborting"
            exit 0
        fi
    fi

    # Archive old key
    log_info "Archiving old key: $OLD_KEY"

    if $DRY_RUN; then
        log_info "DRY RUN: Would archive old key"
    else
        cargo run -- keys archive \
            --dir keys \
            --kid "$old_kid"
        log_success "Old key archived"
    fi

    # Update Helm values to single-key mode
    log_info "Updating Helm values: switch to single-key mode"

    if $DRY_RUN; then
        log_info "DRY RUN: Would switch to single-key mode"
    else
        # Update to single-key mode (only new key)
        local new_kid=$(derive_kid "$NEW_KEY")

        # Create temporary YAML patch
        local temp_patch=$(mktemp)
        cat > "$temp_patch" <<EOF
verifier:
  signing:
    mode: single-key
    keys:
      - kid: "$old_kid"
        path: /keys/old
        status: retired
      - kid: "$new_kid"
        path: /keys/new
        status: active
    default_key: /keys/new
EOF

        # Apply Helm upgrade
        log_info "Applying Helm upgrade..."
        helm upgrade --install cap-verifier-api helm/ \
            --namespace "$NAMESPACE" \
            --values helm/values-prod.yaml \
            --values "$temp_patch" \
            --wait --timeout 10m

        rm "$temp_patch"
        log_success "Helm upgrade complete"
    fi

    # Verify pods are ready
    log_info "Waiting for pods to be ready..."
    if ! $DRY_RUN; then
        kubectl -n "$NAMESPACE" wait --for=condition=ready pod -l app=cap-verifier-api --timeout=300s
        log_success "All pods ready"
    fi

    log_success "✅ Phase 3 complete: Old key decommissioned"
    log_info "Rotation complete! Only new key is active."
}

# Rollback Phase 2 → Phase 1 (revert to old key signing)
rollback_phase_2() {
    log_warning "🔙 Rolling back Phase 2 to Phase 1"
    log_warning "=================================="

    # Derive KIDs
    local old_kid=$(derive_kid "$OLD_KEY")

    log_info "Old KID: $old_kid"

    if ! $FORCE; then
        read -p "Revert to signing with old key? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            log_info "Aborting"
            exit 0
        fi
    fi

    # Update Helm values to revert default_key to old
    log_info "Updating Helm values: revert default_key to /keys/old"

    if $DRY_RUN; then
        log_info "DRY RUN: Would revert default_key to /keys/old"
    else
        kubectl -n "$NAMESPACE" get configmap cap-verifier-config -o json | \
            jq '.data."verifier.yaml" | fromjson | .verifier.signing.default_key = "/keys/old"' | \
            kubectl -n "$NAMESPACE" patch configmap cap-verifier-config --type merge -p '{"data":{"verifier.yaml":"'$(cat)'"}}'

        # Rolling restart
        kubectl -n "$NAMESPACE" rollout restart deployment cap-verifier-api
        kubectl -n "$NAMESPACE" rollout status deployment cap-verifier-api --timeout=5m

        log_success "Rollback complete: Signing with old key"
    fi
}

# Rollback Phase 3 → Phase 2 (re-activate dual-accept)
rollback_phase_3() {
    log_warning "🔙 Rolling back Phase 3 to Phase 2"
    log_warning "=================================="

    # Derive KIDs
    local old_kid=$(derive_kid "$OLD_KEY")
    local new_kid=$(derive_kid "$NEW_KEY")

    log_info "Old KID: $old_kid"
    log_info "New KID: $new_kid"

    # Extend dual-accept period
    local dual_accept_until=$(date -u -v "+${DUAL_ACCEPT_DURATION}" +"%Y-%m-%dT%H:%M:%SZ" 2>/dev/null || \
                              date -u -d "+${DUAL_ACCEPT_DURATION}" +"%Y-%m-%dT%H:%M:%SZ")

    log_info "Extending dual-accept until: $dual_accept_until"

    if ! $FORCE; then
        read -p "Re-activate dual-accept mode? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            log_info "Aborting"
            exit 0
        fi
    fi

    # Update Helm values to dual-accept mode
    log_info "Updating Helm values: re-activate dual-accept mode"

    if $DRY_RUN; then
        log_info "DRY RUN: Would re-activate dual-accept mode"
    else
        # Create temporary YAML patch
        local temp_patch=$(mktemp)
        cat > "$temp_patch" <<EOF
verifier:
  signing:
    mode: dual-accept
    keys:
      - kid: "$old_kid"
        path: /keys/old
        status: active
      - kid: "$new_kid"
        path: /keys/new
        status: active
    default_key: /keys/new
    dual_accept_until: "$dual_accept_until"
EOF

        # Apply Helm upgrade
        helm upgrade --install cap-verifier-api helm/ \
            --namespace "$NAMESPACE" \
            --values helm/values-prod.yaml \
            --values "$temp_patch" \
            --wait --timeout 10m

        rm "$temp_patch"
        log_success "Helm upgrade complete"
    fi

    log_success "Rollback complete: Dual-accept mode re-activated"
    log_info "Dual-accept expires: $dual_accept_until"
}

# Parse command line arguments
parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            --phase)
                PHASE="$2"
                shift 2
                ;;
            --old-key)
                OLD_KEY="$2"
                shift 2
                ;;
            --new-key)
                NEW_KEY="$2"
                shift 2
                ;;
            --namespace)
                NAMESPACE="$2"
                shift 2
                ;;
            --duration)
                DUAL_ACCEPT_DURATION="$2"
                shift 2
                ;;
            --dry-run)
                DRY_RUN=true
                shift
                ;;
            --force)
                FORCE=true
                shift
                ;;
            --rollback)
                ROLLBACK=true
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
    if [[ -z "$PHASE" ]]; then
        log_error "Missing required argument: --phase"
        usage
        exit 2
    fi

    if [[ -z "$OLD_KEY" ]]; then
        log_error "Missing required argument: --old-key"
        usage
        exit 2
    fi

    # Phase 0-2 require new key
    if [[ "$PHASE" != "3" && -z "$NEW_KEY" && ! $ROLLBACK ]]; then
        log_error "Missing required argument: --new-key (required for phase $PHASE)"
        usage
        exit 2
    fi

    # Validate phase
    if [[ ! "$PHASE" =~ ^[0-3]$ ]]; then
        log_error "Invalid phase: $PHASE (must be 0, 1, 2, or 3)"
        exit 2
    fi

    # Verify old key exists
    verify_file "$OLD_KEY" "Old key metadata"

    # Verify new key exists (except for phase 3)
    if [[ "$PHASE" != "3" && -n "$NEW_KEY" ]]; then
        # For phase 0, new key is optional (will be generated)
        if [[ "$PHASE" != "0" ]]; then
            verify_file "$NEW_KEY" "New key metadata"
        fi
    fi
}

# Main rotation function
main() {
    parse_args "$@"

    log_info "🔐 CAP Verifier API Key Rotation Script v$SCRIPT_VERSION"
    log_info "========================================================"

    # Check prerequisites
    check_jq
    check_cargo

    # Only check kubectl if not phase 0 (phase 0 is local-only)
    if [[ "$PHASE" != "0" ]]; then
        check_kubectl
    fi

    # Execute phase or rollback
    if $ROLLBACK; then
        case $PHASE in
            2)
                rollback_phase_2
                ;;
            3)
                rollback_phase_3
                ;;
            *)
                log_error "Rollback not supported for phase $PHASE"
                exit 1
                ;;
        esac
    else
        case $PHASE in
            0)
                phase_0_preparation
                ;;
            1)
                phase_1_dual_accept
                ;;
            2)
                phase_2_sign_switch
                ;;
            3)
                phase_3_decommission
                ;;
            *)
                log_error "Invalid phase: $PHASE"
                exit 1
                ;;
        esac
    fi

    log_success "✅ Key rotation operation complete!"
    if [[ "$PHASE" == "0" ]]; then
        log_info "Next: Run Phase 1 with --phase 1"
    elif [[ "$PHASE" == "1" ]]; then
        log_info "Next: Monitor metrics, then run Phase 2 with --phase 2"
    elif [[ "$PHASE" == "2" ]]; then
        log_info "Next: Monitor signature distribution, then run Phase 3 with --phase 3"
    fi
}

# Run main function
main "$@"
