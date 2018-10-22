# 0.3.3 (unreleased)

* update `from_iter(..)` to accept `std::iter::IntoIterator` types

This is a backwards compatible change because all Iterators have a [provided impl for IntoInterator](https://doc.rust-lang.org/src/core/iter/traits.rs.html#255-262) by default.

# 0.3.2

* add new `envy::prefixed(...)` interface for prefixed env var names

# 0.3.1

* fix option support

# 0.3.0

* upgrade to the latest serde (1.0)

# 0.2.0

* upgrade to the latest serde (0.9)

# 0.1.2

* upgrade to latest serde (0.8)

# 0.1.1 (2016-07-10)

* allow for customization via built in serde [field annotations](https://github.com/serde-rs/serde#annotations)

# 0.1.0 (2016-07-02)

* initial release
