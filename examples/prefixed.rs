use std::collections::HashMap;

use serde::Deserialize;

#[derive(Deserialize)]
struct Config {
    bar: Option<String>,
    foo: HashMap<String, String>, // Must be formatted like "{key:value,key2:value2}"
}

fn main() {
    match envy::prefixed("FOO_").from_env::<Config>() {
        Ok(config) => println!(
            "provided config.bar {:?} and config.foo {:?}",
            config.bar, config.foo
        ),
        Err(err) => println!("error parsing config from env: {}", err),
    }
}
