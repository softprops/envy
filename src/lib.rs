//! Envy is a library for deserializing env vars into typesafe structs
//!
//! # Examples
//!
//! A typical use case for envy is deserializing configuration stored into an env into a struct
//! whose fields map to the names of env vars.
//!
//! Serde makes it easy to provide a deserializable struct with the
//! [serde_derive](https://crates.io/crates/serde_derive) crate. Simply ask for an instance of that
//! struct from envy's `from_env` function.
//!
//! ```no_run
//! #[macro_use]
//! extern crate serde_derive;
//!
//! extern crate envy;
//!
//! #[derive(Deserialize, Debug)]
//! struct Config {
//!   foo: u16,
//!   bar: bool,
//!   baz: String,
//!   boom: Option<u64>
//! }
//!
//! fn main() {
//!    match envy::from_env::<Config>() {
//!       Ok(config) => println!("{:#?}", config),
//!       Err(error) => panic!("{:#?}", error)
//!    }
//! }
//! ```
//!
//! Special treatment is given to collections. For config fields that store a `Vec` of values,
//! use and env var that uses a comma separated value
//!
//! All serde modifier should work as is
//!
//! If you wish to use enum types use the following
//!
//! ```no_run
//! #[macro_use]
//! extern crate serde_derive;
//!
//! extern crate envy;
//!
//! #[derive(Deserialize, Debug, PartialEq)]
//! #[serde(untagged)]
//! #[serde(field_identifier, rename_all = "lowercase")]
//! pub enum Size {
//!    Small,
//!    Medium,
//!    Large
//! }
//!
//! #[derive(Deserialize, Debug)]
//! struct Config {
//!  size: Size,
//! }
//!
//! fn main() {
//!    // set env var for size as `SIZE=medium`
//!    match envy::from_env::<Config>() {
//!       Ok(config) => println!("{:#?}", config),
//!       Err(error) => panic!("{:#?}", error)
//!    }
//! }
//! ```
#[macro_use]
extern crate serde;
#[cfg(test)]
#[macro_use]
extern crate serde_derive;

// Std
use std::borrow::Cow;
use std::env;
use std::iter::IntoIterator;

// Third party
use serde::de::value::{MapDeserializer, SeqDeserializer};
use serde::de::{self, IntoDeserializer};

// Ours
mod error;
pub use error::Error;

/// A type result type specific to `envy::Errors`
pub type Result<T> = std::result::Result<T, Error>;

struct Vars<Iter>(Iter)
where
    Iter: IntoIterator<Item = (String, String)>;

struct Val(String, String);

impl<'de> IntoDeserializer<'de, Error> for Val {
    type Deserializer = Self;

    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}

struct VarName(String);

impl<'de> IntoDeserializer<'de, Error> for VarName {
    type Deserializer = Self;

    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}

impl<Iter: Iterator<Item = (String, String)>> Iterator for Vars<Iter> {
    type Item = (VarName, Val);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(k, v)| (VarName(k.clone()), Val(k, v)))
    }
}

macro_rules! forward_parsed_values {
    ($($ty:ident => $method:ident,)*) => {
        $(
            fn $method<V>(self, visitor: V) -> Result<V::Value>
                where V: de::Visitor<'de>
            {
                match self.1.parse::<$ty>() {
                    Ok(val) => val.into_deserializer().$method(visitor),
                    Err(e) => Err(de::Error::custom(format_args!("{} while parsing environment variable ${}", e, self.0)))
                }
            }
        )*
    }
}

impl<'de> de::Deserializer<'de> for Val {
    type Error = Error;
    fn deserialize_any<V>(
        self,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.1.into_deserializer().deserialize_any(visitor)
    }

    fn deserialize_seq<V>(
        self,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let values = self.1.split(',').map(|v| Val(self.0.clone(), v.to_owned()));
        SeqDeserializer::new(values).deserialize_seq(visitor)
    }

    fn deserialize_option<V>(
        self,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_some(self)
    }

    forward_parsed_values! {
        bool => deserialize_bool,
        u8 => deserialize_u8,
        u16 => deserialize_u16,
        u32 => deserialize_u32,
        u64 => deserialize_u64,
        i8 => deserialize_i8,
        i16 => deserialize_i16,
        i32 => deserialize_i32,
        i64 => deserialize_i64,
        f32 => deserialize_f32,
        f64 => deserialize_f64,
    }

    forward_to_deserialize_any! {
        char str string unit
        bytes byte_buf map unit_struct tuple_struct
        identifier tuple ignored_any newtype_struct enum
        struct
    }
}

impl<'de> de::Deserializer<'de> for VarName {
    type Error = Error;
    fn deserialize_any<V>(
        self,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.0.into_deserializer().deserialize_any(visitor)
    }

    forward_to_deserialize_any! {
        char str string unit seq option
        bytes byte_buf map unit_struct tuple_struct
        identifier tuple ignored_any newtype_struct enum
        struct bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64
    }
}

/// A deserializer for env vars
struct Deserializer<'de, Iter: Iterator<Item = (String, String)>> {
    inner: MapDeserializer<'de, Vars<Iter>, Error>,
}

impl<'de, Iter: Iterator<Item = (String, String)>> Deserializer<'de, Iter> {
    fn new(vars: Iter) -> Self {
        Deserializer {
            inner: MapDeserializer::new(Vars(vars)),
        }
    }
}

impl<'de, Iter: Iterator<Item = (String, String)>> de::Deserializer<'de>
    for Deserializer<'de, Iter>
{
    type Error = Error;
    fn deserialize_any<V>(
        self,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_map<V>(
        self,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_map(self.inner)
    }

    forward_to_deserialize_any! {
        bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string unit seq
        bytes byte_buf unit_struct tuple_struct
        identifier tuple ignored_any option newtype_struct enum
        struct
    }
}

/// Deserializes a type based on information stored in env variables
pub fn from_env<T>() -> Result<T>
where
    T: de::DeserializeOwned,
{
    from_iter(env::vars())
}

/// Deserializes a type based on an iterable of `(String, String)`
/// representing keys and values
pub fn from_iter<Iter, T>(iter: Iter) -> Result<T>
where
    T: de::DeserializeOwned,
    Iter: IntoIterator<Item = (String, String)>,
{
    T::deserialize(Deserializer::new(
        iter.into_iter().map(|(k, v)| (k.to_lowercase(), v)),
    ))
}

/// A type which filters env vars with a prefix for use as serde field inputs
///
/// These types are created with with the [prefixed](fn.prefixed.html) module function
pub struct Prefixed<'a>(Cow<'a, str>);

impl<'a> Prefixed<'a> {
    /// Deserializes a type based on prefixed env varables
    pub fn from_env<T>(&self) -> Result<T>
    where
        T: de::DeserializeOwned,
    {
        self.from_iter(env::vars())
    }

    /// Deserializes a type based on prefixed (String, String) tuples
    pub fn from_iter<Iter, T>(
        &self,
        iter: Iter,
    ) -> Result<T>
    where
        T: de::DeserializeOwned,
        Iter: IntoIterator<Item = (String, String)>,
    {
        ::from_iter(iter.into_iter().filter_map(|(k, v)| {
            if k.starts_with(self.0.as_ref()) {
                Some((k.trim_left_matches(self.0.as_ref()).to_owned(), v))
            } else {
                None
            }
        }))
    }
}

/// Produces a instance of `Prefixed` for prefixing env variable names
///
/// # Example
///
/// ```no_run
/// #[macro_use]
/// extern crate serde_derive;
///
/// extern crate envy;
///
/// #[derive(Deserialize, Debug)]
/// struct Config {
///   foo: u16,
///   bar: bool,
///   baz: String,
///   boom: Option<u64>
/// }
///
/// fn main() {
///    // all env variables will be expected to be prefixed with APP_
///    // i.e. APP_FOO, APP_BAR, ect
///    match envy::prefixed("APP_").from_env::<Config>() {
///       Ok(config) => println!("{:#?}", config),
///       Err(error) => panic!("{:#?}", error)
///    }
/// }
/// ```
pub fn prefixed<'a, C>(prefix: C) -> Prefixed<'a>
where
    C: Into<Cow<'a, str>>,
{
    Prefixed(prefix.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

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
        match from_iter::<_, Foo>(data) {
            Ok(foo) => assert_eq!(
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
            ),
            Err(e) => panic!("{:#?}", e),
        }
    }

    #[test]
    fn fails_with_missing_value() {
        let data = vec![
            (String::from("BAR"), String::from("test")),
            (String::from("BAZ"), String::from("true")),
        ];
        match from_iter::<_, Foo>(data) {
            Ok(_) => panic!("expected failure"),
            Err(e) => assert_eq!(e, Error::MissingValue("doom")),
        }
    }

    #[test]
    fn fails_with_invalid_type() {
        let data = vec![
            (String::from("BAR"), String::from("test")),
            (String::from("BAZ"), String::from("notabool")),
            (String::from("DOOM"), String::from("1,2,3")),
        ];
        match from_iter::<_, Foo>(data) {
            Ok(_) => panic!("expected failure"),
            Err(e) => assert_eq!(
                e,
                Error::Custom(String::from("provided string was not `true` or `false` while parsing environment variable $baz"))
            ),
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
        match prefixed("APP_").from_iter::<_, Foo>(data) {
            Ok(foo) => assert_eq!(
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
            ),
            Err(e) => panic!("{:#?}", e),
        }
    }

    #[test]
    fn prefixed_strips_prefixes() {
        let mut expected = HashMap::new();
        expected.insert("foo".to_string(), "bar".to_string());
        assert_eq!(
            prefixed("PRE_").from_iter(vec![("PRE_FOO".to_string(), "bar".to_string())]),
            Ok(expected)
        );
    }
}
