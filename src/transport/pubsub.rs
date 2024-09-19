use crate::common::error::Error;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyStats {
    pub min: Duration,
    pub max: Duration,
    pub avg: Duration,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum Message {
    Data(String),
    LatencyStats(LatencyStats),
}

#[async_trait]
pub trait PubSub: Clone + Send + Sync + 'static {
    async fn publish(&self, topic: &str, message: Message) -> Result<(), Error>;
    async fn subscribe(&self, topic: &str) -> Result<mpsc::Receiver<Message>, Error>;
    async fn publish_latency_stats(&self, stats: LatencyStats) -> Result<(), Error>;
    async fn on_latency_stats_received(&self, callback: Box<dyn Fn(LatencyStats) + Send + Sync>);
}

#[derive(Debug, Clone)]
pub struct PubSubManager<T: PubSub> {
    inner: Arc<T>,
}

impl<T: PubSub> PubSubManager<T> {
    pub fn new(inner: T) -> Self {
        Self {
            inner: Arc::new(inner),
        }
    }

    pub async fn broadcast(&self, topics: &[&str], message: Message) -> Result<(), Error> {
        for topic in topics {
            self.inner.publish(topic, message.clone()).await?;
        }
        Ok(())
    }
    pub async fn monitor_latency(&self, interval: Duration) {
        let inner = self.inner.clone();
        self.inner
            .on_latency_stats_received(Box::new(move |stats| {
                let inner = inner.clone();
                tokio::spawn(async move {
                    let _ = inner.publish_latency_stats(stats).await;
                });
            }))
            .await;

        loop {
            tokio::time::sleep(interval).await;
        }
    }
}
