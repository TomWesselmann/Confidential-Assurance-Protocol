# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- CONTRIBUTING.md with development guidelines
- CHANGELOG.md (this file)
- Pre-commit hooks for code quality

### Changed
- Improved error handling: replaced critical `unwrap()` calls with proper error handling
- Enhanced Mutex lock messages with `expect()` context

### Fixed
- Clippy warnings: `-D warnings` now passes cleanly
- Rustdoc warnings: `cargo doc --no-deps` passes with `-D warnings`

### Removed
- Orphaned `error.rs` file (never integrated, used missing thiserror dependency)

## [0.12.2] - 2025-12-14

### Changed
- Phase 4 Technical Debt cleanup completed
- Phase 1-3 Production Roadmap completed
- Full workspace CI/CD pipeline

### Fixed
- All Clippy warnings resolved
- Test compilation issues fixed

## [0.12.0] - 2025-12-14

### Added
- Desktop App (Tauri 2.0) with 6-step prover workflow
- Verifier workflow for bundle verification
- Audit trail visualization
- Frontend test coverage (98.95% with 268 tests)
- E2E test infrastructure (WebdriverIO)
- CI/CD pipeline for frontend and Tauri builds

### Changed
- Migrated to Tauri 2.0 architecture
- Updated all dependencies to latest versions

## [0.11.0] - 2025-12-13

### Added
- WebUI integration with Policy Store
- Backend API for policy management
- Comprehensive E2E workflow tests
- Package Flow Refactoring

### Changed
- Policy Engine v2 with linting support
- Bundle V2 format standardization

### Fixed
- Race conditions in policy and metrics tests

## [0.10.0] - 2025-12-12

### Added
- SAP S/4HANA adapter (OData v4 integration)
- Production-ready monitoring stack (Prometheus, Grafana, Loki, Jaeger)
- TLS/mTLS support for API
- Security audit integration (cargo audit)

### Changed
- Enhanced security design for Web UI

## [0.9.0] - 2025-12-11

### Added
- Policy Engine with YAML-based compliance rules
- Key rotation and attestation support
- BLOB Store (Content-Addressable Storage)
- Registry backend (JSON/SQLite)

### Changed
- Improved audit trail format (V1.0)
- Ed25519 signing with attestation chain

## [0.8.0] - 2025-12-10

### Added
- SimplifiedZK proof system
- Cryptographic commitments (BLAKE3 Merkle roots)
- SHA3-256 audit trails
- CLI tool for automation

### Changed
- Initial bundle format specification

---

## Release Notes

### Version Naming

- **0.x.y**: Development releases
- **1.0.0**: First production release (planned)

### Breaking Changes Policy

During 0.x development, breaking changes may occur between minor versions.
Starting with 1.0.0, we will follow strict semantic versioning.

---

[Unreleased]: https://github.com/TomWesselmann/Confidential-Assurance-Protocol/compare/v0.12.2...HEAD
[0.12.2]: https://github.com/TomWesselmann/Confidential-Assurance-Protocol/compare/v0.12.0...v0.12.2
[0.12.0]: https://github.com/TomWesselmann/Confidential-Assurance-Protocol/compare/v0.11.0...v0.12.0
[0.11.0]: https://github.com/TomWesselmann/Confidential-Assurance-Protocol/compare/v0.10.0...v0.11.0
[0.10.0]: https://github.com/TomWesselmann/Confidential-Assurance-Protocol/compare/v0.9.0...v0.10.0
[0.9.0]: https://github.com/TomWesselmann/Confidential-Assurance-Protocol/compare/v0.8.0...v0.9.0
[0.8.0]: https://github.com/TomWesselmann/Confidential-Assurance-Protocol/releases/tag/v0.8.0
