# envy [![Build Status](https://travis-ci.org/softprops/envy.svg?branch=master)](https://travis-ci.org/softprops/envy) [![Coverage Status](https://coveralls.io/repos/github/softprops/envy/badge.svg?branch=master)](https://coveralls.io/github/softprops/envy?branch=master)

> deserialize env vars into typesafe structs

## [Documentation](https://softprops.github.io/envy)

## usage

assuming you're rust program looks something like this


```rust
// extern crate serde and other imports...
extern crate envy;

use std::env;

#[derive(Deserialize, Debug)]
struct Config {
  foo: u16,
  bar: bool,
  baz: String,
  boom: Option<u64>
}

fn main() {
    match envy::from_env::<Config>(env::vars()) {
       Ok(config) => println!("{:#?}", config)
       Err(error) => panic!("{:#?}", error)
    }
}
```

export some environment variables

```bash
$ FOO=8080 BAR=true BAZ=hello yourapp
```

You should be able to access a completely typesafe config struct deserialized from env vars.

Envy assumes env assumes an env var exists for each struct field with a matching name in all uppercase letters. i.e. A struct field `foo_bar` would map to an env var named `FOO_BAR`

Structs with `Option` type fields will successfully be deserialized when their associated env var is absent.

Envy also supports deserializing `Vecs` from comma separated env var values.

## potential areas of improvement

* deserializing enums

* error handling/reporting

Doug Tangren (softprops) 2016
