# Managing Clippy Lint Warnings

## Quick Reference: Commands to Find Warnings

```bash
# 1. Get a summary of all warnings
cargo clippy --all-targets --all-features -- -D warnings 2>&1 | grep "error:" | cut -d: -f4- | sort | uniq -c | sort -rn

# 2. Find specific lint type (e.g., zombie_processes)
cargo clippy --all-targets --all-features 2>&1 | grep -A 5 "zombie_processes"

# 3. Save full output for detailed review
cargo clippy --all-targets --all-features -- -D warnings 2>&1 > clippy_report.txt
```

## How to Handle a Warning: Decision Tree

```
New Clippy Warning Appears
         ↓
    Can you fix it easily?
    ├─ YES → Fix it immediately
    └─ NO  → Is it a real issue?
           ├─ YES → Create TODO/Issue to fix later + add #[allow]
           └─ NO  → Is it intentional behavior?
                  ├─ YES → Add #[allow] with clear comment
                  └─ NO  → Investigate further
```

## Pattern: Adding Intentional Suppressions

### Good Example (Clear Documentation)

```rust
// ALLOW(clippy::zombie_processes): Integration test pattern.
// The spawned process is intentionally left running to test
// server behavior under load. OS cleanup handles termination.
#[allow(clippy::zombie_processes)]
#[test]
fn test_server_under_load() {
    Command::new("cargo")
        .args(["run", "--bin", "server"])
        .spawn()
        .expect("Failed to spawn server");

    // Test logic here...
}
```

### Bad Example (No Documentation)

```rust
#[allow(clippy::zombie_processes)]  // ❌ Why? What's the intent?
#[test]
fn test_something() {
    Command::new("foo").spawn().unwrap();
}
```

## Pattern: Module-Level Suppressions

For test modules with many integration tests that share the same pattern:

```rust
// Integration tests that spawn background processes
// All tests in this module follow the "spawn-and-forget" pattern
// where the OS handles cleanup.
#[cfg(test)]
#[allow(clippy::zombie_processes)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_1() { /* ... */ }

    #[test]
    fn test_2() { /* ... */ }
}
```

## Common Warnings and How to Handle Them

### 1. `clippy::zombie_processes`

**What it is**: Spawned process not `.wait()`ed on

**When to allow**:
- Integration tests where OS handles cleanup
- Background daemon processes in test fixtures
- Process spawning tests where wait() is tested separately

**How to fix instead**:
```rust
// Before (warning):
Command::new("foo").spawn()?;

// After (no warning):
let mut child = Command::new("foo").spawn()?;
child.wait()?;  // or .kill() or use a guard

// Or use a RAII guard:
struct ProcessGuard(Child);
impl Drop for ProcessGuard {
    fn drop(&mut self) {
        let _ = self.0.kill();
    }
}
```

### 2. `clippy::needless_borrows_for_generic_args`

**What it is**: Unnecessary `&` when calling generic functions

**When to allow**:
- Explicit API documentation of lifetime requirements
- Performance-critical code where clarity matters
- Consistency with surrounding code patterns

**How to fix instead**:
```rust
// Before (warning):
verify_signature(&signature, &public_key);

// After (if the function accepts owned values):
verify_signature(signature, public_key);

// Or if you need to keep using the values:
verify_signature(signature.clone(), public_key.clone());
```

### 3. `clippy::suspicious_open_options`

**What it is**: `create(true)` without `truncate()` or `append()`

**Always fix this one**:
```rust
// Before (warning):
OpenOptions::new()
    .create(true)
    .write(true)
    .open("foo.txt")?;

// After (clear intent):
OpenOptions::new()
    .create(true)
    .write(true)
    .truncate(true)  // ← Add this
    .open("foo.txt")?;
```

### 4. `clippy::let_and_return`

**What it is**: Unnecessary intermediate variable before return

**When to allow**:
- Benchmark code where variable naming aids readability
- Complex expressions where naming clarifies intent
- Debugging (temporary - remove after)

**How to fix instead**:
```rust
// Before (warning):
let result = expensive_computation();
result

// After:
expensive_computation()

// Or if naming helps:
expensive_computation() // result: ComplexType
```

## CI/CD Integration

### Recommended `.github/workflows/lint.yml`:

```yaml
name: Lint

on: [push, pull_request]

jobs:
  clippy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy

      - name: Run Clippy
        run: cargo clippy --all-targets --all-features -- -D warnings

      - name: Generate Lint Report (on failure)
        if: failure()
        run: |
          cargo clippy --all-targets --all-features 2>&1 | tee clippy_report.txt
          echo "## Clippy Failed" >> $GITHUB_STEP_SUMMARY
          echo "\`\`\`" >> $GITHUB_STEP_SUMMARY
          head -50 clippy_report.txt >> $GITHUB_STEP_SUMMARY
          echo "\`\`\`" >> $GITHUB_STEP_SUMMARY
```

## Reviewing Suppressions Quarterly

Create a GitHub issue every 3 months:

```markdown
## Quarterly Lint Suppression Review

**Goal**: Review all `#[allow(clippy::...)]` attributes to see if they can be removed.

### Checklist:
- [ ] Run: `rg '#\[allow\(clippy::' --stats`
- [ ] Review each suppression in LINT_POLICY.md
- [ ] Check if Rust/Clippy behavior has changed
- [ ] Try removing suppressions one by one
- [ ] Update LINT_POLICY.md with findings

### Questions:
- Can we fix any suppressions now that didn't make sense before?
- Are there new clippy lints we should allow/deny?
- Do our patterns still match our documentation?
```

## Useful Cargo Commands

```bash
# Check specific lint level
cargo clippy -- -W clippy::zombie_processes

# Allow specific lint globally (add to Cargo.toml or build.rs)
# Not recommended, but possible:
[lints.clippy]
zombie_processes = "allow"

# See all available lints
rustc -W help | grep clippy

# Run clippy with different strictness
cargo clippy -- -W clippy::pedantic
cargo clippy -- -W clippy::nursery
```

## Further Reading

- [Clippy Documentation](https://doc.rust-lang.org/clippy/)
- [Lint Levels RFC](https://doc.rust-lang.org/rustc/lints/levels.html)
- [rust-lang/rust-clippy on GitHub](https://github.com/rust-lang/rust-clippy)
