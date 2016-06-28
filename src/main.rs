extern crate envy;
extern crate serde;

use std::env;

include!(concat!(env!("OUT_DIR"), "/main.rs"));

fn main() {
    println!("{:#?}", envy::from_env::<Foo>(env::vars()))
}
