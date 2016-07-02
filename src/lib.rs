//! envy is a library for deserializing env vars into typesafe structs

extern crate serde;

use serde::de;
use std::collections::HashMap;
use serde::de::value::ValueDeserializer;

mod errors;
pub use errors::Error;

/// A type result type specific to `envy::Errors`
pub type Result<T> = std::result::Result<T, Error>;

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

    fn visit<T>(&mut self) -> Result<Option<T>>
        where T: de::Deserialize
    {
        match self.values.pop() {
            Some(val) => {
                let mut de = ValDeserializer {
                    de: self.de,
                    value: val.clone(),
                };
                Ok(Some(try!(de::Deserialize::deserialize(&mut de))))
            }
            _ => Ok(None),
        }
    }

    fn end(&mut self) -> Result<()> {
        Ok(())
    }
}

/// deserializes a single env string value
struct ValDeserializer<'a> {
    de: &'a mut Deserializer,
    value: String,
}

impl<'a> de::Deserializer for ValDeserializer<'a> {
    type Error = Error;
    fn deserialize<V>(&mut self, mut visitor: V) -> Result<V::Value>
        where V: de::Visitor
    {
        visitor.visit_string(self.value.clone())
    }

    fn deserialize_seq<V>(&mut self, mut visitor: V) -> Result<V::Value>
        where V: de::Visitor
    {
        let mut values = self.value.split(",").map(|v| v.to_string()).collect::<Vec<String>>();
        values.reverse();
        visitor.visit_seq(SeqVisitor::new(self.de, values))
    }

    fn deserialize_struct<V>(&mut self,
                             _name: &'static str,
                             _fields: &'static [&'static str],
                             visitor: V)
                             -> Result<V::Value>
        where V: de::Visitor
    {
        self.de.deserialize_struct(_name, _fields, visitor)
    }
}

/// deserializer for env vars
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

impl de::Deserializer for Deserializer {
    type Error = Error;
    fn deserialize<V>(&mut self, mut visitor: V) -> Result<V::Value>
        where V: de::Visitor
    {
        visitor.visit_map(MapVisitor::new(self))
    }

    fn deserialize_struct<V>(&mut self,
                             _name: &'static str,
                             _fields: &'static [&'static str],
                             visitor: V)
                             -> Result<V::Value>
        where V: de::Visitor
    {
        for f in _fields {
            self.stack.push(Var::new(f, &self.vars))
        }
        self.deserialize_map(visitor)
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
    fn visit_key<K>(&mut self) -> Result<Option<K>>
        where K: de::Deserialize
    {
        match self.de.stack.pop() {
            Some(var) => {
                self.de.stack.push(var.clone());
                Ok(Some(try!(de::Deserialize::deserialize(&mut var.struct_field
                    .into_deserializer()))))
            }
            _ => Ok(None),
        }
    }

    fn visit_value<V>(&mut self) -> Result<V>
        where V: de::Deserialize
    {
        match self.de.stack.pop() {
            Some(Var { value: val, struct_field: field, .. }) => {
                match val {
                    Some(resolved) => {
                        let mut de = ValDeserializer {
                            de: self.de,
                            value: resolved,
                        };
                        Ok(try!(de::Deserialize::deserialize(&mut de)))
                    }
                    _ => self.missing_field(field),
                }
            }
            _ => Err(Error::MissingValue("fixme")),
        }
    }

    fn end(&mut self) -> Result<()> {
        Ok(())
    }

    fn missing_field<V>(&mut self, field: &'static str) -> Result<V>
        where V: de::Deserialize
    {
        struct MissingFieldDeserializer(&'static str);

        impl de::Deserializer for MissingFieldDeserializer {
            type Error = Error;//de::value::Error;

            fn deserialize<V>(&mut self, _visitor: V) -> Result<V::Value>
                where V: de::Visitor
            {
                let &mut MissingFieldDeserializer(field) = self;
                Err(Error::MissingValue(field))
            }

            fn deserialize_option<V>(&mut self, mut visitor: V) -> Result<V::Value>
                where V: de::Visitor
            {
                visitor.visit_none()
            }
        }

        let mut de = MissingFieldDeserializer(field);
        Ok(try!(de::Deserialize::deserialize(&mut de)))
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
