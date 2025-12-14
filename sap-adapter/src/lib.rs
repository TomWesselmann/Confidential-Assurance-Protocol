//! SAP Adapter Library - Production-Ready OData v4 Integration
//!
//! Version 0.3.0 - Implements CAP Engineering Guide principles
//!
//! ## Modules
//! - `odata_client`: OData v4 client for SAP S/4HANA Business Partner API
//! - `sanitizer`: Input validation and cleansing
//! - `mapper`: Deterministic SAP → CAP context transformation with BLAKE3

pub mod odata_client;
pub mod sanitizer;
pub mod mapper;

pub use odata_client::{ODataClient, ODataConfig, SapBusinessPartner};
pub use sanitizer::{sanitize_business_partner, sanitize_batch, SanitizedBusinessPartner};
pub use mapper::{map_to_cap_supplier, map_to_cap_context, CapSupplier, CapContext};
