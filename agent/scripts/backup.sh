#!/usr/bin/env bash
#
# CAP Verifier API - Backup Script
# Week 6 - Track D1: Backup & Restore
#
# Description:
#   Creates a deterministic, evidence-preserving backup of the CAP Verifier API system.
#   Includes IR registry, policy store, configuration, and key metadata (public keys only).
#   Generates backup.manifest.json with SHA3-256 hashes for integrity verification.
#
# Usage:
#   ./scripts/backup.sh --output /backup/cap-backup.tar.gz \
#                       --registry build/registry.sqlite \
#                       --policy-store build/policy_store.json
#
# Requirements:
#   - bash >= 4.0
#   - tar, gzip
#   - sha3sum (or openssl for SHA3-256)
#   - jq (for JSON manipulation)

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default values
OUTPUT=""
REGISTRY=""
POLICY_STORE=""
KEYS_DIR=""
CONFIG=""
MANIFEST=""
COMPRESS=true
ENCRYPT=false
ENCRYPTION_KEY=""
VERBOSE=false

# Script metadata
SCRIPT_VERSION="1.0.0"
BACKUP_VERSION="backup.manifest.v1"

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

Creates a deterministic, evidence-preserving backup of CAP Verifier API.

Options:
  --output PATH          Output tar.gz file path (required)
  --registry PATH        Path to registry.sqlite or registry.json (required)
  --policy-store PATH    Path to policy_store.json (required)
  --keys PATH            Path to keys directory (public keys only, optional)
  --config PATH          Path to helm/values-prod.yaml (optional)
  --manifest PATH        Output path for backup.manifest.json (optional, default: included in tar)
  --compress             Enable gzip compression (default: true)
  --no-compress          Disable compression
  --encrypt              Enable AES-256-GCM encryption (default: false)
  --encryption-key PATH  Path to encryption key (required if --encrypt enabled)
  --verbose              Enable verbose output
  --help                 Show this help message

Examples:
  # Basic backup
  $0 --output /backup/cap-backup.tar.gz \\
     --registry build/registry.sqlite \\
     --policy-store build/policy_store.json

  # Full backup with keys and config
  $0 --output /backup/cap-backup-full.tar.gz \\
     --registry build/registry.sqlite \\
     --policy-store build/policy_store.json \\
     --keys keys/ \\
     --config helm/values-prod.yaml

  # Encrypted backup
  $0 --output /backup/cap-backup-encrypted.tar.gz \\
     --registry build/registry.sqlite \\
     --policy-store build/policy_store.json \\
     --encrypt \\
     --encryption-key /keys/backup.key

Environment Variables:
  BACKUP_OUTPUT          Same as --output
  BACKUP_REGISTRY        Same as --registry
  BACKUP_POLICY_STORE    Same as --policy-store

Exit Codes:
  0   Success
  1   General error
  2   Missing required argument
  3   File not found
  4   Verification failed

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

# Check for PII in file (basic heuristic)
check_no_pii() {
    local file="$1"

    # Check for common PII patterns (names, emails, phone numbers)
    if grep -qiE '(email|phone|address|firstname|lastname|ssn|dob|birthdate)' "$file"; then
        log_warning "Potential PII detected in $file (review manually)"
        return 1
    fi

    return 0
}

# Parse command line arguments
parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            --output)
                OUTPUT="$2"
                shift 2
                ;;
            --registry)
                REGISTRY="$2"
                shift 2
                ;;
            --policy-store)
                POLICY_STORE="$2"
                shift 2
                ;;
            --keys)
                KEYS_DIR="$2"
                shift 2
                ;;
            --config)
                CONFIG="$2"
                shift 2
                ;;
            --manifest)
                MANIFEST="$2"
                shift 2
                ;;
            --compress)
                COMPRESS=true
                shift
                ;;
            --no-compress)
                COMPRESS=false
                shift
                ;;
            --encrypt)
                ENCRYPT=true
                shift
                ;;
            --encryption-key)
                ENCRYPTION_KEY="$2"
                shift 2
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
    if [[ -z "$OUTPUT" ]]; then
        log_error "Missing required argument: --output"
        usage
        exit 2
    fi

    if [[ -z "$REGISTRY" ]]; then
        log_error "Missing required argument: --registry"
        usage
        exit 2
    fi

    if [[ -z "$POLICY_STORE" ]]; then
        log_error "Missing required argument: --policy-store"
        usage
        exit 2
    fi

    if [[ "$ENCRYPT" == true && -z "$ENCRYPTION_KEY" ]]; then
        log_error "Encryption enabled but --encryption-key not provided"
        exit 2
    fi
}

# Create backup manifest JSON
create_manifest() {
    local temp_dir="$1"
    local manifest_file="$temp_dir/backup.manifest.json"
    local files_json="[]"

    log_info "Generating backup manifest..."

    # Add registry file
    local registry_basename=$(basename "$REGISTRY")
    local registry_hash=$(compute_sha3 "$REGISTRY")
    local registry_size=$(stat -f%z "$REGISTRY" 2>/dev/null || stat -c%s "$REGISTRY")
    files_json=$(echo "$files_json" | jq --arg path "$registry_basename" \
                                           --arg hash "$registry_hash" \
                                           --arg size "$registry_size" \
                                           '. += [{"path": $path, "sha3_256": $hash, "size_bytes": ($size|tonumber), "type": "database"}]')

    # Add policy store file
    local policy_basename=$(basename "$POLICY_STORE")
    local policy_hash=$(compute_sha3 "$POLICY_STORE")
    local policy_size=$(stat -f%z "$POLICY_STORE" 2>/dev/null || stat -c%s "$POLICY_STORE")
    files_json=$(echo "$files_json" | jq --arg path "$policy_basename" \
                                          --arg hash "$policy_hash" \
                                          --arg size "$policy_size" \
                                          '. += [{"path": $path, "sha3_256": $hash, "size_bytes": ($size|tonumber), "type": "policy_store"}]')

    # Add keys directory (if provided)
    if [[ -n "$KEYS_DIR" && -d "$KEYS_DIR" ]]; then
        # Compute hash of entire directory (tar + hash)
        local keys_tarball="$temp_dir/keys.tar"
        tar -cf "$keys_tarball" -C "$(dirname "$KEYS_DIR")" "$(basename "$KEYS_DIR")"
        local keys_hash=$(compute_sha3 "$keys_tarball")
        local keys_size=$(stat -f%z "$keys_tarball" 2>/dev/null || stat -c%s "$keys_tarball")
        files_json=$(echo "$files_json" | jq --arg path "keys/" \
                                              --arg hash "$keys_hash" \
                                              --arg size "$keys_size" \
                                              '. += [{"path": $path, "sha3_256": $hash, "size_bytes": ($size|tonumber), "type": "key_metadata"}]')
        rm "$keys_tarball"
    fi

    # Add config file (if provided)
    if [[ -n "$CONFIG" && -f "$CONFIG" ]]; then
        local config_basename=$(basename "$CONFIG")
        local config_hash=$(compute_sha3 "$CONFIG")
        local config_size=$(stat -f%z "$CONFIG" 2>/dev/null || stat -c%s "$CONFIG")
        files_json=$(echo "$files_json" | jq --arg path "$config_basename" \
                                              --arg hash "$config_hash" \
                                              --arg size "$config_size" \
                                              '. += [{"path": $path, "sha3_256": $hash, "size_bytes": ($size|tonumber), "type": "configuration"}]')
    fi

    # Compute totals
    local total_files=$(echo "$files_json" | jq 'length')
    local total_size=$(echo "$files_json" | jq '[.[].size_bytes] | add')

    # Create manifest JSON
    local manifest_json=$(jq -n \
        --arg version "$BACKUP_VERSION" \
        --arg created_at "$(date -u +"%Y-%m-%dT%H:%M:%SZ")" \
        --arg created_by "$(whoami)@$(hostname)" \
        --arg backup_id "backup-$(date +%Y%m%d-%H%M%S)" \
        --arg system_version "0.11.0" \
        --argjson files "$files_json" \
        --arg total_files "$total_files" \
        --arg total_size "$total_size" \
        --arg compression "$(if $COMPRESS; then echo gzip; else echo none; fi)" \
        --arg encryption "$(if $ENCRYPT; then echo aes-256-gcm; else echo none; fi)" \
        '{
            version: $version,
            created_at: $created_at,
            created_by: $created_by,
            backup_id: $backup_id,
            system_version: $system_version,
            files: $files,
            total_files: ($total_files|tonumber),
            total_size_bytes: ($total_size|tonumber),
            compression: $compression,
            encryption: $encryption
        }')

    echo "$manifest_json" > "$manifest_file"
    log_success "Manifest created: $manifest_file"

    if $VERBOSE; then
        echo "$manifest_json" | jq '.'
    fi
}

# Create backup archive
create_archive() {
    local temp_dir="$1"

    log_info "Creating backup archive..."

    # Copy files to temp directory
    cp "$REGISTRY" "$temp_dir/"
    cp "$POLICY_STORE" "$temp_dir/"

    if [[ -n "$KEYS_DIR" && -d "$KEYS_DIR" ]]; then
        mkdir -p "$temp_dir/keys"
        cp -r "$KEYS_DIR"/* "$temp_dir/keys/"
    fi

    if [[ -n "$CONFIG" && -f "$CONFIG" ]]; then
        cp "$CONFIG" "$temp_dir/"
    fi

    # Create tar archive
    local tar_opts="-cf"
    if $COMPRESS; then
        tar_opts="-czf"
    fi

    cd "$temp_dir"
    tar $tar_opts "$OUTPUT" ./*

    log_success "Archive created: $OUTPUT"

    # Display archive contents
    if $VERBOSE; then
        log_info "Archive contents:"
        tar -tzf "$OUTPUT" | head -20
    fi

    # Display size
    local archive_size=$(stat -f%z "$OUTPUT" 2>/dev/null || stat -c%s "$OUTPUT")
    local archive_size_mb=$((archive_size / 1024 / 1024))
    log_info "Archive size: ${archive_size_mb} MB"
}

# Main backup function
main() {
    parse_args "$@"

    log_info "🔒 CAP Verifier API Backup Script v$SCRIPT_VERSION"
    log_info "=================================="

    # Verify input files exist
    verify_file "$REGISTRY" "Registry"
    verify_file "$POLICY_STORE" "Policy Store"

    if [[ -n "$KEYS_DIR" ]]; then
        if [[ ! -d "$KEYS_DIR" ]]; then
            log_error "Keys directory not found: $KEYS_DIR"
            exit 3
        fi
    fi

    if [[ -n "$CONFIG" ]]; then
        verify_file "$CONFIG" "Config"
    fi

    # Check for PII (basic heuristic)
    log_info "Checking for PII..."
    if ! check_no_pii "$REGISTRY"; then
        log_warning "Potential PII detected in registry (manual review recommended)"
    fi

    if ! check_no_pii "$POLICY_STORE"; then
        log_warning "Potential PII detected in policy store (manual review recommended)"
    fi

    # Create temporary directory for staging
    local temp_dir=$(mktemp -d)
    trap "rm -rf $temp_dir" EXIT

    # Create manifest
    create_manifest "$temp_dir"

    # Create archive
    create_archive "$temp_dir"

    # Copy manifest to output location if specified
    if [[ -n "$MANIFEST" ]]; then
        cp "$temp_dir/backup.manifest.json" "$MANIFEST"
        log_success "Manifest copied to: $MANIFEST"
    fi

    # Encrypt archive if requested
    if $ENCRYPT; then
        log_info "Encrypting archive..."
        openssl enc -aes-256-gcm -salt -in "$OUTPUT" -out "$OUTPUT.enc" -pass file:"$ENCRYPTION_KEY"
        mv "$OUTPUT.enc" "$OUTPUT"
        log_success "Archive encrypted"
    fi

    log_success "✅ Backup completed successfully!"
    log_info "Backup location: $OUTPUT"

    # Display manifest summary
    jq -r '.backup_id, .total_files, .total_size_bytes' "$temp_dir/backup.manifest.json" | \
        awk 'NR==1{id=$0} NR==2{files=$0} NR==3{size=$0; mb=size/1024/1024; printf "  Backup ID: %s\n  Files: %s\n  Size: %.2f MB\n", id, files, mb}'
}

# Run main function
main "$@"
