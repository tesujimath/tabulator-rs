use crate::{conversions::anchor_units, Cell};
use num_bigint::{BigInt, BigUint};

impl From<&BigUint> for Cell<'static> {
    // anchor the decimal at the units digit, so will align with e.g. integers
    fn from(value: &BigUint) -> Self {
        anchor_units(value.to_string())
    }
}

impl From<&BigInt> for Cell<'static> {
    // anchor the decimal at the units digit, so will align with e.g. integers
    fn from(value: &BigInt) -> Self {
        anchor_units(value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use crate::Cell;
    use num_bigint::{BigInt, BigUint};
    use test_case::test_case;
    use Cell::*;

    #[test_case("123", 2)]
    #[test_case("12345678901234567890123", 22)]
    fn test_biguint(s: &str, expected_idx: usize) {
        let x = s.parse::<BigUint>().unwrap();
        let result = Into::<Cell>::into(&x);
        if let Anchored(result_s, result_idx) = result {
            assert_eq!(result_s, s);
            assert_eq!(result_idx, expected_idx);
        } else {
            panic!("expected anchor, got {:?}", result);
        }
    }

    #[test_case("123", 2)]
    #[test_case("-123", 3)]
    #[test_case("12345678901234567890123", 22)]
    #[test_case("-12345678901234567890123", 23)]
    fn test_bigint(s: &str, expected_idx: usize) {
        let x = s.parse::<BigInt>().unwrap();
        let result = Into::<Cell>::into(&x);
        if let Anchored(result_s, result_idx) = result {
            assert_eq!(result_s, s);
            assert_eq!(result_idx, expected_idx);
        } else {
            panic!("expected anchor, got {:?}", result);
        }
    }
}
