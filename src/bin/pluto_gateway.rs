// import the orbit module which is located in the src/orbit directory
use pluto::gateway::config;
use pluto::gateway::latency;

#[tokio::main]
async fn main() {
    let conf = config::read_gateway_config().expect("Unable to read config");
    println!("{:?}", conf);
    let rtt = latency::calculate_rtt(&conf).await;
    println!("{:?}", rtt);
    print!("Welcome to pluto-gateway")
}
