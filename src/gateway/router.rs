use std::sync::Arc;

use anyhow::Result;
use tracing::info;

use crate::gateway::pingora::run_pingora;

use super::gateway::Gateway;

impl Gateway {
    pub fn start_pingora_server(self: &Arc<Self>) -> Result<()> {
        std::thread::spawn(|| {
            info!("starting pingora server");
            run_pingora();
        });

        Ok(())
    }
}
