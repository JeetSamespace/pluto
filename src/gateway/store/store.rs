use std::time::{Duration, SystemTime};

use crate::common::types::GatewayLatencyStats;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayToServiceStats {
    pub service_id: String,
    pub latency: Duration,
    pub last_updated: SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayToGatewayStats {
    pub latency: Duration,
    pub last_updated: SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimalPath {
    pub gateway: String,
    pub latency: Duration,
    pub last_updated: SystemTime,
}

pub trait Store: Send + Sync + std::fmt::Debug {
    fn new() -> Self
    where
        Self: Sized;
    fn update_gateway_to_service_stats(&self, stats: GatewayLatencyStats);
    fn update_gateway_to_gateway_stats(
        &self,
        from_gateway: String,
        to_gateway: String,
        latency: Duration,
    );
    fn get_optimal_service_path(&self, service_id: &str) -> Option<(String, Duration)>;
    fn get_gateway_to_service_stats(
        &self,
        gateway_id: &str,
        service_id: &str,
    ) -> Option<GatewayToServiceStats>;
    fn get_gateway_to_gateway_stats(
        &self,
        from_gateway: &str,
        to_gateway: &str,
    ) -> Option<GatewayToGatewayStats>;
}
