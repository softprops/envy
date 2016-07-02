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
FOO=8080 BAR=true BAZ=hello yourapp
```

You should be able to access a completely typesafe config struct deserialized from env vars



Doug Tangren (softprops) 2016
