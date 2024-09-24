use std::sync::Arc;

use pluto::gateway::{config, gateway::Gateway};

#[tokio::main]
async fn main() {
    println!("Welcome to pluto-gateway");
    let conf = config::read_gateway_config().expect("Unable to read config");
    let pluto_gateway = Gateway::new(&conf).await;
    match pluto_gateway {
        Ok(gateway) => {
            if let Err(e) = Arc::new(gateway).run().await {
                println!("Pluto Gateway exited with error: {}", e);
            }
        }
        Err(e) => println!("Failed to create Pluto Gateway: {}", e),
    }
}
