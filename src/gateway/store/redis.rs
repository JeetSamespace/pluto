use crate::{common::types::GatewayLatencyStats, gateway::config::ServiceConfig};
use redis::aio::ConnectionManager;
use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use tokio::sync::Mutex;
use tracing::{debug, info, trace, warn};

use super::{
    store::GatewayToGatewayStats, store::GatewayToServiceStats, store::OptimalPath, store::Store,
};

#[derive(Debug)]
pub struct RedisStore {
    client: redis::Client,
    connection_manager: Mutex<ConnectionManager>,
}

impl Store for RedisStore {
    async fn new() -> Self {
        info!("creating new redis store");
        let client = redis::Client::open("redis://127.0.0.1/").unwrap();
        let connection_manager = Mutex::new(client.get_tokio_connection_manager().await.unwrap());
        RedisStore {
            client,
            connection_manager,
        }
    }

    async fn update_gateway_to_service_stats(
        &self,
        stats: GatewayLatencyStats,
        service_configs: &HashMap<String, ServiceConfig>,
    ) {
        trace!(
            "updating gateway-to-service stats for gateway: {}",
            stats.gateway_id
        );
        let mut con = self.connection_manager.lock().await;
        let affected_services: Vec<String> = stats.stats.keys().cloned().collect();

        for (service_id, service_stat) in stats.stats {
            if let Some(service_config) = service_configs.get(&service_id) {
                let key = format!("gateway_to_service:{}:{}", stats.gateway_id, service_id);
                let value = GatewayToServiceStats {
                    latency: service_stat.latency,
                    last_updated: SystemTime::now(),
                    service_config: service_config.clone(),
                };
                let _: () = con
                    .set_ex(key, serde_json::to_string(&value).unwrap(), 3600) // Set TTL for 1 hour
                    .await
                    .unwrap();
                trace!(
                    "updated stats for service: {} with latency: {:?}",
                    service_id,
                    service_stat.latency
                );
            } else {
                warn!("service config not found for service: {}", service_id);
            }
        }

        // Update optimal paths for all affected services
        for service_id in affected_services {
            self.update_optimal_path(&service_id).await;
        }
    }

    async fn update_gateway_to_gateway_stats(
        &self,
        from_gateway: String,
        to_gateway: String,
        latency: Duration,
    ) {
        trace!(
            "updating gateway-to-gateway stats: {} -> {}",
            from_gateway,
            to_gateway
        );
        let mut con = self.connection_manager.lock().await;
        let key = format!("gateway_to_gateway:{}:{}", from_gateway, to_gateway);
        let value = GatewayToGatewayStats {
            latency,
            last_updated: SystemTime::now(),
        };
        let _: () = con
            .set_ex(key, serde_json::to_string(&value).unwrap(), 3600) // Set TTL for 1 hour
            .await
            .unwrap();

        debug!(
            "updated gateway-to-gateway stats: {} -> {} with latency: {:?}",
            from_gateway, to_gateway, latency
        );

        // Update optimal paths for all services
        let mut cursor = 0;
        loop {
            let (next_cursor, service_ids): (u64, Vec<String>) =
                con.scan_match("optimal_path:*").await.unwrap();
            cursor = next_cursor;
            for service_id in service_ids {
                self.update_optimal_path(&service_id).await;
            }
            if cursor == 0 {
                break;
            }
        }
    }

    async fn get_optimal_service_path(&self, service_id: &str) -> Option<(String, Duration)> {
        trace!("getting optimal service path for service: {}", service_id);
        let mut con = self.connection_manager.lock().await;
        let key = format!("optimal_path:{}", service_id);
        let value: String = con.get(key).await.ok()?;
        let optimal_path: OptimalPath = serde_json::from_str(&value).unwrap();
        trace!(
            "found optimal path for service: {}. gateway: {}, latency: {:?}",
            service_id,
            optimal_path.gateway,
            optimal_path.latency
        );
        Some((optimal_path.gateway, optimal_path.latency))
    }

    async fn get_gateway_to_service_stats(
        &self,
        gateway_id: &str,
        service_id: &str,
    ) -> Option<GatewayToServiceStats> {
        trace!(
            "getting gateway-to-service stats for gateway: {}, service: {}",
            gateway_id,
            service_id
        );
        let mut con = self.connection_manager.lock().await;
        let key = format!("gateway_to_service:{}:{}", gateway_id, service_id);
        let value: String = con.get(key).await.ok()?;
        let stats: GatewayToServiceStats = serde_json::from_str(&value).unwrap();
        trace!(
            "found gateway-to-service stats. latency: {:?}",
            stats.latency
        );
        Some(stats)
    }

    async fn get_gateway_to_gateway_stats(
        &self,
        from_gateway: &str,
        to_gateway: &str,
    ) -> Option<GatewayToGatewayStats> {
        trace!(
            "getting gateway-to-gateway stats for: {} -> {}",
            from_gateway,
            to_gateway
        );
        let mut con = self.connection_manager.lock().await;
        let key = format!("gateway_to_gateway:{}:{}", from_gateway, to_gateway);
        let value: String = con.get(key).await.ok()?;
        let stats: GatewayToGatewayStats = serde_json::from_str(&value).unwrap();
        trace!(
            "found gateway-to-gateway stats. latency: {:?}",
            stats.latency
        );
        Some(stats)
    }
}

impl RedisStore {
    async fn update_optimal_path(&self, service_id: &str) {
        trace!("updating optimal path for service: {}", service_id);
        if let Some((gateway, latency)) = self.calculate_optimal_service_path(service_id).await {
            let mut con = self.connection_manager.lock().await;
            let key = format!("optimal_path:{}", service_id);
            let value = OptimalPath {
                gateway: gateway.clone(),
                latency,
                last_updated: SystemTime::now(),
            };
            let _: () = con
                .set_ex(key, serde_json::to_string(&value).unwrap(), 3600) // Set TTL for 1 hour
                .await
                .unwrap();
            debug!(
                "updated optimal path for service: {}. gateway: {}, latency: {:?}",
                service_id, gateway, latency
            );
        } else {
            warn!(
                "failed to calculate optimal path for service: {}",
                service_id
            );
        }
    }

    async fn calculate_optimal_service_path(&self, service_id: &str) -> Option<(String, Duration)> {
        trace!(
            "calculating optimal service path for service: {}",
            service_id
        );
        let mut best_gateway = None;
        let mut best_latency = Duration::MAX;

        let mut con = self.connection_manager.lock().await;
        let gateway_to_service_keys: Vec<String> = con
            .scan_match(format!("gateway_to_service:*:{}", service_id))
            .await
            .unwrap();
        let gateway_to_gateway_keys: Vec<String> =
            con.scan_match("gateway_to_gateway:*").await.unwrap();

        // Check all gateways for direct connections
        for key in gateway_to_service_keys {
            let value: String = con.get(&key).await.unwrap();
            let service_stats: GatewayToServiceStats = serde_json::from_str(&value).unwrap();
            let gateway_id = key.split(':').nth(1).unwrap().to_string();
            if service_stats.latency < best_latency {
                best_gateway = Some(gateway_id.clone());
                best_latency = service_stats.latency;
                trace!(
                    "found better direct path through gateway: {}. latency: {:?}",
                    gateway_id,
                    best_latency
                );
            }
        }

        // Check paths through other gateways
        for key in gateway_to_gateway_keys {
            let value: String = con.get(&key).await.unwrap();
            let gateway_to_gateway_stats: GatewayToGatewayStats =
                serde_json::from_str(&value).unwrap();
            let from_gateway = key.split(':').nth(1).unwrap().to_string();
            let intermediate_gateway = key.split(':').nth(2).unwrap().to_string();
            let service_key = format!("gateway_to_service:{}:{}", intermediate_gateway, service_id);
            if let Ok(service_value) = con.get::<_, String>(&service_key).await {
                let service_stats: GatewayToServiceStats =
                    serde_json::from_str(&service_value).unwrap();
                let total_latency = gateway_to_gateway_stats.latency + service_stats.latency;
                if total_latency < best_latency {
                    best_latency = total_latency;
                    best_gateway = Some(intermediate_gateway.clone());
                    trace!(
                        "found better path through gateways: {} -> {}. total latency: {:?}",
                        from_gateway,
                        intermediate_gateway,
                        total_latency
                    );
                }
            }
        }

        best_gateway.map(|gateway| {
            debug!(
                "calculated optimal path for service: {}. gateway: {}, latency: {:?}",
                service_id, gateway, best_latency
            );
            (gateway, best_latency)
        })
    }
}
