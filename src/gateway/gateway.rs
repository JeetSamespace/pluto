use anyhow::{Context, Result};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::task::JoinHandle;
use tracing::{debug, error, info};

use super::config::{GatewayConfig, ServiceConfig};
use super::latency::get_service_latency;
use super::store::Store;
use crate::common::types::{GatewayLatencyStats, TransportType};
use crate::transport;
use crate::transport::pubsub::{Message, PubSubManager};
use crate::transport::topics::PubSubTopics;

pub struct Gateway {
    id: String,
    gateway_config: GatewayConfig,
    services: HashMap<String, ServiceConfig>,
    transport: Arc<PubSubManager<transport::nats::NatsPubSub>>,
    store: Arc<Store>,
}

impl Gateway {
    pub async fn new(conf: &GatewayConfig) -> Result<Self> {
        let transport = Self::create_transport(conf).await?;
        let manager = Arc::new(PubSubManager::new(transport));

        let services = conf
            .gateway
            .services
            .iter()
            .map(|service| (service.id.clone(), service.clone()))
            .collect();

        Ok(Gateway {
            id: conf.gateway.name.clone(),
            gateway_config: conf.clone(),
            transport: manager,
            store: Arc::new(Store::new()),
            services,
        })
    }

    async fn create_transport(conf: &GatewayConfig) -> Result<transport::nats::NatsPubSub> {
        match conf.gateway.transport.transport_type {
            TransportType::Nats => {
                let nats_config = conf
                    .gateway
                    .transport
                    .nats
                    .clone()
                    .context("NATS configuration missing")?;
                transport::nats::NatsPubSub::new(nats_config)
                    .await
                    .context("Failed to create NATS PubSub")
            }
            _ => anyhow::bail!("Invalid transport type"),
        }
    }

    pub async fn run(self: Arc<Self>) -> Result<()> {
        info!("starting gateway");
        let stats_sender = self.spawn_stats_sender();
        let stats_receiver = self.spawn_stats_receiver();

        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                error!("Received shutdown signal");
            }
            res = stats_sender => {
                let _ = res.context("Stats sender task failed")?;
            }
            res = stats_receiver => {
                let _ = res.context("Stats receiver task failed")?;
            }
        }

        println!("Shutting down gateway");
        Ok(())
    }

    fn spawn_stats_sender(self: &Arc<Self>) -> JoinHandle<Result<()>> {
        debug!("spawning stats sender");
        let self_clone = Arc::clone(self);
        tokio::spawn(async move { self_clone.start_sending_stats().await })
    }

    fn spawn_stats_receiver(self: &Arc<Self>) -> JoinHandle<Result<()>> {
        debug!("spawning stats receiver");
        let self_clone = Arc::clone(self);
        tokio::spawn(async move { self_clone.start_receiving_stats().await })
    }

    async fn start_receiving_stats(&self) -> Result<()> {
        info!("starting receiving stats");
        let mut rcv = self
            .transport
            .subscribe_to_topics(&[PubSubTopics::SubscribeGatewayLatencyStats])
            .await
            .context("Failed to subscribe to topics")?;

        while let Some(msg) = rcv.recv().await {
            if let Message::GatewayLatencyStats(stats) = msg {
                self.handle_latency_stats(stats).await?;
            }
        }
        Ok(())
    }

    async fn handle_latency_stats(&self, stats: GatewayLatencyStats) -> Result<()> {
        self.store
            .update_gateway_to_service_stats(stats, &self.services);
        Ok(())
    }

    async fn start_sending_stats(&self) -> Result<()> {
        info!("starting sending stats");
        let mut interval = tokio::time::interval(self.gateway_config.gateway.latency.interval);

        loop {
            interval.tick().await;

            let latencies =
                futures_util::future::join_all(self.services.values().map(get_service_latency))
                    .await;

            let mut stats = GatewayLatencyStats::new(self.id.clone());

            for stat in latencies {
                stats.stats.insert(stat.service_id.clone(), stat);
            }

            self.transport
                .broadcast(
                    &[PubSubTopics::PublishGatewayLatencyStats],
                    Message::GatewayLatencyStats(stats),
                )
                .await
                .context("Failed to broadcast gateway latency stats")?;
        }
    }
}
