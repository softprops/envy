extern crate envy;

include!(concat!(env!("OUT_DIR"), "/main.rs"));

fn main() {
    //println!("{:#?}", envy::from_env::<Foo>())
}
