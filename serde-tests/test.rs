extern crate envy;

include!(concat!(env!("OUT_DIR"), "/test.rs"));

#[test]
fn it_works() {
    println!("{:#?}", envy::from_iter::<_, Foo>(
        vec![
            (String::from("BAR"), String::from("test")),
            (String::from("BAZ"), String::from("true"))
        ].into_iter()
    ).unwrap());
}
