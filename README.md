# envy [![Build Status](https://travis-ci.org/softprops/envy.svg?branch=master)](https://travis-ci.org/softprops/envy) [![Coverage Status](https://coveralls.io/repos/github/softprops/envy/badge.svg?branch=master)](https://coveralls.io/github/softprops/envy?branch=master) [![Software License](https://img.shields.io/badge/license-MIT-brightgreen.svg)](LICENSE) [![crates.io](http://meritbadge.herokuapp.com/envy)](https://crates.io/crates/envy)

> deserialize env vars into typesafe structs

## [Documentation](https://softprops.github.io/envy)

## install

Add the following to your Cargo.toml fails_with_invalid_type

```toml
[dependencies]
envy = "0.2"
```

## usage

assuming your rust program looks something like this.

```rust
#[macro_use]
extern crate serde_derive;
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
    match envy::from_env::<Config>() {
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

Envy assumes an env var exists for each struct field with a matching name in all uppercase letters. i.e. A struct field `foo_bar` would map to an env var named `FOO_BAR`

Structs with `Option` type fields will successfully be deserialized when their associated env var is absent.

Envy also supports deserializing `Vecs` from comma separated env var values.

Because envy is built on top of serde, you take use all of serde's [annotations](https://github.com/serde-rs/serde#annotations) to your advantage

For instance let's say you're app requires a field but would like a sensible default when one is not provided.
```rust

/// provides default value for zoom if ZOOM env var is not set
fn default_zoom() -> {
  32
}

#[derive(Deserialize, Debug)]
struct Config {
  foo: u16,
  bar: bool,
  baz: String,
  boom: Option<u64>,
  #[serde(default="default_zoom")]
  zoom: u16
}
```

The following will yield an application configured with a zoom of 32

```bash
$ FOO=8080 BAR=true BAZ=hello yourapp
```

The following will yield an application configured with a zoom of 10

```bash
$ FOO=8080 BAR=true BAZ=hello ZOOM=10 yourapp
```

## potential areas of improvement

* deserializing enums

* error handling/reporting

Doug Tangren (softprops) 2016
