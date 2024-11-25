use pluto::common::logger::init_logger;
use pluto::orbit::config::read_orbit_config;
use pluto::orbit::orbit::Orbit;
use std::sync::Arc;
use tracing::info;
use pluto::orbit::api;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_logger();

     let config = read_orbit_config()?;
     let api_config = config.orbit.api.clone();
     let orbit = Arc::new(Orbit::new(config).await?);

    info!("starting orbit");

    // Start the main Orbit run task.
    let orbit_handle = Arc::clone(&orbit);
    tokio::spawn(async move {
        if let Err(e) = orbit_handle.run().await {
            println!("Pluto Orbit exited with error: {}", e);
        }
    });
    // Start the API server on a separate Tokio thread.
    let api_handle = tokio::spawn(async move {
        api::start_server(&api_config).await
    });

    // Wait for the API server and the main Orbit task.
    let _ = api_handle.await?;

    Ok(())
}