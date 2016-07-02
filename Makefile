build:
	@cargo build

test:
	@cargo test --manifest-path serde-tests/Cargo.toml
	@cargo test
