# tabulator

This is grid-style tabulation for Rust, for left/right/centre justification of strings and decimal point alignment.

The motivation to build such a thing is [Beancount Lima](https://github.com/tesujimath/beancount-lima).

## Example Output

### Simple case with manual anchoring

```text
A   1.25 A99
B1 12.2    B
```

### Using `rust_decimal` auto-anchor

```text
Assets:Bank:Current    350.75 NZD Howzah!
Assets:Bank:Investment   2.25 NZD   Skint
```

## Features

There are no default features.

Optional features are:

- `rust_decimal` - adds a dependency on that crate and `From::<rust_decimal::Decimal>` for `Cell`

## License

Licensed under either of

 * Apache License, Version 2.0
   [LICENSE-APACHE](http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   [LICENSE-MIT](http://opensource.org/licenses/MIT)

at your option.
