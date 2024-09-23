// This file defines a Store struct that manages the state of services and their associated gateways.
// The Store struct contains a HashMap of services, where each service has a unique ID and contains
// a collection of gateways. Each gateway has an ID, name, latency, status, and a timestamp of the last update.

use std::collections::HashMap;
use std::time::{Duration, SystemTime};

#[derive(Debug, Clone)]
pub struct GatewayStats {
    id: String,
    latency: u64,
    last_updated: SystemTime,
}

#[derive(Debug, Clone)]
pub struct ServiceStats {
    id: String,
    gateways: HashMap<String, GatewayStats>,
}

pub struct Store {
    services: HashMap<String, ServiceStats>,
}

impl Store {
    pub fn new() -> Self {
        Store {
            services: HashMap::new(),
        }
    }

    pub fn update_service_gateways(&mut self, service_id: &str, gateways: Vec<GatewayStats>) {
        let service = self
            .services
            .entry(service_id.to_string())
            .or_insert_with(|| ServiceStats {
                id: service_id.to_string(),
                gateways: HashMap::new(),
            });

        for gateway in gateways {
            service.gateways.insert(gateway.id.clone(), gateway);
        }
    }

    pub fn get_service_with_lowest_latency(&self, service_id: &str) -> Option<&GatewayStats> {
        self.services.get(service_id).and_then(|service| {
            service
                .gateways
                .values()
                .min_by_key(|gateway| gateway.latency)
        })
    }

    pub fn add_service(&mut self, service: ServiceStats) {
        self.services.insert(service.id.clone(), service);
    }

    pub fn get_service(&self, service_id: &str) -> Option<&ServiceStats> {
        self.services.get(service_id)
    }

    pub fn update_gateway_status(
        &mut self,
        service_id: &str,
        gateway_id: &str,
    ) -> Result<(), String> {
        if let Some(service) = self.services.get_mut(service_id) {
            if let Some(gateway) = service.gateways.get_mut(gateway_id) {
                gateway.last_updated = SystemTime::now();
                return Ok(());
            }
        }
        Err(format!(
            "Gateway with id {} not found in service {}",
            gateway_id, service_id
        ))
    }

    pub fn remove_stale_gateways(&mut self, max_age: Duration) {
        let now = SystemTime::now();
        for service in self.services.values_mut() {
            service.gateways.retain(|_, gateway| {
                now.duration_since(gateway.last_updated)
                    .unwrap_or_else(|_| Duration::new(0, 0))
                    < max_age
            });
        }
    }
}
