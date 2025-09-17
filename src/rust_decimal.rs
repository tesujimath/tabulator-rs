use std::borrow::Cow;

use rust_decimal::Decimal;

use crate::Cell;

impl<'a> From<Decimal> for Cell<'a> {
    fn from(value: Decimal) -> Self {
        let sign_width = if value.is_sign_negative() { 1u32 } else { 0 };
        let mut mantissa_width = 0u32;
        let mut abs_mantissa = value.mantissa().abs();
        while abs_mantissa > 0 {
            abs_mantissa /= 10;
            mantissa_width += 1;
        }

        let idx = if sign_width + mantissa_width > value.scale() {
            sign_width + mantissa_width - value.scale()
        } else {
            1
        };

        Cell::Anchor(Cow::Owned(value.to_string()), idx as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::Cell;
    use rust_decimal::Decimal;
    use test_case::test_case;
    use Cell::*;

    #[test_case("1.23", 1)]
    #[test_case("-1.23", 2)]
    #[test_case("1001.234", 4)]
    fn decimal_anchor(s: &str, expected_idx: usize) {
        let d = s.parse::<Decimal>().unwrap();
        let result = Into::<Cell>::into(d);
        if let Anchor(result_s, result_idx) = result {
            assert_eq!(result_s, s);
            assert_eq!(result_idx, expected_idx);
        } else {
            panic!("expected anchor, got {:?}", result);
        }
    }
}
