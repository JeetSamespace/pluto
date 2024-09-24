use std::sync::Arc;

use anyhow::{Context, Result};
use log::debug;
use tokio::task::JoinHandle;
use tracing::{error, info, trace};

use crate::{
    common::{
        error::Error,
        types::{GatewayLatencyStats, TransportType},
    },
    transport::{
        self,
        pubsub::{Message, PubSubManager},
        topics::PubSubTopics,
    },
};

use super::config::OrbitConfig;

pub struct Orbit {
    pub config: OrbitConfig,
    transport: Arc<PubSubManager<transport::nats::NatsPubSub>>,
}

impl Orbit {
    pub async fn new(config: OrbitConfig) -> Result<Self> {
        let transport = Self::create_transport(&config).await?;
        let manager = Arc::new(PubSubManager::new(transport));

        Ok(Self {
            config,
            transport: manager,
        })
    }

    async fn create_transport(conf: &OrbitConfig) -> Result<transport::nats::NatsPubSub> {
        match conf.orbit.transport.transport_type {
            TransportType::Nats => {
                let nats_config = conf
                    .orbit
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

    pub async fn run(self: &Arc<Self>) -> Result<()> {
        let stats_receiver = self.spawn_stats_receiver();
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                error!("Received shutdown signal");
            }
            res = stats_receiver => {
                let _ = res.context("Stats receiver task failed")?;
            }
        }
        Ok(())
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
            .subscribe_to_topics(&[PubSubTopics::GatewayToOrbitStats])
            .await
            .context("failed to subscribe to topics")?;

        while let Some(msg) = rcv.recv().await {
            if let Message::GatewayLatencyStats(stats) = msg {
                trace!("received stats: {:?}", stats);
                self.broadcast_stats(stats).await?;
            }
        }
        Ok(())
    }

    async fn broadcast_stats(&self, stats: GatewayLatencyStats) -> Result<(), Error> {
        info!("broadcasting stats");
        self.transport
            .broadcast(
                &[PubSubTopics::OrbitToGatewayStats],
                Message::GatewayLatencyStats(stats),
            )
            .await
            .context("failed to broadcast stats")
            .map_err(|e| Error::PublishError(e.to_string()))?;
        Ok(())
    }
}
