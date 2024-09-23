use serde::{Deserialize, Serialize};
use std::{collections::HashMap, time::Duration};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ServiceStatus {
    Up,
    Down,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServiceStat {
    pub service_id: String,
    pub status: ServiceStatus,
    pub latency: Duration,
    pub error: Option<String>,
}

pub type GatewayLatencyStats = HashMap<String, ServiceStat>;

#[derive(Debug, Clone, Deserialize)]
pub struct TransportConfig {
    #[serde(rename = "type")]
    pub transport_type: TransportType,
    pub nats: Option<NatsConfig>,
    pub kafka: Option<KafkaConfig>,
    pub rabbitmq: Option<RabbitMQConfig>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransportType {
    Nats,
    Redis,
    Kafka,
    RabbitMQ,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NatsConfig {
    pub url: String,
    pub cluster_id: Option<String>,
    pub client_id: Option<String>,
    pub max_reconnects: Option<i32>,
    pub reconnect_wait: Option<Duration>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct KafkaConfig {
    pub brokers: Vec<String>,
    pub client_id: String,
    pub group_id: String,
    pub max_message_bytes: Option<usize>,
    pub session_timeout_ms: Option<i32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RabbitMQConfig {
    pub url: String,
    pub exchange: String,
    pub queue: String,
    pub routing_key: String,
    pub prefetch_count: Option<u16>,
    pub connection_timeout: Option<Duration>,
}
