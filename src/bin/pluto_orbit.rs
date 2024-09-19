use pluto::orbit::config;

fn main() {
    let conf = config::read_orbit_config().expect("Unable to read config");
    println!("{:?}", conf);
}
