use std::sync::Arc;

use pluto::common::logger::init_logger;
use pluto::gateway::{config, gateway::Gateway};
use tracing::error;

fn main() {
    init_logger();
    let conf = config::read_gateway_config().expect("Unable to read config");

    let runtime = tokio::runtime::Runtime::new().expect("Failed to create runtime");
    let pluto_gateway = runtime.block_on(async { Gateway::new(&conf).await });

    match pluto_gateway {
        Ok(gateway) => {
            let gateway = Arc::new(gateway);
            if let Err(e) = gateway.start_pingora_server() {
                println!("Pluto Gateway exited with error: {}", e);
            }

            // run the run() method in blocking way
            if let Err(e) = runtime.block_on(gateway.run()) {
                println!("Pluto Gateway exited with error: {}", e);
            }
        }
        Err(e) => error!("Failed to create Pluto Gateway: {}", e),
    }
}
