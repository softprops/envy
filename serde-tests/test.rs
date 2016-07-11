extern crate envy;

pub fn default_kaboom() -> u16 {
    8080
}

include!(concat!(env!("OUT_DIR"), "/test.rs"));

#[test]
fn deserialize_from_iter() {
    let data = vec![
        (String::from("BAR"), String::from("test")),
        (String::from("BAZ"), String::from("true")),
        (String::from("DOOM"), String::from("1,2,3"))
    ];
    match envy::from_iter::<_, Foo>(data.into_iter()) {
        Ok(foo) => {
            assert_eq!(
                    foo, Foo {
                        bar: String::from("test"),
                        baz: true,
                        zoom: None,
                        doom: vec![1,2,3],
                        kaboom: 8080
                    }
            )
        },
        Err(e) => panic!("{:#?}", e)
    }
}

#[test]
fn fails_with_missing_value() {
    let data = vec![
        (String::from("BAR"), String::from("test")),
        (String::from("BAZ"), String::from("true"))
    ];
    match envy::from_iter::<_, Foo>(data.into_iter()) {
        Ok(_) => panic!("expected failure"),
        Err(e) => assert_eq!(e, envy::Error::MissingValue("doom"))
    }
}

#[test]
fn fails_with_invalid_type() {
    let data = vec![
        (String::from("BAR"), String::from("test")),
        (String::from("BAZ"), String::from("notabool")),
        (String::from("DOOM"), String::from("1,2,3"))
    ];
    match envy::from_iter::<_, Foo>(data.into_iter()) {
        Ok(_) => panic!("expected failure"),
        Err(e) => assert_eq!(e, envy::Error::Custom(String::from("Invalid type. Expected `Bool`")))
    }
}
