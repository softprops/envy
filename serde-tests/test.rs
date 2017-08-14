#[macro_use]
extern crate serde_derive;

extern crate envy;

#[derive(Deserialize, Debug, PartialEq)]
#[serde(untagged)]
#[serde(field_identifier, rename_all = "lowercase")]
pub enum Size {
    Small,
    Medium,
    Large,
}

impl Default for Size {
    fn default() -> Size {
        Size::Medium
    }
}

pub fn default_kaboom() -> u16 {
    8080
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct Foo {
    bar: String,
    baz: bool,
    zoom: Option<u16>,
    doom: Vec<u64>,
    #[serde(default = "default_kaboom")]
    kaboom: u16,
    #[serde(default)]
    debug_mode: bool,
    #[serde(default)]
    size: Size,
    provided: Option<String>,
}

#[test]
fn deserialize_from_iter() {
    let data = vec![
        (String::from("BAR"), String::from("test")),
        (String::from("BAZ"), String::from("true")),
        (String::from("DOOM"), String::from("1,2,3")),
        (String::from("SIZE"), String::from("small")),
        (String::from("PROVIDED"), String::from("test")),
    ];
    match envy::from_iter::<_, Foo>(data.into_iter()) {
        Ok(foo) => {
            assert_eq!(
                foo,
                Foo {
                    bar: String::from("test"),
                    baz: true,
                    zoom: None,
                    doom: vec![1, 2, 3],
                    kaboom: 8080,
                    debug_mode: false,
                    size: Size::Small,
                    provided: Some(String::from("test")),
                }
            )
        }
        Err(e) => panic!("{:#?}", e),
    }
}

#[test]
fn fails_with_missing_value() {
    let data = vec![
        (String::from("BAR"), String::from("test")),
        (String::from("BAZ"), String::from("true")),
    ];
    match envy::from_iter::<_, Foo>(data.into_iter()) {
        Ok(_) => panic!("expected failure"),
        Err(e) => assert_eq!(e, envy::Error::MissingValue("doom")),
    }
}

#[test]
fn fails_with_invalid_type() {
    let data = vec![
        (String::from("BAR"), String::from("test")),
        (String::from("BAZ"), String::from("notabool")),
        (String::from("DOOM"), String::from("1,2,3")),
    ];
    match envy::from_iter::<_, Foo>(data.into_iter()) {
        Ok(_) => panic!("expected failure"),
        Err(e) => {
            assert_eq!(
                e,
                envy::Error::Custom(String::from("provided string was not `true` or `false`"))
            )
        }
    }
}

#[test]
fn deserializes_from_prefixed_fieldnames() {
    let data = vec![
        (String::from("APP_BAR"), String::from("test")),
        (String::from("APP_BAZ"), String::from("true")),
        (String::from("APP_DOOM"), String::from("1,2,3")),
        (String::from("APP_SIZE"), String::from("small")),
        (String::from("APP_PROVIDED"), String::from("test")),
    ];
    match envy::prefixed("APP_").from_iter::<_, Foo>(data.into_iter()) {
        Ok(foo) => {
            assert_eq!(
                foo,
                Foo {
                    bar: String::from("test"),
                    baz: true,
                    zoom: None,
                    doom: vec![1, 2, 3],
                    kaboom: 8080,
                    debug_mode: false,
                    size: Size::Small,
                    provided: Some(String::from("test")),
                }
            )
        }
        Err(e) => panic!("{:#?}", e),
    }
}
