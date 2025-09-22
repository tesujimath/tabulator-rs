#[allow(unused_imports)] // unsure why there's otherwise a warning here
use joinery::Joinable;
use tabulator::{layout, Align, Cell, Layout, Spacing, Style};
use test_case::test_case;
use Align::*;
use Cell::*;

#[test_case(("Letsa go Mario!", Left).into(), r#"Letsa go Mario!"#)]
#[test_case(Row(vec![("Letsa go Mario!", Left).into(), ("OK", Left).into()]), r#"Letsa go Mario!  OK"#)]
fn default_style(cell: Cell, expected: &str) {
    let result = cell.to_string();
    assert_eq!(&result, expected);
}

#[test_case(vec![
        vec!["A123", "B", "C1" ],
        vec!["D", "E1", "F999" ],
    ], r#"A123  B   C1  
D     E1  F999"#)]
fn left_justified_strings(rows: Vec<Vec<&str>>, expected: &str) {
    let cell = Column(
        rows.iter()
            .map(|row| {
                Row(row
                    .iter()
                    .map(|s| (s.to_string(), Left).into())
                    .collect::<Vec<_>>())
            })
            .collect::<Vec<_>>(),
    );
    let result = cell.to_string();
    assert_eq!(result, expected);
}

#[test_case(vec![
        vec!["A123", "B", "C1" ],
        vec!["D", "E1", "F999" ],
    ], r#"A123  B   C1  
D     E1  F999"#)]
fn left_justified_strs(rows: Vec<Vec<&str>>, expected: &str) {
    let cell = Column(
        rows.iter()
            .map(|row| Row(row.iter().map(|s| (*s, Left).into()).collect::<Vec<_>>()))
            .collect::<Vec<_>>(),
    );
    let result = cell.to_string();
    assert_eq!(result, expected);
}

#[test_case(vec![
        vec!["A123", "B", "C1" ],
        vec!["D", "E1", "F999" ],
    ], r#"A123   B    C1
   D  E1  F999"#)]
fn right_justified_strs(rows: Vec<Vec<&str>>, expected: &str) {
    let cell = Column(
        rows.iter()
            .map(|row| Row(row.iter().map(|s| (*s, Right).into()).collect::<Vec<_>>()))
            .collect::<Vec<_>>(),
    );
    let result = cell.to_string();
    assert_eq!(result, expected);
}

#[test_case(vec![
        Vec::default(),
        vec!["A123", "B", "C1" ],
        Vec::default(),
        vec!["D", "E1", "F999" ],
        vec!["G", "H" ],
        Vec::default(),
    ], vec![
        "              ",
        "A123   B    C1",
        "              ",
        "   D  E1  F999",
        "   G   H      ",
        "              ",
    ]
)]
fn empty_lines_space_filled(rows: Vec<Vec<&str>>, expected_lines: Vec<&str>) {
    let cell = Column(
        rows.iter()
            .map(|row| Row(row.iter().map(|s| (*s, Right).into()).collect::<Vec<_>>()))
            .collect::<Vec<_>>(),
    );
    let result = cell.to_string();
    let expected = expected_lines.join_with("\n").to_string();
    assert_eq!(&result, &expected);
}

#[test_case(vec![
        vec!["A123", "B", "C1" ],
        vec!["D", "E1", "F999" ],
    ], r#"A123  B    C1 
 D    E1  F999"#)]
fn centred_strs(rows: Vec<Vec<&str>>, expected: &str) {
    let cell = Column(
        rows.iter()
            .map(|row| Row(row.iter().map(|s| (*s, Centre).into()).collect::<Vec<_>>()))
            .collect::<Vec<_>>(),
    );
    let result = cell.to_string();
    assert_eq!(result, expected);
}

#[test_case(Column(vec![
        Row(vec![("A", Left).into(), Cell::anchored("1.25", 1), ("A99", Right).into()]),
        Row(vec![("B1", Left).into(), Cell::anchored("12.2", 2), ("B", Right).into()]),
    ]), vec![
        "A    1.25  A99",
        "B1  12.2     B",
        ]
)]
fn anchored(cell: Cell, expected_lines: Vec<&str>) {
    let result = cell.to_string();
    let expected = expected_lines.join_with("\n").to_string();
    assert_eq!(&result, &expected);
}

#[test_case(Column(vec![
        Row(vec![Row(vec![("A1", Left).into(), ("B1", Left).into()]), ("C1", Left).into()]),
        Row(vec![("A2", Left).into(), ("B", Right).into(), ("C", Left).into()]),
    ]), vec![
        "A1  B1  C1   ",
        "A2       B  C",
        ]
)]
fn merge_without_anchor(cell: Cell, expected_lines: Vec<&str>) {
    let result = cell.to_string();
    let expected = expected_lines.join_with("\n").to_string();
    assert_eq!(&result, &expected);
}

#[test_case(Column(vec![
        Row(vec![Row(vec![("A1", Left).into(), ("B1", Left).into()]), ("C1a", Left).into(), ("D1-abc", Left).into()]),
        Row(vec![("A2", Left).into(), Cell::anchored("12.50", 2), ("D", Left).into()]),
        Row(vec![("A3", Left).into(), Cell::anchored("17.305", 2), ("D3", Right).into()]),
        Row(vec![("A4", Right).into(), ("C4", Right).into(), ("D4", Right).into()]),
    ]), vec![
        "A1  B1  C1a     D1-abc",
        "A2      12.50   D     ",
        "A3      17.305      D3",
        "    A4      C4      D4",
        ]
)]
fn merge_with_anchor(cell: Cell, expected_lines: Vec<&str>) {
    let result = cell.to_string();
    let expected = expected_lines.join_with("\n").to_string();
    assert_eq!(&result, &expected);
}

#[test_case(Row(vec![Row(vec![("A", Left).into(), ("B", Left).into()]),Row(vec![("C", Left).into(), ("D", Left).into()])]),
        r#"A||B||C||D"#)]
fn styled(cell: Cell, expected: &str) {
    use Style::*;
    let layout = Layout::default();

    let result = cell.layout(&layout, Piped).to_string();
    assert_eq!(&result, expected);
}

#[test_case(Column(vec![
        Row(vec![("A", Left).into(), Row(vec![("B1", Left).into(), ("C1", Left).into()]), Row(vec![("D1a", Left).into(), ("E1-abc", Left).into()]), ("F", Right).into()]),
        Row(vec![("A2", Left).into(), Row(vec![("B1", Left).into(), ("C1", Left).into()]), Row(vec![("D1a", Left).into(), ("E1-abc", Left).into()]), ("F2", Right).into()]),
    ]), vec![
        "A    B1  C1   D1a E1-abc    F",
        "A2   B1  C1   D1a E1-abc   F2",
        ]
)]
fn simple_nested_layout(cell: Cell, expected_lines: Vec<&str>) {
    use Spacing::*;

    let layout = layout!([Major; _, Medium, Minor]);
    // println!("layout {:?}", &layout);
    let result = cell.layout(&layout, Style::default()).to_string();
    let expected = expected_lines.join_with("\n").to_string();
    assert_eq!(&result, &expected);
}

#[test_case(Column(vec![
        Row(vec![
            ("A1", Left).into(),
            Row(vec![
                ("B1", Left).into(),
                Row(vec![
                    ("C1", Left).into(),
                    Row(vec![
                        ("X1", Left).into(),
                        ("Y1", Left).into()]),
                    ("D1", Left).into()]),
                ("E1", Left).into()]),
            Row(vec![
                ("F1", Left).into(),
                ("G1", Left).into()]),
            ("H1", Right).into()]),
    ]), vec![
        "A1   B1  C1 X1Y1 D1  E1   F1 G1   H1",
        ]
)]
fn doubly_nested_layout(cell: Cell, expected_lines: Vec<&str>) {
    use Spacing::*;

    let layout = layout!([ Major;
        _,
        [
            Medium;
                _,
                [
                    Minor;
                    _, Flush, _
                ],
                _
            ],
        Minor,
        _
    ]);
    // println!("layout {:?}", &layout);
    let result = cell.layout(&layout, Style::default()).to_string();
    let expected = expected_lines.join_with("\n").to_string();
    assert_eq!(&result, &expected);
}

#[test_case(
        Row(vec![
            ("A", Left).into(),
            Row(vec![
                ("BA", Left).into(),
                Row(vec![
                    ("BBA", Left).into(),
                    ("BBB", Left).into()]),
               ("BC", Left).into()]),
            Row(vec![
                ("CA", Left).into(),
                Row(vec![
                    ("CBA", Left).into(),
                    ("CBB", Left).into()]),
                ("CC", Left).into()]),
            Row(vec![
                ("DA", Left).into(),
                ("DB", Left).into()]),
            ("X", Right).into()]),
    vec![
        "A   BA  BBA BBB  BC   CA  CBA CBB  CC   DA  DB   X",
        ]
)]
fn nested_layout_repeat(cell: Cell, expected_lines: Vec<&str>) {
    use Spacing::*;

    let layout = layout!([Major; _, [Medium; Minor]]);
    // println!("layout {:?}", &layout);
    let result = cell.layout(&layout, Style::default()).to_string();
    let expected = expected_lines.join_with("\n").to_string();
    assert_eq!(&result, &expected);
}
