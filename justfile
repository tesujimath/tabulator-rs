build:
    cargo build

test: build
    cargo test
    cargo test --features rust_decimal
