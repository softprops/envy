extern crate serde;

use serde::de;
use std::collections::HashMap;
use serde::de::value::ValueDeserializer;

mod errors;
pub use errors::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug)]
struct Var {
    key: String,
    struct_field: String,
    value: Option<String>,
}

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

pub fn from_env<T>() -> Result<T>
    where T: de::Deserialize
{
    let mut vars = std::collections::HashMap::new();
    for (k, v) in ::std::env::vars() {
        vars.insert(k, v);
    }
    let mut deser = Deserializer::new(vars);
    let value = try!(de::Deserialize::deserialize(&mut deser));
    Ok(value)
}

impl de::Deserializer for Deserializer {
    type Error = Error;
    fn deserialize<V>(&mut self, mut visitor: V) -> Result<V::Value>
        where V: de::Visitor
    {
        visitor.visit_map(MapVisitor { de: self })
    }

    fn deserialize_struct<V>(&mut self,
                             _name: &'static str,
                             _fields: &'static [&'static str],
                             visitor: V)
                             -> Result<V::Value>
        where V: de::Visitor
    {
        for f in _fields {
            let key = f.to_string().to_uppercase();
            let value = self.vars.get(&key).map(|v| v.clone());
            self.stack.push(Var {
                key: key,
                struct_field: f.to_string(),
                value: value,
            })
        }
        self.deserialize_map(visitor)
    }
}

struct MapVisitor<'a> {
    de: &'a mut Deserializer,
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
            Some(Var { value: Some(val), .. }) => {
                Ok(try!(de::Deserialize::deserialize(&mut val.into_deserializer())))
            }
            _ => Err(Error::MissingValue),
        }
    }

    fn end(&mut self) -> Result<()> {
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
