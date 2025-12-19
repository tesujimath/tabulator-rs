build:
    cargo build --features=bin

test: build rust-tests json-tests psv-tests psvf-tests

rust-tests:
    cargo test
    cargo test --features rust_decimal
    cargo test --features num-bigint

json-tests:
    #!/usr/bin/env bash
    set -euxo pipefail
    for jsonfile in cli-tests/*.json; do
        expected="${jsonfile%%.json}.expected"
        cat $jsonfile | tabulator -f json | diff - $expected
    done

psv-tests:
    #!/usr/bin/env bash
    set -euxo pipefail
    for psvfile in cli-tests/*.psv; do
        expected="${psvfile%%.psv}.expected"
        cat $psvfile | tabulator -f psv | diff - $expected
    done

psvf-tests:
    #!/usr/bin/env bash
    set -euxo pipefail
    for psvffile in cli-tests/*.psvf; do
        expected="${psvffile%%.psvf}.expected"
        cat $psvffile | tabulator -f psvf | diff - $expected
    done
