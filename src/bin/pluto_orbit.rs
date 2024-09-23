use pluto::transport::nats::NatsPubSub;
use pluto::transport::pubsub::{Message, PubSubManager};
use pluto::transport::topics::PubSubTopics;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let nats = NatsPubSub::new(pluto::common::types::NatsConfig {
        url: "nats://localhost:4222".to_string(),
        cluster_id: Some("test-cluster".to_string()),
        client_id: Some("orbit-1".to_string()),
        max_reconnects: Some(5),
        reconnect_wait: Some(Duration::from_secs(5)),
    })
    .await?;

    let manager = PubSubManager::new(nats);

    // Example: Publish a message
    manager
        .broadcast(
            &[PubSubTopics::PublishGatewayLatencyStats],
            Message::Data("Hello, NATS!".to_string()),
        )
        .await?;

    // Example: Subscribe to a topic
    let mut receiver = manager
        .subscribe_to_topics(&[PubSubTopics::SubscribeGatewayLatencyStats])
        .await?;

    // Monitor latency
    let _ = manager.clone();
    // Process incoming messages
    while let Some(message) = receiver.recv().await {
        match message {
            Message::Data(data) => println!("Received: {}", data),
            Message::GatewayLatencyStats(stats) => println!("Latency stats: {:?}", stats),
            Message::Ping => println!("Received ping"),
            Message::Pong => println!("Received pong"),
        }
    }

    Ok(())
}
