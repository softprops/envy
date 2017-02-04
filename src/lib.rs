//! envy is a library for deserializing env vars into typesafe structs
#[macro_use]
extern crate serde;

use serde::de;
use std::collections::HashMap;
use serde::de::value::ValueDeserializer;

mod errors;
pub use errors::Error;

/// A type result type specific to `envy::Errors`
pub type Result<T> = std::result::Result<T, Error>;

/// an env var key, struct_field name, and value if one exists
#[derive(Clone, Debug)]
struct Var {
    key: String,
    struct_field: &'static str,
    value: Option<String>,
}

impl Var {
    fn new(struct_field: &'static str, vars: &HashMap<String, String>) -> Var {
        let key = struct_field.to_string().to_uppercase();
        let value = vars.get(&key).map(|v| v.clone());
        Var {
            key: key,
            struct_field: struct_field,
            value: value,
        }
    }
}

/// visits literal vec of strings
struct SeqVisitor<'a> {
    de: &'a mut Deserializer,
    values: Vec<String>,
}

impl<'a> SeqVisitor<'a> {
    fn new(de: &'a mut Deserializer, values: Vec<String>) -> SeqVisitor<'a> {
        SeqVisitor {
            de: de,
            values: values,
        }
    }
}

impl<'a> de::SeqVisitor for SeqVisitor<'a> {
    type Error = Error;

    fn visit_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
        where T: de::DeserializeSeed
    {
        match self.values.pop() {
            Some(val) => {
                let mut de = ValDeserializer {
                    de: self.de,
                    value: val.clone(),
                };
                seed.deserialize(&mut de).map(Some)
            }
            _ => Ok(None),
        }
    }
}

/// deserializes a single env string value
/// if the intent to deserialize a seq, we split the string by
// comma an deserialize the resolving values separately
struct ValDeserializer<'a> {
    de: &'a mut Deserializer,
    value: String,
}

impl<'a, 'r> de::Deserializer for &'r mut ValDeserializer<'a> {
    type Error = Error;
    fn deserialize<V>(self, visitor: V) -> Result<V::Value>
        where V: de::Visitor
    {
        visitor.visit_string(self.value.clone())
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
        where V: de::Visitor
    {
        let mut values = self.value.split(",").map(|v| v.to_string()).collect::<Vec<String>>();
        values.reverse();
        visitor.visit_seq(SeqVisitor::new(self.de, values))
    }

    fn deserialize_struct<V>(self,
                             _name: &'static str,
                             _fields: &'static [&'static str],
                             visitor: V)
                             -> Result<V::Value>
        where V: de::Visitor
    {
        self.de.deserialize_struct(_name, _fields, visitor)
    }

    forward_to_deserialize! {
        bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string unit
        seq_fixed_size bytes byte_buf map unit_struct tuple_struct
        struct_field tuple ignored_any option newtype_struct enum
    }
}

/// A deserializer for env vars
struct Deserializer {
    vars: HashMap<String, String>,
    stack: Vec<Var>,
}

impl Deserializer {
    fn new(vars: HashMap<String, String>) -> Deserializer {
        Deserializer {
            vars: vars,
            stack: vec![],
        }
    }
}

impl <'r> de::Deserializer for  &'r mut Deserializer {
    type Error = Error;
    fn deserialize<V>(self, visitor: V) -> Result<V::Value>
        where V: de::Visitor
    {
        visitor.visit_map(MapVisitor::new(self))
    }

    fn deserialize_struct<V>(self,
                             _name: &'static str,
                             _fields: &'static [&'static str],
                             visitor: V)
                             -> Result<V::Value>
        where V: de::Visitor
    {
        for f in _fields {
            let var = Var::new(f, &self.vars);
            if var.value.is_some() {
                self.stack.push(var)
            }
        }
        self.deserialize_map(visitor)
    }

    forward_to_deserialize! {
        bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string unit seq
        seq_fixed_size bytes byte_buf map unit_struct tuple_struct
        struct_field tuple ignored_any option newtype_struct enum
    }
}

/// visits env map
struct MapVisitor<'a> {
    de: &'a mut Deserializer,
}

impl<'a> MapVisitor<'a> {
    fn new(de: &'a mut Deserializer) -> MapVisitor<'a> {
        MapVisitor { de: de }
    }
}

impl<'a> de::MapVisitor for MapVisitor<'a> {
    type Error = Error;

    fn visit_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
        where K: de::DeserializeSeed
    {
        match self.de.stack.pop() {
            Some(var) => {
                // we have a var!...
                self.de.stack.push(var.clone());
                seed.deserialize(var.struct_field.into_deserializer()).map(Some)
                //Ok(Some(try!(de::Deserialize::deserialize(&mut var.struct_field
                //    .into_deserializer()))))
            }
            _ => Ok(None),
        }
    }

    fn visit_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
        where V: de::DeserializeSeed
    {
        match self.de.stack.pop() {
            Some(Var { value: val, struct_field: field, .. }) => {
                match val {
                    Some(resolved) => {
                        let mut de = ValDeserializer {
                            de: self.de,
                            value: resolved,
                        };
                        seed.deserialize(&mut de)
                    }
                    _ => Err(Error::MissingValue(field))
                }
            }
            _ => unreachable!(),
        }
    }
}

/// Deserializes a type based on information stored in env
pub fn from_env<T>() -> Result<T>
    where T: de::Deserialize
{
    from_iter(::std::env::vars())
}

/// Deserializes a type based on an iterable of `(String, String)`
pub fn from_iter<Iter, T>(iter: Iter) -> Result<T>
    where T: de::Deserialize,
          Iter: Iterator<Item = (String, String)>
{
    let mut vars = HashMap::new();
    for (k, v) in iter {
        vars.insert(k, v);
    }
    let mut deser = Deserializer::new(vars);
    let value = try!(de::Deserialize::deserialize(&mut deser));
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::Var;
    use std::collections::HashMap;
    #[test]
    fn var_name_format() {
        let mut vars = HashMap::new();
        vars.insert(String::from("FOO_BAR"), String::from("BAR"));
        let var = Var::new("foo_bar", &vars);
        assert_eq!(var.key, String::from("FOO_BAR"))
    }
}
