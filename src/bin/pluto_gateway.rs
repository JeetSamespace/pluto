// import the orbit module which is located in the src/orbit directory
use pluto::gateway::config;

fn main() {
    let conf = config::read_gateway_config().expect("Unable to read config");
    println!("{:?}", conf);
    print!("Welcome to pluto-gateway")
}
