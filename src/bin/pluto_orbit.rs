use pluto::common::logger::init_logger;
use pluto::orbit::config::read_orbit_config;
use pluto::orbit::orbit::Orbit;
use std::sync::Arc;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_logger();

    let config = read_orbit_config()?;
    let orbit = Arc::new(Orbit::new(config).await?);

    info!("starting orbit");

    if let Err(e) = Arc::new(orbit).run().await {
        println!("Pluto Orbit exited with error: {}", e);
    }

    Ok(())
}
