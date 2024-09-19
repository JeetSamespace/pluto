use crate::common::error::Error;
use crate::transport::pubsub::{LatencyStats, Message, PubSub};
use async_nats::Client;
use async_trait::async_trait;
use futures_util::stream::StreamExt;
use std::sync::Arc;
use tokio::sync::mpsc;

#[derive(Clone)]
pub struct NatsPubSub {
    client: Arc<Client>,
}

impl NatsPubSub {
    pub async fn new(url: &str) -> Result<Self, Error> {
        let client = async_nats::connect(url)
            .await
            .map_err(|e| Error::ConnectionError(e.to_string()))?;
        Ok(Self {
            client: Arc::new(client),
        })
    }
}

#[async_trait]
impl PubSub for NatsPubSub {
    async fn publish(&self, topic: &str, message: Message) -> Result<(), Error> {
        let payload =
            serde_json::to_vec(&message).map_err(|e| Error::SerializationError(e.to_string()))?;
        self.client
            .publish(topic.to_string(), payload.into())
            .await
            .map_err(|e| Error::PublishError(e.to_string()))?;
        Ok(())
    }

    async fn subscribe(&self, topic: &str) -> Result<mpsc::Receiver<Message>, Error> {
        let mut subscription = self
            .client
            .subscribe(topic.to_string())
            .await
            .map_err(|e| Error::SubscriptionError(e.to_string()))?;
        let (tx, rx) = mpsc::channel(100); // 100 is the buffer size
        tokio::spawn(async move {
            while let Some(msg) = subscription.next().await {
                if let Ok(message) = serde_json::from_slice::<Message>(&msg.payload) {
                    if tx.send(message).await.is_err() {
                        break;
                    }
                }
            }
        });

        Ok(rx)
    }

    async fn publish_latency_stats(&self, stats: LatencyStats) -> Result<(), Error> {
        self.publish("latency_stats", Message::LatencyStats(stats))
            .await
    }

    async fn on_latency_stats_received(&self, callback: Box<dyn Fn(LatencyStats) + Send + Sync>) {
        let mut subscription = self
            .client
            .subscribe("latency_stats".to_string())
            .await
            .expect("Failed to subscribe to latency_stats");

        tokio::spawn(async move {
            while let Some(msg) = subscription.next().await {
                if let Ok(Message::LatencyStats(stats)) = serde_json::from_slice(&msg.payload) {
                    callback(stats);
                }
            }
        });
    }
}
