use crate::common::{types::TransportConfig, utils::handle_duration_string};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Deserialize)]
pub struct GatewayConfig {
    pub gateway: Gateway,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Gateway {
    pub name: String,
    pub region: String,
    pub listen_port: u16,

    pub services: Vec<ServiceConfig>,
    pub transport: TransportConfig,
    pub latency: LatencyConfig,
    pub heartbeat: HeartbeatConfig,
    pub failover: FailoverConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServiceConfig {
    pub id: String,
    pub address: String,
    pub port: u16,
    pub health_check: HealthCheckConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HealthCheckConfig {
    pub r#type: HealthCheckType,
    #[serde(with = "handle_duration_string")]
    pub interval: Duration,
    #[serde(with = "handle_duration_string")]
    pub timeout: Duration,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthCheckType {
    Tcp,
    Http,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LatencyConfig {
    #[serde(with = "handle_duration_string")]
    pub interval: Duration,
    #[serde(with = "handle_duration_string")]
    pub timeout: Duration,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HeartbeatConfig {
    #[serde(with = "handle_duration_string")]
    pub interval: Duration,
    pub retries: u32,
    #[serde(with = "handle_duration_string")]
    pub timeout: Duration,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FailoverConfig {
    pub retries: u32,
    #[serde(with = "handle_duration_string")]
    pub interval: Duration,
}

pub fn read_gateway_config() -> Result<GatewayConfig, Box<dyn std::error::Error>> {
    let config_path =
        std::env::var("GATEWAY_CONFIG_PATH").unwrap_or_else(|_| "config-gateway.hcl".to_string());
    let config_data = std::fs::read_to_string(&config_path)?;
    let config: GatewayConfig =
        hcl::from_str(&config_data).map_err(|e| format!("Failed to parse config file: {}", e))?;
    Ok(config)
}
