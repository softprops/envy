#[macro_use]
extern crate serde_derive;

extern crate envy;

#[derive(Deserialize)]
struct Config {
    size: Option<u32>,
}

fn main() {
    match envy::from_env::<Config>() {
        Ok(config) => println!("provided config.example {:?}", config.size),
        Err(err) => println!("error parsing config from env: {}", err),
    }
}
