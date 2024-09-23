use crate::gateway::config::GatewayConfig;
use std::collections::HashMap;
use std::error::Error;
use tokio::time::{Duration, interval};
use tokio::sync::mpsc;

pub async fn run_heartbeat(config: &GatewayConfig, mut rtt_receiver: mpsc::Receiver<HashMap<String, Duration>>) -> Result<(), Box<dyn Error>> {
    let mut interval = interval(config.gateway.heartbeat.interval);
    let mut latest_rtt_map: HashMap<String, Duration> = HashMap::new();

    loop {
        interval.tick().await;

        // Check for new RTT data
        while let Ok(new_rtt_map) = rtt_receiver.try_recv() {
            latest_rtt_map = new_rtt_map;
        }

        // Use the latest RTT data in the heartbeat logic
        for service in &config.gateway.services {
            let rtt = latest_rtt_map.get(&service.name).cloned().unwrap_or_default();
            //send_heartbeat(service, rtt).await?;
        }
    }
}

// async fn send_heartbeat(service: &GatewayConfig, rtt: Duration) -> Result<(), Box<dyn Error>> {
//     // Implement the logic to send a heartbeat to the service
//     println!("Sending heartbeat to {} with RTT: {:?}", service.name, rtt);
//     // Add your heartbeat sending logic here
//     Ok(())
// }
