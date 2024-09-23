use std::collections::HashMap;
use std::time::Instant;

use super::config::{GatewayConfig, ServiceConfig};
use super::latency::get_service_latency;
use super::store::Store;
use crate::common::types::{GatewayLatencyStats, TransportType};
use crate::gateway::config::read_gateway_config;
use crate::transport::pubsub::PubSubManager;
use crate::transport::topics::PubSubTopics;
use crate::transport::{self};

struct Service {
    id: String,
    service_config: ServiceConfig,
    last_heartbeat: Instant,
    last_latency: Instant,
}

pub struct Gateway {
    id: String,
    gateway_config: GatewayConfig,
    services: HashMap<String, Service>,
    transport: PubSubManager<transport::nats::NatsPubSub>,
    store: Store,
}

impl Gateway {
    pub async fn new(conf: &GatewayConfig) -> Self {
        let transport = match conf.gateway.transport.transport_type {
            TransportType::Nats => {
                let nats_config = conf
                    .gateway
                    .transport
                    .nats
                    .clone()
                    .expect("NATS configuration missing");
                transport::nats::NatsPubSub::new(nats_config)
                    .await
                    .expect("Failed to create NATS PubSub")
            }
            _ => panic!("Invalid transport type"),
        };

        let manager = PubSubManager::new(transport);

        let services = conf
            .gateway
            .services
            .iter()
            .map(|service| {
                (
                    service.id.clone(),
                    Service {
                        id: service.id.clone(),
                        service_config: service.clone(),
                        last_heartbeat: Instant::now(),
                        last_latency: Instant::now(),
                    },
                )
            })
            .collect();

        Gateway {
            id: conf.gateway.name.clone(),
            gateway_config: conf.clone(),
            transport: manager,
            store: Store::new(),
            services,
        }
    }

    pub async fn run(&self) {
        self.start_sending_stats().await;
    }

    async fn start_sending_stats(&self) {
        let mut interval = tokio::time::interval(self.gateway_config.gateway.latency.interval);

        loop {
            let latencies = futures_util::future::join_all(
                self.services
                    .iter()
                    .map(|service| get_service_latency(&service.1.service_config)),
            )
            .await;

            let mut stats = GatewayLatencyStats::new();

            for stat in latencies {
                stats.insert(stat.service_id.clone(), stat);
            }

            self.transport
                .broadcast(
                    &[PubSubTopics::PublishGatewayLatencyStats],
                    transport::pubsub::Message::GatewayLatencyStats(stats),
                )
                .await
                .expect("Failed to broadcast gateway latency stats");

            interval.tick().await;
        }
    }
}
