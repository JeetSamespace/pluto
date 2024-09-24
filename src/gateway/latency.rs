use crate::common::types::{ServiceStat, ServiceStatus};
use tracing::{instrument, trace};

use super::config::{HealthCheckType, ServiceConfig};
use reqwest::Client;
use std::time::{Duration, Instant};
use tokio::net::TcpStream;

#[instrument(level = "trace")]
async fn get_tcp_latency(addr: &str) -> Result<u64, std::io::Error> {
    trace!("measuring tcp latency for address: {}", addr);
    let start = Instant::now();
    let _stream = TcpStream::connect(addr).await?;
    let latency = start.elapsed().as_millis() as u64;
    trace!("tcp latency measurement complete: {} ms", latency);
    Ok(latency)
}

#[instrument(level = "trace")]
async fn get_http_latency(url: &str) -> Result<u64, std::io::Error> {
    trace!("measuring http latency for url: {}", url);
    let client = Client::new();
    let start = Instant::now();
    let _res = client
        .get(url)
        .send()
        .await
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    let latency = start.elapsed().as_millis() as u64;
    trace!("http latency measurement complete: {} ms", latency);
    Ok(latency)
}

#[instrument(level = "trace", skip(srv))]
pub async fn get_service_latency(srv: &ServiceConfig) -> ServiceStat {
    trace!("getting service latency for service id: {}", srv.id);
    let latency = match srv.health_check.r#type {
        HealthCheckType::Tcp => {
            let addr = format!("{}:{}", srv.address, srv.port);
            let res = get_tcp_latency(&addr).await;
            match res {
                Ok(latency) => latency,
                Err(e) => {
                    trace!("tcp latency measurement failed: {}", e);
                    return ServiceStat {
                        service_id: srv.id.clone(),
                        status: ServiceStatus::Down,
                        latency: Duration::from_millis(0),
                        error: Some(e.to_string()),
                    };
                }
            }
        }
        HealthCheckType::Http => {
            let res = get_http_latency(srv.health_check.url.as_ref().unwrap()).await;

            match res {
                Ok(latency) => latency,
                Err(e) => {
                    trace!("http latency measurement failed: {}", e);
                    return ServiceStat {
                        service_id: srv.id.clone(),
                        status: ServiceStatus::Down,
                        latency: Duration::from_millis(0),
                        error: Some(e.to_string()),
                    };
                }
            }
        }
    };
    trace!("service latency measurement complete: {} ms", latency);
    ServiceStat {
        service_id: srv.id.clone(),
        status: ServiceStatus::Up,
        latency: Duration::from_millis(latency),
        error: None,
    }
}
