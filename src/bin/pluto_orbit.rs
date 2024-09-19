use pluto::transport::nats::NatsPubSub;
use pluto::transport::pubsub::{Message, PubSubManager};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let nats = NatsPubSub::new("nats://localhost:4222").await?;
    let manager = PubSubManager::new(nats);

    // Example: Publish a message
    manager
        .broadcast(&["test_topic"], Message::Data("Hello, NATS!".to_string()))
        .await?;

    // Example: Subscribe to a topic
    let mut receiver = manager.subscribe_to_topics(&vec!["test_topic"]).await?;

    // Monitor latency
    let manager_clone = manager.clone();

    tokio::spawn(async move {
        manager_clone.monitor_latency(Duration::from_secs(5)).await;
    });

    // Process incoming messages
    while let Some(message) = receiver.recv().await {
        match message {
            Message::Data(data) => println!("Received: {}", data),
            Message::LatencyStats(stats) => println!("Latency stats: {:?}", stats),
        }
    }

    Ok(())
}
