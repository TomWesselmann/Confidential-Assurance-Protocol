/// SAP Adapter - Week 2: E2E Integration with Verifier API
/// Reads mock SAP data, hashes with BLAKE3, sends to Verifier API, parses response

use anyhow::{Context as AnyhowContext, Result};
use blake3::Hasher;
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Parser)]
struct Cli {
    #[arg(short, long, default_value = "examples/suppliers.json")]
    suppliers: PathBuf,

    #[arg(short, long)]
    output: Option<PathBuf>,

    #[arg(long)]
    dry_run: bool,

    /// Verifier API base URL
    #[arg(long, default_value = "https://localhost:8443")]
    verifier_url: String,

    /// Accept self-signed TLS certificates (dev only)
    #[arg(long)]
    accept_invalid_certs: bool,

    /// Skip actual API call (Week 1 mode)
    #[arg(long)]
    skip_verify: bool,
}

#[derive(Debug, Deserialize)]
struct SapSupplier {
    #[serde(rename = "LIFNR")]
    id: String,
    #[serde(rename = "NAME1")]
    name: String,
    #[serde(rename = "LAND1")]
    country: String,
    #[serde(rename = "TIER")]
    tier: String,
}

#[derive(Debug, Deserialize)]
struct SapMockData {
    suppliers: Vec<SapSupplier>,
}

#[derive(Debug, Clone, Serialize)]
struct HashedSupplier {
    id_hash: String,
    country: String,
    tier: String,
}

#[derive(Debug, Serialize)]
struct ContextData {
    supplier_hashes: Vec<String>,
    supplier_regions: Vec<String>,
}

#[derive(Debug, Serialize)]
struct VerifyRequest {
    policy_id: String,
    context: ContextData,
    backend: String,
}

#[derive(Debug, Deserialize)]
struct VerifyResponse {
    result: String,
    #[serde(default)]
    valid_until: Option<String>,
    #[serde(default)]
    manifest_hash: Option<String>,
    #[serde(default)]
    trace: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
struct Context {
    suppliers: Vec<HashedSupplier>,
    total_count: usize,
}

fn hash_field(input: &str) -> String {
    let mut hasher = Hasher::new();
    hasher.update(input.as_bytes());
    format!("0x{}", hasher.finalize().to_hex())
}

async fn call_verifier_api(cli: &Cli, request: &VerifyRequest) -> Result<VerifyResponse> {
    // Build HTTP client with optional self-signed cert support
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(cli.accept_invalid_certs)
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .context("Failed to build HTTP client")?;

    let url = format!("{}/verify", cli.verifier_url);
    println!("📡 POST {}", url);

    let response = client
        .post(&url)
        .json(request)
        .send()
        .await
        .context("Failed to send request to Verifier API")?;

    let status = response.status();
    println!("📥 Response: {}", status);

    if !status.is_success() {
        let error_text = response.text().await.unwrap_or_default();
        anyhow::bail!("Verifier API error {}: {}", status, error_text);
    }

    let verify_response: VerifyResponse = response
        .json()
        .await
        .context("Failed to parse Verifier response")?;

    Ok(verify_response)
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    println!("🧩 SAP Adapter v0.2.0 (Week 2: E2E Integration)");
    println!("📂 Reading: {}", cli.suppliers.display());

    // Load SAP mock data
    let data: SapMockData = serde_json::from_reader(std::fs::File::open(&cli.suppliers)?)?;
    println!("✅ Loaded {} suppliers", data.suppliers.len());

    // Hash sensitive fields (BLAKE3)
    let hashed_suppliers: Vec<HashedSupplier> = data
        .suppliers
        .iter()
        .map(|s| HashedSupplier {
            id_hash: hash_field(&format!("{}:{}", s.id, s.name)),
            country: s.country.clone(),
            tier: s.tier.clone(),
        })
        .collect();

    let context = Context {
        total_count: hashed_suppliers.len(),
        suppliers: hashed_suppliers.clone(),
    };

    println!("🔐 Hashed {} suppliers with BLAKE3", context.total_count);

    // Output context.json (legacy format)
    if let Some(output) = &cli.output {
        std::fs::write(output, serde_json::to_string_pretty(&context)?)?;
        println!("💾 Saved to: {}", output.display());
    }

    if cli.dry_run {
        println!("\n📄 Context Preview:");
        println!("{}", serde_json::to_string_pretty(&context)?);
    }

    // Skip verification if requested (Week 1 mode)
    if cli.skip_verify {
        println!("\n⏭️  Skipping verification (--skip-verify)");
        return Ok(());
    }

    // Build VerifyRequest (PRD format)
    let verify_request = VerifyRequest {
        policy_id: "lksg.v1".to_string(),
        context: ContextData {
            supplier_hashes: hashed_suppliers.iter().map(|s| s.id_hash.clone()).collect(),
            supplier_regions: hashed_suppliers.iter().map(|s| s.country.clone()).collect(),
        },
        backend: "mock".to_string(),
    };

    println!("\n🔍 Calling Verifier API...");

    // Call Verifier API
    match call_verifier_api(&cli, &verify_request).await {
        Ok(response) => {
            println!("\n✅ Verification Result: {}", response.result);
            if let Some(valid_until) = &response.valid_until {
                println!("📅 Valid Until: {}", valid_until);
            }
            if let Some(manifest_hash) = &response.manifest_hash {
                println!("🔐 Manifest Hash: {}", manifest_hash);
            }
            if let Some(trace) = &response.trace {
                println!("📊 Trace: {}", serde_json::to_string_pretty(trace)?);
            }

            // TODO Week 2: Writeback to SAP Z-Table
            println!("\n⏭️  Writeback to SAP (TODO: Week 2)");
        }
        Err(e) => {
            eprintln!("\n❌ Verification failed: {}", e);
            std::process::exit(1);
        }
    }

    println!("\n✅ Done!");
    Ok(())
}
