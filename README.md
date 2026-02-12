# tabulator

This is grid-style tabulation for left/right/centre justification of strings and decimal point alignment.

It is available both as a Rust crate and a command line program, `tabulator`.

The motivation to build such a thing is [limabean](https://github.com/tesujimath/limabean), a new implementation of Beancount in Rust and Clojure.

## Example Output

### Simple left/right and decimal point alignment

```text
A   1.25 A99
B1 12.5    B
```

### Using `rust_decimal` auto-anchor

```text
Assets:Bank:Current    350.75 NZD Howzah!
Assets:Bank:Investment   2.25 NZD   Skint
```

## tabulator Command Line Usage

`tabulator` receives table layout on standard input, as one of the supported formats:

- JSON - all tabulation features available, with flexible alignment options and nesting of cells
- PSV - pipe separated values, with leading/trailing whitespace stripped, and lines split on pipe
- PSVF - framed pipe separated values, with leading/trailing whitespace stripped, after which each line is required to begin and end with pipe, and lines split on pipe

### Example

```shell
$ tabulator -f psv <<EOF
A1|B|C1
A2|C2
|C3
D|E|F
A3|17.305|D3
A4|1.5|D4
EOF

A1  B       C1
A2  C2
    C3
D   E       F
A3  17.305  D3
A4   1.5    D4

```

## Crate Features

There are no default features.

Optional features are:

- `rust_decimal` - adds a dependency on that crate and `From::<Decimal>` for `Cell`
- `num-bigint` - adds a dependency on that crate and `From::<BigInt>` and `From::<BigUint>` for `Cell`
- `json` - adds `Cell::from_json(s: &str)`
- `psv` - adds `Cell::from_psv(s: &str, ...)` and `Cell::from_psvf(s: &str, ...)`

Note that the `clap` feature is not useful as a library feature, it purely supports the binary.

## License

Licensed under either of

 * Apache License, Version 2.0
   [LICENSE-APACHE](http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   [LICENSE-MIT](http://opensource.org/licenses/MIT)

at your option.
