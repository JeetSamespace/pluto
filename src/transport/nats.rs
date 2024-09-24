use crate::common::error::Error;
use crate::common::types::NatsConfig;
use crate::transport::pubsub::{Message, PubSub};
use anyhow::{Context, Error as AnyhowError};
use async_nats::Client;
use async_nats::ConnectOptions;
use async_trait::async_trait;
use futures_util::StreamExt;
use std::sync::Arc;
use tokio::sync::mpsc;

use super::topics::PubSubTopics;

#[derive(Clone, Debug)]
pub struct NatsPubSub {
    client: Arc<Client>,
}

impl NatsPubSub {
    pub async fn new(conf: NatsConfig) -> Result<Self, AnyhowError> {
        let mut options = ConnectOptions::new();
        if let Some(max_reconnects) = conf.max_reconnects {
            options = options.max_reconnects(max_reconnects as usize);
        }

        let client = options
            .connect(&conf.url)
            .await
            .context("Failed to connect to NATS server")?;
        Ok(Self {
            client: Arc::new(client),
        })
    }
}

#[async_trait]
impl PubSub for NatsPubSub {
    async fn publish(&self, topic: PubSubTopics, message: Message) -> Result<(), AnyhowError> {
        let payload =
            serde_json::to_vec(&message).map_err(|e| Error::SerializationError(e.to_string()))?;
        self.client
            .publish(topic.as_str(), payload.into())
            .await
            .map_err(|e| Error::PublishError(e.to_string()))?;
        Ok(())
    }

    async fn subscribe(&self, topic: PubSubTopics) -> Result<mpsc::Receiver<Message>, AnyhowError> {
        let mut subscription = self
            .client
            .subscribe(topic.as_str())
            .await
            .map_err(|e| Error::SubscriptionError(e.to_string()))?;
        let (tx, rx) = mpsc::channel(100);
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
}
