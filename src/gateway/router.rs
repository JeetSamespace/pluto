use async_trait::async_trait;
use pingora::{lb::Backend, prelude::*};
use std::sync::Arc;
use pingora::proxy::HttpProxy;

// Re-export necessary types and functions
pub use pingora::server::Server;
pub use pingora::lb::LoadBalancer;

pub struct LB {
    pub balancer: Arc<LoadBalancer<RoundRobin>>
}

#[async_trait]
impl ProxyHttp for LB {

    /// For this small example, we don't need context storage
    type CTX = ();
    fn new_ctx(&self) -> () {
        ()
    }

    async fn upstream_peer(&self, _session: &mut Session, _ctx: &mut ()) -> Result<Box<HttpPeer>> {
        let upstream = self.balancer
            .select(b"", 256) // hash doesn't matter for round robin
            .unwrap();

        println!("upstream peer is: {upstream:?}");


        // Set SNI to one.one.one.one
        let peer = Box::new(HttpPeer::new(upstream, false, "one.one.one.one".to_string()));
        Ok(peer)
    }


    async fn upstream_request_filter(
        &self,
        _session: &mut Session,
        _upstream_request: &mut RequestHeader,
        _ctx: &mut Self::CTX,
    ) -> Result<()> {
        Ok(())
    }
}

pub struct BackgroundService {
    name: String,
    upstreams: LoadBalancer<RoundRobin>,
}

impl BackgroundService {
    fn new(name: String, upstreams: LoadBalancer<RoundRobin>) -> Self {
        Self { name, upstreams }
    }

    pub fn task(self) -> Arc<LoadBalancer<RoundRobin>> {
        Arc::new(self.upstreams)
    }
}

pub fn background_service(name: &str, upstreams: LoadBalancer<RoundRobin>) -> BackgroundService {
    BackgroundService::new(name.to_string(), upstreams)
}