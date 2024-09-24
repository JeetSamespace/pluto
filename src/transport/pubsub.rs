use crate::common::error::Error;
use crate::common::types::GatewayLatencyStats;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::debug;

use super::topics::PubSubTopics;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyStats {
    pub min: Duration,
    pub max: Duration,
    pub avg: Duration,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Message {
    Data(String),
    GatewayLatencyStats(GatewayLatencyStats),
    Ping,
    Pong,
}

#[async_trait]
pub trait PubSub: Clone + Send + Sync + 'static {
    async fn publish(&self, topic: PubSubTopics, message: Message) -> Result<(), Error>;
    async fn subscribe(&self, topic: PubSubTopics) -> Result<mpsc::Receiver<Message>, Error>;
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

    pub async fn broadcast(&self, topics: &[PubSubTopics], message: Message) -> Result<(), Error> {
        debug!("broadcasting message to topics: {:?}", topics);
        for topic in topics {
            self.inner.publish(topic.clone(), message.clone()).await?;
        }
        Ok(())
    }

    pub async fn subscribe_to_topics(
        &self,
        topics: &[PubSubTopics],
    ) -> Result<mpsc::Receiver<Message>, Error> {
        let (tx, rx) = mpsc::channel(100);

        for topic in topics {
            let mut receiver = self.inner.subscribe(topic.clone()).await?;
            let tx = tx.clone();

            tokio::spawn(async move {
                while let Some(message) = receiver.recv().await {
                    if tx.send(message).await.is_err() {
                        break;
                    }
                }
            });
        }

        Ok(rx)
    }
}
