#[allow(unused_imports)] // unsure why there's otherwise a warning here
use joinery::Joinable;
#[cfg(feature = "rust_decimal")]
use rust_decimal_macros::dec;
use tabulator::{Align, Cell};
use test_case::test_case;
use Align::*;
use Cell::*;

#[test_case(
    Stack(vec![
        Row(vec![
            Stack(vec![("A1", Left).into(), ("A2", Left).into()]),
            ("B", Left).into(),
            Stack(vec![("C1", Left).into(), ("C2", Left).into(), ("C3", Left).into()]),
            ], SPACE_MEDIUM),
        Row(vec![
                ("D", Left).into(),
                ("E", Left).into(),
                ("F", Left).into(),
            ], SPACE_MEDIUM),
    ]),
    vec![
        "A1  B  C1",
        "A2     C2",
        "       C3",
        "D   E  F ",
        ]
)]
fn nested_columns(cell: Cell, expected_lines: Vec<&str>) {
    let result = cell.to_string();
    let expected = expected_lines.join_with("\n").to_string();
    assert_eq!(&result, &expected);
}

#[cfg(feature = "rust_decimal")]
#[test_case(
    Stack(vec![
        Row(vec![dec!(1.25).into(), ("B1", Left).into()], SPACE_MEDIUM),
        Row(vec![dec!(13.5).into(), ("B2", Left).into()], SPACE_MEDIUM),
        Stack( vec![
            Row(vec![dec!(100.2).into(), ("B3", Left).into()], SPACE_MEDIUM),
            Row(vec![dec!(3.125).into(), ("B4", Left).into()], SPACE_MEDIUM),
        ]),
    ]),
    vec![
        "  1.25   B1",
        " 13.5    B2",
        "100.2    B3",
        "  3.125  B4",
        ]
)]
#[test_case(
    Stack(vec![
        Row(vec![("A1", Left).into(), Row(vec![dec!(1.25).into(), ("B1", Left).into()], SPACE_MEDIUM)], SPACE_MEDIUM),
        Row(vec![("A2", Left).into(), Row(vec![dec!(13.5).into(), ("B2", Left).into()], SPACE_MEDIUM)], SPACE_MEDIUM),
        Row(vec![("A3", Left).into(), Stack( vec![
            Row(vec![dec!(100.2).into(), ("B3", Left).into()], SPACE_MEDIUM),
            Row(vec![dec!(3.125).into(), ("B4", Left).into()], SPACE_MEDIUM),
            Row(vec![dec!(7.50).into(), ("B5", Left).into(), Row(vec![("C5abc", Right).into(), ("D5", Right).into()], SPACE_MEDIUM)], SPACE_MEDIUM),
            Row(vec![dec!(12.5).into(), ("B6", Left).into(), Row(vec![("C6", Right).into(), ("D6a", Right).into()], SPACE_MEDIUM)], SPACE_MEDIUM),
        ])], SPACE_MEDIUM),
    ]),
    vec![
        "A1    1.25   B1            ",
        "A2   13.5    B2            ",
        "A3  100.2    B3            ",
        "      3.125  B4            ",
        "      7.50   B5  C5abc   D5",
        "     12.5    B6     C6  D6a",
        ]
)]
fn nested_decimals(cell: Cell, expected_lines: Vec<&str>) {
    let result = cell.to_string();
    let expected = expected_lines.join_with("\n").to_string();
    assert_eq!(&result, &expected);
}

const SPACE_MEDIUM: &str = "  ";
