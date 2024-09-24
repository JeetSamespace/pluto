use tracing::{subscriber::set_global_default, Level};
use tracing_subscriber::FmtSubscriber;

pub fn init_logger() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();

    set_global_default(subscriber).expect("failed to set global default subscriber");
    tracing::info!("logger initialized");
}
