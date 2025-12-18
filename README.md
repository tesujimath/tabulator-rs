# tabulator

This is grid-style tabulation for Rust, for left/right/centre justification of strings and decimal point alignment.

The motivation to build such a thing is [Beancount Lima](https://github.com/tesujimath/beancount-lima).

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

## Features

There are no default features.

Optional features are:

- `rust_decimal` - adds a dependency on that crate and `From::<Decimal>` for `Cell`
- `num-bigint` - adds a dependency on that crate and `From::<BigInt>` and `From::<BigUint>` for `Cell`
- `json` - adds `Cell::from_json(s: &str)`

## License

Licensed under either of

 * Apache License, Version 2.0
   [LICENSE-APACHE](http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   [LICENSE-MIT](http://opensource.org/licenses/MIT)

at your option.
