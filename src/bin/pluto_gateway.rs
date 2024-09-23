use pluto::gateway::{config, gateway::Gateway};

#[tokio::main]
async fn main() {
    let conf = config::read_gateway_config().expect("Unable to read config");

    let pluto_gateway = Gateway::new(&conf).await;

    pluto_gateway.run().await;

    print!("Welcome to pluto-gateway")
}
