use crate::Cell;
use rust_decimal::Decimal;

impl<'a> From<Decimal> for Cell<'a> {
    // anchor the decimal at the units digit, so will align with e.g. integers
    fn from(value: Decimal) -> Self {
        let sign_width = if value.is_sign_negative() { 1u32 } else { 0 };
        let mut mantissa_width = 0u32;
        let mut abs_mantissa = value.mantissa().abs();
        while abs_mantissa > 0 {
            abs_mantissa /= 10;
            mantissa_width += 1;
        }

        let idx = if sign_width + mantissa_width > value.scale() {
            sign_width + mantissa_width - value.scale() - 1
        } else {
            0
        };

        Cell::anchored(value.to_string(), idx as usize)
    }
}

#[cfg(test)]
mod tests {
    use crate::{Align, Cell};
    use joinery::Joinable;
    use rust_decimal::Decimal;
    use test_case::test_case;
    use Align::*;
    use Cell::*;

    #[test_case("1.23", 0)]
    #[test_case("-1.23", 1)]
    #[test_case("1001.234", 3)]
    fn decimal_anchor(s: &str, expected_idx: usize) {
        let d = s.parse::<Decimal>().unwrap();
        let result = Into::<Cell>::into(d);
        if let Anchored(result_s, result_idx) = result {
            assert_eq!(result_s, s);
            assert_eq!(result_idx, expected_idx);
        } else {
            panic!("expected anchor, got {:?}", result);
        }
    }

    #[test_case(Stack(vec![
        Row(vec![("Assets:Bank:Current", Left).into(), "350.75".parse::<Decimal>().unwrap().into(), ("NZD", Left).into(), ("Howzah!", Right).into()], SMALL_SPACE),
        Row(vec![("Assets:Bank:Investment", Left).into(), "2.25".parse::<Decimal>().unwrap().into(), ("NZD", Left).into(), ("Skint", Right).into()], SMALL_SPACE),
    ]), vec![
        "Assets:Bank:Current    350.75 NZD Howzah!",
        "Assets:Bank:Investment   2.25 NZD   Skint",
        ]
)]
    fn bank_accounts(cell: Cell, expected_lines: Vec<&str>) {
        let result = cell.to_string();
        let expected = expected_lines.join_with("\n").to_string();
        assert_eq!(&result, &expected);
    }

    const SMALL_SPACE: &str = " ";
}
