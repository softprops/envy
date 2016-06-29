extern crate envy;

include!(concat!(env!("OUT_DIR"), "/test.rs"));

#[test]
fn it_works() {
    panic!("{:#?}", envy::from_iter::<_, Foo>(
        vec![
            (String::from("BAR"), String::from("test")),
            (String::from("BAZ"), String::from("true")),
            (String::from("DOOM"), String::from("1,2,3"))
        ].into_iter()
    ).unwrap());
}
