use std::sync::Arc;
use anyhow::Result;
use crate::gateway::pingora::run_pingora;
use super::gateway::Gateway;

impl Gateway {
    pub fn start_pingora_server(self: Arc<Self>) -> Result<()> {
        tracing::info!("Starting Pingora server");
        let api_ip = &self.api_ip;
        let api_port = &self.api_port;

        run_pingora(api_ip, api_port);

        Ok(())
    }
}
