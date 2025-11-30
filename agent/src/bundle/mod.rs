//! Bundle Layer - cap-bundle.v1 Format
//!
//! Dieses Modul definiert das standardisierte Proof-Package-Format
//! cap-bundle.v1 mit Metadaten, File-Hashes und Proof-Units.
//!
//! ## Bundle-Source-Abstraktion (REQ-03)
//!
//! Unterstützt mehrere Bundle-Quellen:
//! - `BundleSource::Directory`: Entpacktes Bundle-Verzeichnis
//! - `BundleSource::ZipFile`: ZIP-Archiv mit Bundle-Inhalt
//!
//! Zukünftig erweiterbar um:
//! - `BundleSource::Memory`: In-Memory-Bundle
//! - `BundleSource::Stream`: Streaming-Source

pub mod meta;
pub mod source;

pub use meta::{BundleFileMeta, BundleMeta, ProofUnitMeta, BUNDLE_SCHEMA_V1};
pub use source::{load_bundle_atomic, parse_bundle_source, BundleData, BundleSource};
