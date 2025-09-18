use super::Cell;

fn anchor_units(s: String) -> Cell<'static> {
    let idx = s.len() - 1;
    Cell::anchored(s, idx)
}

impl From<i8> for Cell<'static> {
    fn from(value: i8) -> Self {
        anchor_units(value.to_string())
    }
}

impl From<i16> for Cell<'static> {
    fn from(value: i16) -> Self {
        anchor_units(value.to_string())
    }
}

impl From<i32> for Cell<'static> {
    fn from(value: i32) -> Self {
        anchor_units(value.to_string())
    }
}

impl From<i64> for Cell<'static> {
    fn from(value: i64) -> Self {
        anchor_units(value.to_string())
    }
}

impl From<i128> for Cell<'static> {
    fn from(value: i128) -> Self {
        anchor_units(value.to_string())
    }
}

impl From<isize> for Cell<'static> {
    fn from(value: isize) -> Self {
        anchor_units(value.to_string())
    }
}

impl From<u8> for Cell<'static> {
    fn from(value: u8) -> Self {
        anchor_units(value.to_string())
    }
}

impl From<u16> for Cell<'static> {
    fn from(value: u16) -> Self {
        anchor_units(value.to_string())
    }
}

impl From<u32> for Cell<'static> {
    fn from(value: u32) -> Self {
        anchor_units(value.to_string())
    }
}

impl From<u64> for Cell<'static> {
    fn from(value: u64) -> Self {
        anchor_units(value.to_string())
    }
}

impl From<u128> for Cell<'static> {
    fn from(value: u128) -> Self {
        anchor_units(value.to_string())
    }
}

impl From<usize> for Cell<'static> {
    fn from(value: usize) -> Self {
        anchor_units(value.to_string())
    }
}

fn anchor_decimal(s: String) -> Cell<'static> {
    let idx = s
        .find('.')
        .map(|idx| idx - 1)
        .unwrap_or_else(|| s.len() - 1);
    Cell::anchored(s, idx)
}

impl From<f32> for Cell<'static> {
    fn from(value: f32) -> Self {
        anchor_decimal(value.to_string())
    }
}

impl From<f64> for Cell<'static> {
    fn from(value: f64) -> Self {
        anchor_decimal(value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use crate::Cell;

    macro_rules! check_anchor {
        ($value:expr, $idx:expr) => {{
            match (&$value, &$idx) {
                (value_val, idx_val) => {
                    let cell = Into::<Cell>::into(*value_val);
                    if let Cell::Anchored(cell_s, cell_idx) = cell {
                        assert_eq!(cell_s, value_val.to_string());
                        assert_eq!(cell_idx, *idx_val);
                    } else {
                        panic!("expected anchored found {:?}", &cell);
                    }
                }
            }
        }};
    }

    #[test]
    fn test_anchors() {
        check_anchor!(13u8, 1);
        check_anchor!(131u16, 2);
        check_anchor!(12345678u32, 7);
        check_anchor!(123456781u64, 8);
        check_anchor!(1234567812u128, 9);
        check_anchor!(1001usize, 3);

        check_anchor!(73i8, 1);
        check_anchor!(-73i8, 2);
        check_anchor!(789i16, 2);
        check_anchor!(-789i16, 3);
        check_anchor!(100302i32, 5);
        check_anchor!(-100302i32, 6);
        check_anchor!(1003023i64, 6);
        check_anchor!(-1003023i64, 7);
        check_anchor!(10030234i128, 7);
        check_anchor!(-10030234i128, 8);
        check_anchor!(10030213isize, 7);
        check_anchor!(-10030213isize, 8);

        check_anchor!(15.67f32, 1);
        check_anchor!(-15.67f32, 2);
        check_anchor!(101f32, 2);
        check_anchor!(-101f32, 3);

        check_anchor!(15.67f64, 1);
        check_anchor!(-15.67f64, 2);
        check_anchor!(101f64, 2);
        check_anchor!(-101f64, 3);
    }
}
