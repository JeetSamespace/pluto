use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, SystemTime};

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
    pub path: Vec<String>,
    pub latency: Duration,
    pub last_updated: SystemTime,
}

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
        let mut gateway_to_service = self.gateway_to_service.write().unwrap();
        let gateway_stats = gateway_to_service
            .entry(stats.gateway_id.clone())
            .or_insert_with(HashMap::new);

        let affected_services: Vec<String> = stats.stats.keys().cloned().collect();

        for (service_id, service_stat) in stats.stats {
            if let Some(service_config) = service_configs.get(&service_id) {
                gateway_stats.insert(
                    service_id,
                    GatewayToServiceStats {
                        latency: service_stat.latency,
                        last_updated: SystemTime::now(),
                        service_config: service_config.clone(),
                    },
                );
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
        let mut gateway_to_gateway = self.gateway_to_gateway.write().unwrap();
        let gateway_stats = gateway_to_gateway
            .entry(from_gateway)
            .or_insert_with(HashMap::new);

        gateway_stats.insert(
            to_gateway,
            GatewayToGatewayStats {
                latency,
                last_updated: SystemTime::now(),
            },
        );
        drop(gateway_to_gateway);

        // Update optimal paths for all services
        let service_ids: Vec<String> = self.optimal_paths.read().unwrap().keys().cloned().collect();
        for service_id in service_ids {
            self.update_optimal_path(&service_id);
        }
    }

    fn update_optimal_path(&self, service_id: &str) {
        if let Some((path, latency)) = self.calculate_optimal_service_path(service_id) {
            let mut optimal_paths = self.optimal_paths.write().unwrap();
            optimal_paths.insert(
                service_id.to_string(),
                OptimalPath {
                    path,
                    latency,
                    last_updated: SystemTime::now(),
                },
            );
        }
    }

    pub fn get_optimal_service_path(&self, service_id: &str) -> Option<(Vec<String>, Duration)> {
        self.optimal_paths
            .read()
            .unwrap()
            .get(service_id)
            .map(|optimal_path| (optimal_path.path.clone(), optimal_path.latency))
    }

    fn calculate_optimal_service_path(&self, service_id: &str) -> Option<(Vec<String>, Duration)> {
        let mut best_path = None;
        let mut best_latency = Duration::MAX;

        let gateway_to_service = self.gateway_to_service.read().unwrap();
        let gateway_to_gateway = self.gateway_to_gateway.read().unwrap();

        // Check all gateways for direct connections
        for (gateway_id, services) in &*gateway_to_service {
            if let Some(service_stats) = services.get(service_id) {
                if service_stats.latency < best_latency {
                    best_path = Some(vec![gateway_id.clone()]);
                    best_latency = service_stats.latency;
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
                        best_path = Some(vec![from_gateway.clone(), intermediate_gateway.clone()]);
                    }
                }
            }
        }

        best_path.map(|path| (path, best_latency))
    }

    pub fn get_gateway_to_service_stats(
        &self,
        gateway_id: &str,
        service_id: &str,
    ) -> Option<GatewayToServiceStats> {
        self.gateway_to_service
            .read()
            .unwrap()
            .get(gateway_id)
            .and_then(|services| services.get(service_id).cloned())
    }

    pub fn get_gateway_to_gateway_stats(
        &self,
        from_gateway: &str,
        to_gateway: &str,
    ) -> Option<GatewayToGatewayStats> {
        self.gateway_to_gateway
            .read()
            .unwrap()
            .get(from_gateway)
            .and_then(|gateways| gateways.get(to_gateway).cloned())
    }
}
