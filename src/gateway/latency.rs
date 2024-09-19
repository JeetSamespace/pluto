use crate::gateway::config::GatewayConfig;
use std::collections::HashMap;
use std::error::Error;
use std::net::ToSocketAddrs;
use tokio::net::TcpStream;
use tokio::time::{Duration, Instant, interval};

pub async fn calculate_rtt(config: &GatewayConfig) -> Result<(), Box<dyn Error>> {
    let mut rtt_map = HashMap::new();
    let interval_duration = config.gateway.latency.interval;

    let mut interval = interval(interval_duration);

    loop {
        interval.tick().await; // Wait for the next interval

        for service in &config.gateway.services {
            match ping(&service.address, service.port, config.gateway.latency.timeout).await {
                Ok(duration) => {
                    println!("Ping to {} ({}:{}) successful: {:?}", service.name, service.address, service.port, duration);
                    rtt_map.insert(service.name.clone(), duration);
                }
                Err(e) => {
                    println!("Failed to ping {} ({}:{}): {}", service.name, service.address, service.port, e);
                }
            }
        }

        // Here you can add logic to use or store the rtt_map as needed
        // For example, you might want to update a shared state or send it to another part of your application
        println!("RTT map: {:?}", rtt_map);
    }
}

async fn ping(address: &str, port: u16, timeout: Duration) -> Result<Duration, Box<dyn Error>> {
    println!("Pinging {} on port {}", address, port);
    // Resolve the address with port to a socket address
    let addr = format!("{}:{}", address, port).to_socket_addrs()?.next().ok_or("Invalid address")?;

    // Record the start time
    let start = Instant::now();

    // Attempt to establish a TCP connection with timeout
    match tokio::time::timeout(timeout, TcpStream::connect(addr)).await {
        Ok(Ok(_)) => {
            // Calculate the round-trip time
            let rtt = start.elapsed();
            Ok(rtt)
        }
        Ok(Err(e)) => Err(Box::new(e)),
        Err(_) => Err("Connection timed out".into()),
    }
}
