# Clippy Lint Policy

This document describes intentional lint suppressions in the CAP Agent codebase.

## Globally Allowed Lints (None)

We strive for zero clippy warnings. No lints are globally suppressed.

## Intentionally Suppressed Lints (By Category)

### 1. `clippy::zombie_processes` (8 instances)

**Where**: Integration test files
**Why**: Integration tests spawn subprocesses that are intentionally not waited on. The OS or test framework handles cleanup.

**Pattern**:
```rust
#[allow(clippy::zombie_processes)]
#[test]
fn test_integration() {
    Command::new("cargo").spawn().expect("spawn failed");
    // Process cleanup handled by OS/test framework
}
```

**Files**:
- `tests/test_integration_http.rs` (multiple instances)
- `tests/test_bundle_v2.rs`
- `tests/audit_chain_tamper.rs`

### 2. `clippy::needless_borrows_for_generic_args` (10 instances)

**Where**: Various test files
**Why**: Explicit borrows for:
- API consistency with production code
- Clear lifetime documentation
- Some generic function requirements

**Pattern**:
```rust
// Intentional borrow for consistency with non-test code
let result = verify_signature(&signature, &public_key);
```

**Action Required**: Review each instance - some may be fixable.

### 3. `clippy::suspicious_open_options` (2 instances)

**Where**: File I/O operations
**Why**: Existing behavior that works correctly

**Pattern**:
```rust
OpenOptions::new().create(true).write(true).open(path)
// Missing explicit .truncate(true) or .append(true)
```

**Action Required**: Add explicit `.truncate(true)` to clarify intent.

### 4. `clippy::let_and_return` (1 instance)

**Where**: Benchmark code
**Why**: Clear variable naming for benchmark readability

**Action Required**: Low priority - can be simplified.

## How to Handle New Warnings

### When a Clippy Warning Appears:

1. **Try to fix it first** - Most warnings indicate real issues or improvements
2. **If intentional**, add an `#[allow(...)]` attribute with a comment:
   ```rust
   // ALLOW: Integration test pattern - OS handles cleanup
   #[allow(clippy::zombie_processes)]
   fn test_something() { ... }
   ```
3. **Document in this file** - Add the lint to the appropriate category above
4. **Review periodically** - Re-evaluate suppressions during refactoring

### CI/CD Policy

- **CI runs**: `cargo clippy --all-targets --all-features -- -D warnings`
- **Goal**: Zero warnings (with documented exceptions using `#[allow]`)
- **Failed builds**: Investigate new warnings, fix or document

## Reviewing Suppressions

Schedule quarterly reviews of this document to:
- Remove obsolete suppressions
- Fix previously "intentional" warnings
- Update patterns as best practices evolve
