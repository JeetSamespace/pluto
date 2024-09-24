use std::collections::HashMap;
use std::sync::Arc;

use super::config::{GatewayConfig, ServiceConfig};
use super::latency::get_service_latency;
use super::store::Store;
use crate::common::types::{GatewayLatencyStats, TransportType};
use crate::transport::pubsub::{Message, PubSubManager};
use crate::transport::topics::PubSubTopics;
use crate::transport::{self};

pub struct Gateway {
    id: String,
    gateway_config: GatewayConfig,
    services: HashMap<String, ServiceConfig>,
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
            .map(|service| (service.id.clone(), service.clone()))
            .collect();

        Gateway {
            id: conf.gateway.name.clone(),
            gateway_config: conf.clone(),
            transport: manager,
            store: Store::new().into(),
            services,
        }
    }

    pub async fn run(self: Arc<Self>) {
        let self_clone = Arc::clone(&self);
        tokio::spawn(async move {
            self_clone.start_sending_stats().await;
        });

        let self_clone = Arc::clone(&self);
        tokio::spawn(async move {
            self_clone.start_receiving_stats().await;
        });

        // should block until the program is terminated
        tokio::signal::ctrl_c().await.unwrap();
    }

    async fn start_receiving_stats(&self) {
        let mut rcv = self
            .transport
            .subscribe_to_topics(&[PubSubTopics::SubscribeGatewayLatencyStats])
            .await
            .expect("Failed to subscribe to topics");

        while let Some(msg) = rcv.recv().await {
            match msg {
                Message::GatewayLatencyStats(stats) => {
                    self.handle_latency_stats(stats).await;
                }
                _ => (),
            }
        }
    }

    async fn handle_latency_stats(&self, stats: GatewayLatencyStats) {
        let mut store = self.store.clone();
        store.update_gateway_to_service_stats(stats, &self.services);
    }

    async fn start_sending_stats(&self) {
        let mut interval = tokio::time::interval(self.gateway_config.gateway.latency.interval);

        loop {
            let latencies = futures_util::future::join_all(
                self.services
                    .iter()
                    .map(|service| get_service_latency(&service.1)),
            )
            .await;

            let mut stats = GatewayLatencyStats::new(self.id.clone());

            for stat in latencies {
                stats.stats.insert(stat.service_id.clone(), stat);
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
