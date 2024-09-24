use std::sync::Arc;

use pluto::common::logger::init_logger;
use pluto::gateway::{config, gateway::Gateway};
use tracing::error;

#[tokio::main]
async fn main() {
    init_logger();
    let conf = config::read_gateway_config().expect("Unable to read config");
    let pluto_gateway = Gateway::new(&conf).await;
    match pluto_gateway {
        Ok(gateway) => {
            if let Err(e) = Arc::new(gateway).run().await {
                println!("Pluto Gateway exited with error: {}", e);
            }
        }
        Err(e) => error!("Failed to create Pluto Gateway: {}", e),
    }
}
