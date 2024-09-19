use pluto::transport::nats::NatsPubSub;
use pluto::transport::pubsub::{Message, PubSub};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let nats = NatsPubSub::new("nats://localhost:4222").await?;

    // Example: Publish a message
    nats.publish("test_topic", Message::Data("Hello, NATS!".to_string()))
        .await?;

    // Example: Subscribe to a topic
    let mut receiver = nats.subscribe("test_topic").await?;

    // Process incoming messages
    while let Some(message) = receiver.recv().await {
        match message {
            Message::Data(data) => println!("Received: {}", data),
            Message::LatencyStats(stats) => println!("Latency stats: {:?}", stats),
        }
    }

    Ok(())
}
