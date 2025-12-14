# Contributing to CAP (Confidential Assurance Protocol)

Thank you for your interest in contributing to CAP! This document provides guidelines and instructions for contributing to the project.

## Table of Contents

- [Development Setup](#development-setup)
- [Code Style Guide](#code-style-guide)
- [Pull Request Process](#pull-request-process)
- [Testing Requirements](#testing-requirements)
- [Commit Message Format](#commit-message-format)

---

## Development Setup

### Prerequisites

- **Rust** 1.70+ with `rustfmt` and `clippy`
- **Node.js** 24+ with npm
- **Git** 2.30+

### Getting Started

```bash
# 1. Clone the repository
git clone https://github.com/TomWesselmann/Confidential-Assurance-Protocol.git
cd Confidential-Assurance-Protocol

# 2. Install Rust toolchain components
rustup component add rustfmt clippy

# 3. Build the workspace
cargo build --workspace

# 4. Run tests
cargo test --workspace

# 5. Install frontend dependencies
cd tauri-frontend && npm ci
```

### Project Structure

```
LsKG-Agent/
├── agent/              # Rust Core Library + CLI (cap-agent crate)
├── src-tauri/          # Tauri Desktop App Backend (cap-tauri crate)
├── tauri-frontend/     # React Frontend (TypeScript)
├── sap-adapter/        # SAP S/4HANA Integration
├── infrastructure/     # Docker, K8s, Monitoring configs
└── docs/               # Project documentation
```

---

## Code Style Guide

### Rust Code

- **Formatting**: Run `cargo fmt --all` before committing
- **Linting**: Code must pass `cargo clippy --workspace -- -D warnings`
- **Documentation**: All public items require doc comments
- **Error Handling**: Use `anyhow::Result` in CLI, proper error types in library code
- **No unwrap()**: Use `?` operator or `.expect("context")` with meaningful messages

```rust
// Good
let value = map.get("key").ok_or_else(|| anyhow!("key not found"))?;

// Acceptable (with context)
let value = map.get("key").expect("key must exist after validation");

// Bad
let value = map.get("key").unwrap();
```

### TypeScript/React Code

- **Formatting**: Run `npm run lint` before committing
- **Type Safety**: No `any` types - use proper TypeScript types
- **Testing**: Component tests required for new features
- **Imports**: Use absolute imports from `@/` prefix

---

## Pull Request Process

### Before Opening a PR

1. **Create a feature branch** from `main`:
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Run all checks locally**:
   ```bash
   # Rust
   cargo fmt --all -- --check
   cargo clippy --workspace -- -D warnings
   cargo test --workspace

   # Frontend
   cd tauri-frontend
   npm run lint
   npm run build
   npm test
   ```

3. **Ensure tests pass**: All existing tests must pass

### PR Requirements

- **Title**: Use conventional commit format (see below)
- **Description**: Explain what changes were made and why
- **Tests**: Include tests for new functionality
- **Documentation**: Update relevant documentation

### Review Process

1. CI checks must pass
2. At least one approval from a maintainer
3. All review comments addressed
4. Branch is up-to-date with `main`

---

## Testing Requirements

### Rust Tests

```bash
# Run all tests
cargo test --workspace

# Run with output
cargo test --workspace -- --nocapture

# Run specific test
cargo test test_name

# Coverage (requires cargo-tarpaulin)
cargo tarpaulin --workspace --out Html
```

### Frontend Tests

```bash
cd tauri-frontend

# Run all tests
npm test

# Run with coverage
npm run test:coverage

# Run E2E tests
npm run e2e
```

### Test Coverage Requirements

- **Rust**: Minimum 70% line coverage for new code
- **Frontend**: Minimum 80% line coverage for new code

---

## Commit Message Format

We follow the [Conventional Commits](https://www.conventionalcommits.org/) specification:

```
<type>(<scope>): <description>

[optional body]

[optional footer(s)]
```

### Types

- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `chore`: Maintenance tasks
- `ci`: CI/CD changes
- `perf`: Performance improvements

### Scopes

- `agent`: Changes to cap-agent crate
- `tauri`: Changes to Tauri backend
- `frontend`: Changes to React frontend
- `cli`: CLI-specific changes
- `policy`: Policy engine changes
- `bundle`: Bundle format changes

### Examples

```
feat(agent): add support for policy v2 format

fix(frontend): resolve verification status display bug

docs(readme): update quick start instructions

test(agent): add unit tests for bundle validation

chore(ci): update GitHub Actions workflow
```

---

## Questions?

If you have questions about contributing, please:

1. Check existing [documentation](docs/)
2. Search [existing issues](https://github.com/TomWesselmann/Confidential-Assurance-Protocol/issues)
3. Open a new issue for discussion

---

**Thank you for contributing to CAP!**
