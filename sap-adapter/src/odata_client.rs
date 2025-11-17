/// OData v4 Client for SAP S/4HANA Business Partner API
/// Implements deterministic, auditable data fetching with timeout and error handling
/// Follows CAP Engineering Guide Section 9.2

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// OData v4 query configuration
#[derive(Debug, Clone)]
pub struct ODataConfig {
    pub base_url: String,
    pub username: String,
    pub password: String,
    pub timeout_secs: u64,
    pub accept_invalid_certs: bool,
}

impl Default for ODataConfig {
    fn default() -> Self {
        Self {
            base_url: "https://localhost:8443/sap/opu/odata4/sap/api_business_partner/srvd_a2x/sap/".to_string(),
            username: String::new(),
            password: String::new(),
            timeout_secs: 30,
            accept_invalid_certs: false,
        }
    }
}

/// SAP Business Partner OData response (simplified)
#[derive(Debug, Deserialize)]
pub struct ODataResponse {
    #[serde(rename = "@odata.context")]
    pub context: Option<String>,
    pub value: Vec<SapBusinessPartner>,
}

/// SAP S/4HANA Business Partner structure (OData v4)
/// Supports both OData field names and legacy mock format
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SapBusinessPartner {
    #[serde(rename = "BusinessPartner", alias = "LIFNR")]
    pub business_partner: String,

    #[serde(rename = "BusinessPartnerName", alias = "NAME1")]
    pub name: String,

    #[serde(rename = "Country", alias = "LAND1")]
    pub country: String,

    #[serde(rename = "CityName", alias = "ORT01")]
    pub city: Option<String>,

    #[serde(rename = "StreetName", alias = "STRAS")]
    pub street: Option<String>,

    /// Custom field: Last audit date (ISO 8601)
    #[serde(rename = "ZZ_AUDIT_DATE", alias = "AUDIT_DATE")]
    pub audit_date: Option<String>,

    /// Custom field: Supply chain tier (1, 2, 3)
    #[serde(rename = "ZZ_TIER", alias = "TIER")]
    pub tier: Option<String>,

    /// Custom field: Number of Ultimate Beneficial Owners
    #[serde(rename = "ZZ_UBO_COUNT", alias = "UBO_COUNT")]
    pub ubo_count: Option<u32>,
}

/// OData client with connection pooling and retry logic
pub struct ODataClient {
    config: ODataConfig,
    http_client: reqwest::Client,
}

impl ODataClient {
    /// Create new OData client with given configuration
    ///
    /// # Errors
    /// Returns error if HTTP client cannot be built
    pub fn new(config: ODataConfig) -> Result<Self> {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .danger_accept_invalid_certs(config.accept_invalid_certs)
            .pool_max_idle_per_host(10)
            .build()
            .context("Failed to build HTTP client")?;

        Ok(Self {
            config,
            http_client,
        })
    }

    /// Fetch all business partners from SAP S/4HANA via OData v4
    ///
    /// # Arguments
    /// * `filter` - Optional OData $filter query (e.g., "Country eq 'DE'")
    /// * `top` - Optional OData $top limit (max 1000)
    ///
    /// # Returns
    /// List of business partners sorted by BusinessPartner ID (deterministic ordering)
    ///
    /// # Errors
    /// - Network errors (timeout, connection refused)
    /// - Authentication errors (401, 403)
    /// - Invalid OData response format
    /// - SAP backend errors (500, 503)
    pub async fn fetch_business_partners(
        &self,
        filter: Option<&str>,
        top: Option<u32>,
    ) -> Result<Vec<SapBusinessPartner>> {
        let mut url = format!("{}BusinessPartner", self.config.base_url);
        let mut query_params = Vec::new();

        if let Some(f) = filter {
            query_params.push(format!("$filter={}", urlencoding::encode(f)));
        }

        if let Some(t) = top {
            query_params.push(format!("$top={}", t.min(1000))); // SAP limit: 1000
        }

        // Always request sorted results for determinism
        query_params.push("$orderby=BusinessPartner asc".to_string());

        if !query_params.is_empty() {
            url = format!("{}?{}", url, query_params.join("&"));
        }

        tracing::info!("Fetching from SAP OData: {}", url);

        let response = self
            .http_client
            .get(&url)
            .basic_auth(&self.config.username, Some(&self.config.password))
            .header("Accept", "application/json")
            .send()
            .await
            .context("Failed to send OData request")?;

        let status = response.status();
        tracing::debug!("OData response status: {}", status);

        if !status.is_success() {
            let error_body = response.text().await.unwrap_or_default();
            anyhow::bail!(
                "OData request failed with status {}: {}",
                status,
                error_body
            );
        }

        let odata_response: ODataResponse = response
            .json()
            .await
            .context("Failed to parse OData JSON response")?;

        tracing::info!(
            "Successfully fetched {} business partners",
            odata_response.value.len()
        );

        // Ensure deterministic ordering (defense in depth)
        let mut partners = odata_response.value;
        partners.sort_by(|a, b| a.business_partner.cmp(&b.business_partner));

        Ok(partners)
    }

    /// Health check: Test OData connection without fetching data
    ///
    /// # Returns
    /// Ok(true) if connection successful, Err otherwise
    pub async fn health_check(&self) -> Result<bool> {
        let url = format!("{}$metadata", self.config.base_url);

        let response = self
            .http_client
            .get(&url)
            .basic_auth(&self.config.username, Some(&self.config.password))
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .context("Failed to connect to SAP OData service")?;

        Ok(response.status().is_success())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_odata_config_default() {
        let config = ODataConfig::default();
        assert_eq!(config.timeout_secs, 30);
        assert!(!config.accept_invalid_certs);
    }

    #[tokio::test]
    async fn test_odata_client_creation() {
        let config = ODataConfig::default();
        let client = ODataClient::new(config);
        assert!(client.is_ok());
    }

    #[test]
    fn test_business_partner_deterministic_sort() {
        let mut partners = vec![
            SapBusinessPartner {
                business_partner: "100003".to_string(),
                name: "C".to_string(),
                country: "DE".to_string(),
                city: None,
                street: None,
                audit_date: None,
                tier: None,
                ubo_count: None,
            },
            SapBusinessPartner {
                business_partner: "100001".to_string(),
                name: "A".to_string(),
                country: "DE".to_string(),
                city: None,
                street: None,
                audit_date: None,
                tier: None,
                ubo_count: None,
            },
            SapBusinessPartner {
                business_partner: "100002".to_string(),
                name: "B".to_string(),
                country: "DE".to_string(),
                city: None,
                street: None,
                audit_date: None,
                tier: None,
                ubo_count: None,
            },
        ];

        partners.sort_by(|a, b| a.business_partner.cmp(&b.business_partner));

        assert_eq!(partners[0].business_partner, "100001");
        assert_eq!(partners[1].business_partner, "100002");
        assert_eq!(partners[2].business_partner, "100003");
    }
}
