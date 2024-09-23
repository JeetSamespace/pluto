use crate::common::types::{ServiceStat, ServiceStatus};

use super::config::{HealthCheckType, ServiceConfig};
use reqwest::Client;
use std::time::{Duration, Instant};
use tokio::net::TcpStream;

async fn get_tcp_latency(addr: &str) -> Result<u64, std::io::Error> {
    let start = Instant::now();
    let _stream = TcpStream::connect(addr).await?;
    let latency = start.elapsed().as_millis() as u64;
    Ok(latency)
}

async fn get_http_latency(url: &str) -> Result<u64, std::io::Error> {
    let client = Client::new();
    let start = Instant::now();
    let _res = client
        .get(url)
        .send()
        .await
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    let latency = start.elapsed().as_millis() as u64;
    Ok(latency)
}

pub async fn get_service_latency(srv: &ServiceConfig) -> ServiceStat {
    let latency = match srv.health_check.r#type {
        HealthCheckType::Tcp => {
            let addr = format!("{}:{}", srv.address, srv.port);
            let res = get_tcp_latency(&addr).await;
            match res {
                Ok(latency) => latency,
                Err(e) => {
                    return ServiceStat {
                        service_id: srv.id.clone(),
                        status: ServiceStatus::Down,
                        latency: Duration::from_millis(0),
                        error: Some(e.to_string()),
                    }
                }
            }
        }
        HealthCheckType::Http => {
            let res = get_http_latency(srv.health_check.url.as_ref().unwrap()).await;

            match res {
                Ok(latency) => latency,
                Err(e) => {
                    return ServiceStat {
                        service_id: srv.id.clone(),
                        status: ServiceStatus::Down,
                        latency: Duration::from_millis(0),
                        error: Some(e.to_string()),
                    }
                }
            }
        }
    };
    ServiceStat {
        service_id: srv.id.clone(),
        status: ServiceStatus::Up,
        latency: Duration::from_millis(latency),
        error: None,
    }
}
