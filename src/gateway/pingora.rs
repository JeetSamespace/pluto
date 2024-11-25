use async_trait::async_trait;
use bytes::Bytes;
use pingora::http::RequestHeader;
use std::sync::Arc;
use tracing::info;
use lazy_static::lazy_static;
use std::collections::HashMap;
use tokio::sync::RwLock;
use pingora::proxy::{http_proxy_service, ProxyHttp, Session};
use pingora::server::configuration::Opt;
use pingora::server::Server;
use pingora::upstreams::peer::HttpPeer;
use pingora::Result;
use crate::gateway::blacklist::handle_blacklist;
use crate::gateway::api::{self as api_server};
use tokio::runtime::Builder;
use std::thread;

pub struct LB {}

pub struct MyGateway {}

lazy_static! {
    static ref ROUTES: Arc<RwLock<HashMap<String, (String, u16)>>> = Arc::new(RwLock::new({
        let mut m = HashMap::new();
        m.insert("/test".to_string(), ("127.0.0.1".to_string(), 6191));
        m.insert("/llm".to_string(), ("127.0.0.1".to_string(), 3001));
        m
    }));
}

#[async_trait]
impl ProxyHttp for LB {
    type CTX = ();
    fn new_ctx(&self) -> Self::CTX {}

    async fn request_filter(&self, session: &mut Session, _ctx: &mut Self::CTX) -> Result<bool> {
        let is_blacklisted = handle_blacklist(session.req_header()).await;
        if is_blacklisted {
            let _ = session.write_response_body(Some(Bytes::from("You are blacklisted")), false).await?;
            let _ = session.respond_error(403).await?;
            session.body_bytes_sent();
            return Ok(true);
        }
        Ok(false)   
    }

    async fn upstream_peer(
        &self,
        session: &mut Session,
        _ctx: &mut Self::CTX,
    ) -> Result<Box<HttpPeer>> {
        let mut path = session.req_header().uri.path().to_string();
        if path.ends_with('/') {
            path.pop();
        }
    
        let routes = ROUTES.read().await;
        let addr = routes
            .iter()
            .find(|(key, _)| path.starts_with(key.as_str()))
            .map(|(_, addr)| addr.clone())
            .unwrap_or(("3.110.77.152".to_string(), 443));
    
        info!("connecting to {addr:?}");
    
        let peer = Box::new(HttpPeer::new((addr.0.as_str(), addr.1), false, "google.com".to_string()));
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

pub fn run_pingora(api_ip: &String, api_port: &u16) {
    env_logger::init();

    let routes_for_api = Arc::clone(&ROUTES);

    tracing::info!("Starting Pingora server");

    let api_ip = api_ip.clone();

    let api_port = api_port.clone();

    // Spawn API server in a separate OS thread
    thread::spawn(move || {
        Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                api_server::run_api_server(routes_for_api, &api_ip, &api_port).await;
                tracing::info!("API server started")
            });
    });

    // read command line arguments
    let opt = Opt::parse_args();
    let mut my_server = Server::new(Some(opt)).unwrap();
    my_server.bootstrap();

    let mut lb = http_proxy_service(&my_server.configuration, LB {});
    lb.add_tcp("0.0.0.0:6188");
    my_server.add_service(lb);
    my_server.run_forever();
} 
