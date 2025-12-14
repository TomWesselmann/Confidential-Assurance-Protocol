//! SAP Adapter - v0.3.0: Production-Ready OData v4 Integration
//!
//! Implements CAP Engineering Guide Section 9.2:
//! 1. OData fetch Funktion erstellen      (done)
//! 2. Sanitizer definieren                 (done)
//! 3. Mapping-Funktion schreiben           (done)
//! 4. Merkle Root Berechnung ergänzen      (integrated in mapper)
//! 5. context.json erweitern               (done)
//! 6. Tests schreiben                      (module tests)
//! 7. CLI Command ergänzen                 (done)
//!
//! ## CLI Usage
//! ```bash
//! sap-adapter --mode mock --output context.json
//! sap-adapter --mode odata --sap-url <URL> --sap-user <USER> --sap-pass <PASS>
//! ```

use anyhow::{Context as AnyhowContext, Result};
use clap::{Parser, ValueEnum};
use sap_adapter::{
    mapper::{map_to_cap_context, CapContext},
    odata_client::{ODataClient, ODataConfig, SapBusinessPartner},
    sanitizer::sanitize_batch,
};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "sap-adapter")]
#[command(version = "0.3.0")]
#[command(about = "SAP S/4HANA to CAP context adapter with OData v4 support")]
struct Cli {
    /// Operation mode: mock (JSON file) or odata (SAP S/4HANA)
    #[arg(short, long, value_enum, default_value = "mock")]
    mode: Mode,

    /// Mock JSON file path (for --mode mock)
    #[arg(long, default_value = "examples/suppliers.json")]
    mock_file: PathBuf,

    /// SAP OData base URL (for --mode odata)
    /// Can also be set via SAP_ODATA_URL environment variable
    #[arg(long)]
    sap_url: Option<String>,

    /// SAP username (for --mode odata)
    /// Can also be set via SAP_USER environment variable
    #[arg(long)]
    sap_user: Option<String>,

    /// SAP password (for --mode odata)
    /// Can also be set via SAP_PASS environment variable
    #[arg(long)]
    sap_pass: Option<String>,

    /// OData $filter query (e.g., "Country eq 'DE'")
    #[arg(long)]
    filter: Option<String>,

    /// OData $top limit (max 1000)
    #[arg(long)]
    top: Option<u32>,

    /// Accept self-signed TLS certificates (dev only)
    #[arg(long)]
    accept_invalid_certs: bool,

    /// Output context.json file path
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Enable verbose logging (DEBUG level)
    #[arg(short, long)]
    verbose: bool,

    /// Dry run: Show context without writing to file
    #[arg(long)]
    dry_run: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Mode {
    /// Load data from mock JSON file
    Mock,
    /// Fetch data from SAP S/4HANA via OData v4
    Odata,
}

/// Mock data structure (for JSON files)
#[derive(Debug, Deserialize)]
struct MockData {
    suppliers: Vec<SapBusinessPartner>,
}

/// Main entry point
#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    let log_level = if cli.verbose { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::new(log_level))
        .init();

    tracing::info!("🧩 SAP Adapter v0.3.0 (CAP Engineering Guide compliant)");

    // Fetch data based on mode
    let raw_partners = match cli.mode {
        Mode::Mock => fetch_mock_data(&cli.mock_file)?,
        Mode::Odata => fetch_odata_data(&cli).await?,
    };

    tracing::info!("✅ Loaded {} business partners", raw_partners.len());

    // Step 2: Sanitize input
    let (sanitized_partners, errors) = sanitize_batch(&raw_partners);

    if !errors.is_empty() {
        tracing::warn!("⚠️  {} sanitization errors:", errors.len());
        for (id, error) in &errors {
            tracing::warn!("  - Business Partner {}: {}", id, error);
        }
    }

    tracing::info!("🔐 Sanitized {} suppliers (dropped {} invalid)",
                   sanitized_partners.len(), errors.len());

    // Step 3+4: Map to CAP context with BLAKE3 hashing
    let timestamp = chrono::Utc::now().to_rfc3339();
    let source = match cli.mode {
        Mode::Mock => format!("Mock: {}", cli.mock_file.display()),
        Mode::Odata => cli.sap_url.clone().unwrap_or_else(|| "SAP S/4HANA".to_string()),
    };

    let cap_context = map_to_cap_context(&sanitized_partners, &source, &timestamp);

    tracing::info!("✅ Generated CAP context with {} hashed suppliers", cap_context.total_count);

    // Step 5: Output context.json
    if cli.dry_run {
        println!("\n📄 CAP Context (Dry Run):");
        println!("{}", serde_json::to_string_pretty(&cap_context)?);
    }

    if let Some(output_path) = &cli.output {
        write_context_json(&cap_context, output_path)?;
        tracing::info!("💾 Saved to: {}", output_path.display());
    }

    // Summary
    print_summary(&cap_context, &errors);

    Ok(())
}

/// Fetch data from mock JSON file
fn fetch_mock_data(path: &PathBuf) -> Result<Vec<SapBusinessPartner>> {
    tracing::info!("📂 Reading mock data from: {}", path.display());

    let file = std::fs::File::open(path)
        .with_context(|| format!("Failed to open mock file: {}", path.display()))?;

    let mock_data: MockData = serde_json::from_reader(file)
        .context("Failed to parse mock JSON file")?;

    Ok(mock_data.suppliers)
}

/// Fetch data from SAP S/4HANA via OData v4
async fn fetch_odata_data(cli: &Cli) -> Result<Vec<SapBusinessPartner>> {
    // Try CLI args first, then environment variables
    let base_url = cli.sap_url.clone()
        .or_else(|| std::env::var("SAP_ODATA_URL").ok())
        .ok_or_else(|| anyhow::anyhow!("--sap-url or SAP_ODATA_URL required for --mode odata"))?;

    let username = cli.sap_user.clone()
        .or_else(|| std::env::var("SAP_USER").ok())
        .ok_or_else(|| anyhow::anyhow!("--sap-user or SAP_USER required for --mode odata"))?;

    let password = cli.sap_pass.clone()
        .or_else(|| std::env::var("SAP_PASS").ok())
        .ok_or_else(|| anyhow::anyhow!("--sap-pass or SAP_PASS required for --mode odata"))?;

    let config = ODataConfig {
        base_url: base_url.clone(),
        username: username.clone(),
        password: password.clone(),
        timeout_secs: 30,
        accept_invalid_certs: cli.accept_invalid_certs,
    };

    tracing::info!("📡 Connecting to SAP OData: {}", base_url);

    let client = ODataClient::new(config)
        .context("Failed to create OData client")?;

    // Health check
    if let Err(e) = client.health_check().await {
        tracing::warn!("⚠️  OData health check failed: {}", e);
    }

    // Fetch business partners
    let partners = client
        .fetch_business_partners(cli.filter.as_deref(), cli.top)
        .await
        .context("Failed to fetch business partners from SAP")?;

    Ok(partners)
}

/// Write CAP context to JSON file
fn write_context_json(context: &CapContext, path: &PathBuf) -> Result<()> {
    let json = serde_json::to_string_pretty(context)
        .context("Failed to serialize context to JSON")?;

    std::fs::write(path, json)
        .with_context(|| format!("Failed to write to: {}", path.display()))?;

    Ok(())
}

/// Print execution summary
fn print_summary(context: &CapContext, errors: &[(String, String)]) {
    println!("\n=== Summary ===");
    println!("  Schema Version:   {}", context.schema_version);
    println!("  Source:           {}", context.metadata.source);
    println!("  Extracted At:     {}", context.metadata.extracted_at);
    println!("  Adapter Version:  {}", context.metadata.adapter_version);
    println!("  Total Suppliers:  {}", context.total_count);
    println!("  Sanitization Errors: {}", errors.len());

    if !context.suppliers.is_empty() {
        println!("\n  First Supplier:");
        println!("    ID Hash:      {}", context.suppliers[0].id_hash);
        println!("    Country:      {}", context.suppliers[0].country);
        println!("    Tier:         {}", context.suppliers[0].tier);
    }

    println!("\n✅ Done!");
}
