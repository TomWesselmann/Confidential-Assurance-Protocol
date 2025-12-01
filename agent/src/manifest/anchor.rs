//! Time Anchor Types - Dual-Anchor Support (v0.9.0)
//!
//! Provides time anchoring for manifests via TSA, blockchain, or file-based timestamps.

use chrono::Utc;
use serde::{Deserialize, Serialize};

/// Public Chain Enum für Blockchain-Anker
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PublicChain {
    Ethereum,
    Hedera,
    #[serde(rename = "btc")]
    Btc,
}

/// Private Anchor (lokaler Audit-Tip)
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct TimeAnchorPrivate {
    pub audit_tip_hex: String, // 0x-prefixed SHA3-256 hash
    pub created_at: String,    // RFC3339 Timestamp
}

/// Public Anchor (öffentlicher Ledger-Verweis)
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct TimeAnchorPublic {
    pub chain: PublicChain,
    pub txid: String,       // Transaction ID (format depends on chain)
    pub digest: String,     // 0x-prefixed hash of audit_tip for on-chain notarization
    pub created_at: String, // RFC3339 Timestamp
}

/// Zeitanker für externe Timestamps (Dual-Anchor Support v0.9.0)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TimeAnchor {
    pub kind: String,          // "tsa", "blockchain", "file", "none"
    pub reference: String,     // Pfad, TxID oder URI
    pub audit_tip_hex: String, // Audit-Chain-Tip zum Zeitpunkt des Anchors
    pub created_at: String,    // RFC3339 Timestamp

    // Dual-Anchor Fields (v0.9.0, optional, additive)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private: Option<TimeAnchorPrivate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public: Option<TimeAnchorPublic>,
}

impl TimeAnchor {
    /// Creates a new TimeAnchor
    pub fn new(kind: String, reference: String, audit_tip_hex: String) -> Self {
        Self {
            kind,
            reference,
            audit_tip_hex,
            created_at: Utc::now().to_rfc3339(),
            private: None,
            public: None,
        }
    }
}
