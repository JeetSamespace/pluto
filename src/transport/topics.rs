#[derive(Debug, Clone)]
pub enum PubSubTopics {
    PublishGatewayLatencyStats,   // from gateway to orbit
    SubscribeGatewayLatencyStats, // from orbit to gateway
    PublishGatewayHeartbeat,      // from gateway to orbit
    SubscribeGatewayHeartbeat,    // from orbit to gateway
    PublishGatewayFailover,       // from gateway to orbit
    SubscribeGatewayFailover,     // from orbit to gateway
    PublishConfigUpdate,          // from orbit to gateways
    SubscribeConfigUpdate,        // from gateways to orbit
    PublishGatewayMetrics,        // from gateway to orbit
    SubscribeGatewayMetrics,      // from orbit to gateway
}

impl PubSubTopics {
    pub fn as_str(&self) -> &'static str {
        match self {
            PubSubTopics::PublishGatewayLatencyStats => "orbit.gateway.latency.stats", // from gateway to orbit
            PubSubTopics::SubscribeGatewayLatencyStats => "orbit.*.latency.stats", // from orbit to gateway
            PubSubTopics::PublishGatewayHeartbeat => "orbit.gateway.heartbeat", // from gateway to orbit
            PubSubTopics::SubscribeGatewayHeartbeat => "orbit.*.heartbeat", // from orbit to gateway
            PubSubTopics::PublishGatewayFailover => "orbit.gateway.failover", // from gateway to orbit
            PubSubTopics::SubscribeGatewayFailover => "orbit.*.failover", // from orbit to gateway
            PubSubTopics::PublishConfigUpdate => "orbit.config.update",   // from orbit to gateways
            PubSubTopics::SubscribeConfigUpdate => "orbit.*.config.update", // from gateways to orbit
            PubSubTopics::PublishGatewayMetrics => "orbit.gateway.metrics", // from gateway to orbit
            PubSubTopics::SubscribeGatewayMetrics => "orbit.*.metrics",     // from orbit to gateway
        }
    }
}
