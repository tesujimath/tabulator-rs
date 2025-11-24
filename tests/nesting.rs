#[allow(unused_imports)] // unsure why there's otherwise a warning here
use joinery::Joinable;
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

const SPACE_MEDIUM: &str = "  ";
