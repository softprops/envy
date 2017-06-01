#[macro_use]
extern crate serde_derive;

extern crate envy;

#[derive(Deserialize)]
struct Config {
    #[serde(rename="foo_bar")]
    bar: Option<String>,
}

fn main() {
    match envy::from_env::<Config>() {
        Ok(config) => println!("provided config.bar {:?}", config.bar),
        Err(err) => println!("error parsing config from env: {}", err),
    }
}
