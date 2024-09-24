use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, SystemTime};
use tracing::{debug, trace, warn};

use crate::common::types::GatewayLatencyStats;
use crate::gateway::config::ServiceConfig;

#[derive(Debug, Clone)]
pub struct GatewayToServiceStats {
    pub latency: Duration,
    pub last_updated: SystemTime,
    pub service_config: ServiceConfig,
}

#[derive(Debug, Clone)]
pub struct GatewayToGatewayStats {
    pub latency: Duration,
    pub last_updated: SystemTime,
}

#[derive(Debug, Clone)]
pub struct OptimalPath {
    pub gateway: String,
    pub latency: Duration,
    pub last_updated: SystemTime,
}

/*
This module defines the data structures and methods for managing and updating statistics related to gateways and services in a network.
The main structure, `Store`, holds three types of data:
1. `gateway_to_service`: A mapping from gateway IDs to service IDs, which contains statistics about the latency and last update time for each service.
2. `gateway_to_gateway`: A mapping from gateway IDs to other gateway IDs, which contains statistics about the latency and last update time for each gateway-to-gateway connection.
3. `optimal_paths`: A mapping from service IDs to the optimal gateway and the associated latency for reaching that service.

The `Store` struct provides several methods for updating and retrieving these statistics:
- `new`: Creates a new, empty `Store`.
- `update_gateway_to_service_stats`: Updates the statistics for a given gateway and its associated services.
- `update_gateway_to_gateway_stats`: Updates the statistics for a given gateway-to-gateway connection.
- `update_optimal_path`: Calculates and updates the optimal gateway for a given service.
- `get_optimal_service_path`: Retrieves the optimal gateway and latency for a given service.
- `get_gateway_to_service_stats`: Retrieves the statistics for a given gateway and service.
- `get_gateway_to_gateway_stats`: Retrieves the statistics for a given gateway-to-gateway connection.

When `get_optimal_service_path` is called, it returns an `Option` containing a tuple with two elements:
1. A `String` representing the gateway ID that has the optimal path to the service.
2. A `Duration` representing the total latency for the optimal path.

If no optimal path is found, the method returns `None`.
*/
#[derive(Debug)]
pub struct Store {
    // Gateway ID -> Service ID -> Stats
    gateway_to_service: RwLock<HashMap<String, HashMap<String, GatewayToServiceStats>>>,
    // Gateway ID -> Gateway ID -> Stats
    gateway_to_gateway: RwLock<HashMap<String, HashMap<String, GatewayToGatewayStats>>>,
    // Service ID -> Optimal Path
    optimal_paths: RwLock<HashMap<String, OptimalPath>>,
}

impl Store {
    pub fn new() -> Self {
        trace!("creating new store");
        Store {
            gateway_to_service: RwLock::new(HashMap::new()),
            gateway_to_gateway: RwLock::new(HashMap::new()),
            optimal_paths: RwLock::new(HashMap::new()),
        }
    }

    pub fn update_gateway_to_service_stats(
        &self,
        stats: GatewayLatencyStats,
        service_configs: &HashMap<String, ServiceConfig>,
    ) {
        trace!(
            "updating gateway-to-service stats for gateway: {}",
            stats.gateway_id
        );
        let mut gateway_to_service = self.gateway_to_service.write().unwrap();
        let gateway_stats = gateway_to_service
            .entry(stats.gateway_id.clone())
            .or_insert_with(HashMap::new);

        let affected_services: Vec<String> = stats.stats.keys().cloned().collect();

        for (service_id, service_stat) in stats.stats {
            if let Some(service_config) = service_configs.get(&service_id) {
                gateway_stats.insert(
                    service_id.clone(),
                    GatewayToServiceStats {
                        latency: service_stat.latency,
                        last_updated: SystemTime::now(),
                        service_config: service_config.clone(),
                    },
                );
                trace!(
                    "updated stats for service: {} with latency: {:?}",
                    service_id,
                    service_stat.latency
                );
            } else {
                warn!("service config not found for service: {}", service_id);
            }
        }
        drop(gateway_to_service);

        // Update optimal paths for all affected services
        for service_id in affected_services {
            self.update_optimal_path(&service_id);
        }
    }

    pub fn update_gateway_to_gateway_stats(
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
        let mut gateway_to_gateway = self.gateway_to_gateway.write().unwrap();
        let gateway_stats = gateway_to_gateway
            .entry(from_gateway.clone())
            .or_insert_with(HashMap::new);

        gateway_stats.insert(
            to_gateway.clone(),
            GatewayToGatewayStats {
                latency,
                last_updated: SystemTime::now(),
            },
        );
        drop(gateway_to_gateway);

        debug!(
            "updated gateway-to-gateway stats: {} -> {} with latency: {:?}",
            from_gateway, to_gateway, latency
        );

        // Update optimal paths for all services
        let service_ids: Vec<String> = self.optimal_paths.read().unwrap().keys().cloned().collect();
        for service_id in service_ids {
            self.update_optimal_path(&service_id);
        }
    }

    fn update_optimal_path(&self, service_id: &str) {
        trace!("updating optimal path for service: {}", service_id);
        if let Some((gateway, latency)) = self.calculate_optimal_service_path(service_id) {
            let mut optimal_paths = self.optimal_paths.write().unwrap();
            optimal_paths.insert(
                service_id.to_string(),
                OptimalPath {
                    gateway: gateway.clone(),
                    latency,
                    last_updated: SystemTime::now(),
                },
            );
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

    pub fn get_optimal_service_path(&self, service_id: &str) -> Option<(String, Duration)> {
        trace!("getting optimal service path for service: {}", service_id);
        self.optimal_paths
            .read()
            .unwrap()
            .get(service_id)
            .map(|optimal_path| {
                trace!(
                    "found optimal path for service: {}. gateway: {}, latency: {:?}",
                    service_id,
                    optimal_path.gateway,
                    optimal_path.latency
                );
                (optimal_path.gateway.clone(), optimal_path.latency)
            })
    }

    fn calculate_optimal_service_path(&self, service_id: &str) -> Option<(String, Duration)> {
        trace!(
            "calculating optimal service path for service: {}",
            service_id
        );
        let mut best_gateway = None;
        let mut best_latency = Duration::MAX;

        let gateway_to_service = self.gateway_to_service.read().unwrap();
        let gateway_to_gateway = self.gateway_to_gateway.read().unwrap();

        // Check all gateways for direct connections
        for (gateway_id, services) in &*gateway_to_service {
            if let Some(service_stats) = services.get(service_id) {
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
        }

        // Check paths through other gateways
        for (from_gateway, gateway_stats) in &*gateway_to_gateway {
            for (intermediate_gateway, gateway_to_gateway_stats) in gateway_stats {
                if let Some(service_stats) = gateway_to_service
                    .get(intermediate_gateway)
                    .and_then(|services| services.get(service_id))
                {
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
        }

        best_gateway.map(|gateway| {
            debug!(
                "calculated optimal path for service: {}. gateway: {}, latency: {:?}",
                service_id, gateway, best_latency
            );
            (gateway, best_latency)
        })
    }

    pub fn get_gateway_to_service_stats(
        &self,
        gateway_id: &str,
        service_id: &str,
    ) -> Option<GatewayToServiceStats> {
        trace!(
            "getting gateway-to-service stats for gateway: {}, service: {}",
            gateway_id,
            service_id
        );
        self.gateway_to_service
            .read()
            .unwrap()
            .get(gateway_id)
            .and_then(|services| services.get(service_id).cloned())
            .map(|stats| {
                trace!(
                    "found gateway-to-service stats. latency: {:?}",
                    stats.latency
                );
                stats
            })
    }

    pub fn get_gateway_to_gateway_stats(
        &self,
        from_gateway: &str,
        to_gateway: &str,
    ) -> Option<GatewayToGatewayStats> {
        trace!(
            "getting gateway-to-gateway stats for: {} -> {}",
            from_gateway,
            to_gateway
        );
        self.gateway_to_gateway
            .read()
            .unwrap()
            .get(from_gateway)
            .and_then(|gateways| gateways.get(to_gateway).cloned())
            .map(|stats| {
                trace!(
                    "found gateway-to-gateway stats. latency: {:?}",
                    stats.latency
                );
                stats
            })
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        common::types::ServiceStatus,
        gateway::config::{HealthCheckConfig, HealthCheckType},
    };

    use super::*;
    use std::time::Duration;

    #[test]
    fn test_new_store() {
        let store = Store::new();
        assert!(store.gateway_to_service.read().unwrap().is_empty());
        assert!(store.gateway_to_gateway.read().unwrap().is_empty());
        assert!(store.optimal_paths.read().unwrap().is_empty());
    }

    #[test]
    fn test_update_gateway_to_service_stats() {
        let store = Store::new();
        let mut stats = GatewayLatencyStats {
            gateway_id: "gateway1".to_string(),
            stats: HashMap::new(),
        };
        stats.stats.insert(
            "service1".to_string(),
            crate::common::types::ServiceStat {
                latency: Duration::from_secs(1),
                service_id: "service1".to_string(),
                status: ServiceStatus::Up,
                error: None,
            },
        );
        let mut service_configs = HashMap::new();
        service_configs.insert(
            "service1".to_string(),
            ServiceConfig {
                id: "service1".to_string(),
                address: "localhost".to_string(),
                port: 8080,
                health_check: HealthCheckConfig {
                    r#type: HealthCheckType::Http,
                    url: Some("http://localhost:8080/health".to_string()),
                    interval: Duration::from_secs(5),
                    timeout: Duration::from_secs(2),
                },
                // Add other necessary fields
            },
        );

        store.update_gateway_to_service_stats(stats, &service_configs);

        let gateway_to_service = store.gateway_to_service.read().unwrap();
        assert!(gateway_to_service.contains_key("gateway1"));
        assert!(gateway_to_service["gateway1"].contains_key("service1"));
        assert_eq!(
            gateway_to_service["gateway1"]["service1"].latency,
            Duration::from_secs(1)
        );
    }

    #[test]
    fn test_update_gateway_to_gateway_stats() {
        let store = Store::new();
        store.update_gateway_to_gateway_stats(
            "gateway1".to_string(),
            "gateway2".to_string(),
            Duration::from_secs(2),
        );

        let gateway_to_gateway = store.gateway_to_gateway.read().unwrap();
        assert!(gateway_to_gateway.contains_key("gateway1"));
        assert!(gateway_to_gateway["gateway1"].contains_key("gateway2"));
        assert_eq!(
            gateway_to_gateway["gateway1"]["gateway2"].latency,
            Duration::from_secs(2)
        );
    }

    #[test]
    fn test_get_optimal_service_path() {
        let store = Store::new();
        let mut optimal_paths = store.optimal_paths.write().unwrap();
        optimal_paths.insert(
            "service1".to_string(),
            OptimalPath {
                gateway: "gateway1".to_string(),
                latency: Duration::from_secs(3),
                last_updated: SystemTime::now(),
            },
        );
        drop(optimal_paths);

        let result = store.get_optimal_service_path("service1");
        assert!(result.is_some());
        let (gateway, latency) = result.unwrap();
        assert_eq!(gateway, "gateway1".to_string());
        assert_eq!(latency, Duration::from_secs(3));
    }

    #[test]
    fn test_get_gateway_to_service_stats() {
        let store = Store::new();
        let mut gateway_to_service = store.gateway_to_service.write().unwrap();
        let mut services = HashMap::new();
        services.insert(
            "service1".to_string(),
            GatewayToServiceStats {
                latency: Duration::from_secs(1),
                last_updated: SystemTime::now(),
                service_config: ServiceConfig {
                    id: "service1".to_string(),
                    address: "localhost".to_string(),
                    port: 8080,
                    health_check: HealthCheckConfig {
                        r#type: HealthCheckType::Http,
                        url: Some("http://localhost:8080/health".to_string()),
                        interval: Duration::from_secs(5),
                        timeout: Duration::from_secs(2),
                    },
                },
            },
        );
        gateway_to_service.insert("gateway1".to_string(), services);
        drop(gateway_to_service);

        let result = store.get_gateway_to_service_stats("gateway1", "service1");
        assert!(result.is_some());
        let stats = result.unwrap();
        assert_eq!(stats.latency, Duration::from_secs(1));
    }

    #[test]
    fn test_get_gateway_to_gateway_stats() {
        let store = Store::new();
        let mut gateway_to_gateway = store.gateway_to_gateway.write().unwrap();
        let mut gateways = HashMap::new();
        gateways.insert(
            "gateway2".to_string(),
            GatewayToGatewayStats {
                latency: Duration::from_secs(2),
                last_updated: SystemTime::now(),
            },
        );
        gateway_to_gateway.insert("gateway1".to_string(), gateways);
        drop(gateway_to_gateway);

        let result = store.get_gateway_to_gateway_stats("gateway1", "gateway2");
        assert!(result.is_some());
        let stats = result.unwrap();
        assert_eq!(stats.latency, Duration::from_secs(2));
    }
}
