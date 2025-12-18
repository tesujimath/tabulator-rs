build:
    cargo build

test: build rust-tests json-tests

rust-tests:
    cargo test
    cargo test --features rust_decimal
    cargo test --features num-bigint

json-tests:
    #!/usr/bin/env bash
    set -euxo pipefail
    for jsonfile in json-tests/*.json; do
        expected="${jsonfile%%.json}.expected"
        cat $jsonfile | tabulator | diff - $expected
    done
