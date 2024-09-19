use crate::common::utils::handle_duration_string;
use serde::Deserialize;
use std::time::Duration;

#[derive(Debug, Deserialize)]
pub struct OrbitConfig {
    pub orbit: Orbit,
}

#[derive(Debug, Deserialize)]
pub struct Orbit {
    pub listen_port: u16,
    pub max_connections: u32,
    pub gateways: Vec<GatewayConfig>,
    pub heartbeat: HeartbeatConfig,
    pub load_balancing: LoadBalancingConfig,
    pub security: SecurityConfig,
    pub logging: LoggingConfig,
    pub metrics: MetricsConfig,
}

#[derive(Debug, Deserialize)]
pub struct GatewayConfig {
    pub address: String,
    pub port: u16,
}

#[derive(Debug, Deserialize)]
pub struct HeartbeatConfig {
    #[serde(with = "handle_duration_string")]
    pub interval: Duration,
    #[serde(with = "handle_duration_string")]
    pub timeout: Duration,
    pub retries: u8,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LoadBalancingMethod {
    RoundRobin,
    LeastConnections,
    Random,
    IpHash,
}

#[derive(Debug, Deserialize)]
pub struct LoadBalancingConfig {
    pub method: LoadBalancingMethod,
}

#[derive(Debug, Deserialize)]
pub struct SecurityConfig {
    pub ssl_enabled: bool,
    pub cert_file: String,
    pub key_file: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Deserialize)]
pub struct LoggingConfig {
    pub level: LogLevel,
    pub file: String,
}

#[derive(Debug, Deserialize)]
pub struct MetricsConfig {
    pub enabled: bool,
    pub endpoint: String,
}

pub fn read_orbit_config() -> Result<OrbitConfig, Box<dyn std::error::Error>> {
    let config_path =
        std::env::var("ORBIT_CONFIG_PATH").unwrap_or_else(|_| "config-orbit.hcl".to_string());
    let config_data = std::fs::read_to_string(&config_path)?;
    let config: OrbitConfig =
        hcl::from_str(&config_data).map_err(|e| format!("Failed to parse config file: {}", e))?;
    Ok(config)
}
