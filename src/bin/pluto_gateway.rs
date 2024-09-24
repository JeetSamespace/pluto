use std::sync::Arc;

use pluto::gateway::{config, gateway::Gateway};

#[tokio::main]
async fn main() {
    println!("Welcome to pluto-gateway");
    let conf = config::read_gateway_config().expect("Unable to read config");
    let pluto_gateway = Arc::new(Gateway::new(&conf).await);
    pluto_gateway.run().await;
}
