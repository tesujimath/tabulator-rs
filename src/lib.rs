#![doc = include_str!("../README.md")]

//! # Simple example
//!
//! Generates the output as shown above.
//!
//!```
//! # use tabulator::{Align, Cell};
//!
//!fn main() {
//!    use Align::*;
//!    use Cell::*;
//!
//!    let cell = Column(vec![
//!        Row(vec![("A", Left).into(), Cell::anchored("1.25", 1), ("A99", Right).into()]),
//!        Row(vec![("B1", Left).into(), Cell::anchored("12.2", 2), ("B", Right).into()]),
//!    ]);
//!
//!    let output = cell.to_string();
//!}

use itertools::Itertools;
#[allow(unused_imports)] // unsure why there's otherwise a warning here
use joinery::Joinable;
use lazy_format::make_lazy_format;
use std::{
    cmp::max,
    {borrow::Cow, fmt::Display},
};
use unicode_width::UnicodeWidthStr;

#[derive(Debug)]
pub enum Align {
    Left,
    Right,
    Centre,
}

#[derive(Debug)]
pub enum Cell<'a> {
    Aligned(Cow<'a, str>, Align),
    Anchored(Cow<'a, str>, usize), // index of character which is anchored, e.g. the decimal point
    Row(Vec<Cell<'a>>),
    Column(Vec<Cell<'a>>),
}

impl<'a, S> From<(S, Align)> for Cell<'a>
where
    S: Into<Cow<'a, str>>,
{
    fn from(value: (S, Align)) -> Self {
        Cell::Aligned(value.0.into(), value.1)
    }
}

impl<'a> Cell<'a> {
    fn empty() -> Self {
        Cell::Aligned(Cow::Borrowed(""), Align::Left)
    }

    /// Return the string anchored at position `idx`.
    pub fn anchored<S>(s: S, idx: usize) -> Self
    where
        S: Into<Cow<'a, str>>,
    {
        Cell::Anchored(s.into(), idx)
    }
}

#[derive(Debug)]
pub struct Style<'a> {
    column_separator: Cow<'a, str>,
}

impl<'a> Default for Style<'a> {
    fn default() -> Self {
        Self {
            column_separator: Cow::Borrowed(" "),
        }
    }
}

impl<'a> Style<'a> {
    pub fn with_column_separator(column_separator: Cow<'a, str>) -> Self {
        Self { column_separator }
    }

    fn column_separator_width(&self) -> usize {
        self.column_separator.as_ref().len()
    }
}

impl<'a> Cell<'a> {
    pub fn styled<'s>(&self, style: &Style<'s>) -> impl Display {
        let spec: ColSpec = self.into();

        make_lazy_format!(|f| self.format(f, style, &spec))
    }

    fn format<'s>(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        style: &Style<'s>,
        spec: &ColSpec,
    ) -> std::fmt::Result {
        use Cell::*;
        use ColSpec::*;

        let spec_width = spec.width(style.column_separator_width());
        match (self, spec) {
            (Aligned(s, align), _spec) => {
                use Align::*;

                let total = spec_width - s.width();
                let (left, right) = match align {
                    Left => (0, total),
                    Right => (total, 0),
                    Centre => {
                        let left = total / 2;
                        (left, total - left)
                    }
                };
                write!(f, "{}{}{}", pad(left), s, pad(right))
            }
            (Anchored(s, idx), spec) => {
                let (spec_idx, spec_trailing) = spec.anchor(style.column_separator_width());
                let trailing = s.width() - idx;
                let pad_l = spec_idx - idx;
                let pad_r = spec_trailing - trailing;
                write!(f, "{}{}{}", pad(pad_l), s, pad(pad_r))
            }
            (Row(cells), Composite((_spec_s, spec_c))) => {
                use itertools::EitherOrBoth::*;

                let mut sep = false;
                let empty_cell = Cell::empty();
                for (cell, spec) in cells.iter().zip_longest(spec_c).map(|x| match x {
                    Both(cell, spec) => (cell, spec),
                    Left(_cell) => todo!("cell without spec"),
                    Right(spec) => (&empty_cell, spec),
                }) {
                    if sep {
                        write!(f, "{}", &style.column_separator)?
                    }
                    sep = true;
                    cell.format(f, style, spec)?;
                }
                Ok(())
            }
            (Column(cells), spec) => {
                let mut sep = false;
                for cell in cells {
                    if sep {
                        f.write_str("\n")?;
                    }
                    sep = true;
                    cell.format(f, style, spec)?;
                }
                Ok(())
            }
            _ => todo!("mismatched cell {:?} and spec {:?}", self, &spec),
        }
    }
}

impl<'a> Display for Cell<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.styled(&Style::default()))
    }
}

fn pad(n: usize) -> impl Display {
    make_lazy_format!(|f| {
        for _ in 0..n {
            f.write_str(" ")?;
        }
        Ok(())
    })
}

#[derive(PartialEq, Eq, Clone, Debug)]
struct SimpleColSpec {
    width: Option<usize>,
    anchor: Option<(usize, usize)>,
}

fn degenerate_anchor(width: Option<usize>) -> (usize, usize) {
    (width.unwrap_or(0), 0)
}

impl SimpleColSpec {
    fn from_width(width: usize) -> Self {
        Self {
            width: Some(width),
            anchor: None,
        }
    }

    fn from_anchor(idx: usize, trailing: usize) -> Self {
        Self {
            width: None,
            anchor: Some((idx, trailing)),
        }
    }

    fn width(&self) -> usize {
        max(
            self.width.unwrap_or(0),
            self.anchor
                .map(|(idx, trailing)| idx + trailing)
                .unwrap_or(0),
        )
    }

    fn anchor(&self) -> (usize, usize) {
        // anchor is expanded on the left to match the width
        let width = self.width();
        self.anchor
            .map_or(degenerate_anchor(Some(width)), |(idx, trailing)| {
                let expanded_idx = max(width, idx + trailing) - trailing;
                (expanded_idx, trailing)
            })
    }

    fn merge(self, other: SimpleColSpec) -> Self {
        let width = match (self.width, other.width) {
            (Some(w0), Some(w1)) => Some(max(w0, w1)),
            (w0, None) => w0,
            (None, w1) => w1,
        };

        let anchor = match (self.anchor, other.anchor) {
            (Some((idx0, trailing0)), Some((idx1, trailing1))) => {
                Some((max(idx0, idx1), max(trailing0, trailing1)))
            }
            (a0, None) => a0,
            (None, a1) => a1,
        };

        Self { width, anchor }
    }
}

#[derive(PartialEq, Eq, Clone, Debug, Default)]
enum ColSpec {
    #[default]
    Empty,
    Simple(SimpleColSpec),
    Composite((Option<SimpleColSpec>, Vec<ColSpec>)),
}

impl ColSpec {
    fn merge(self, other: ColSpec) -> Self {
        use itertools::EitherOrBoth::*;
        use ColSpec::*;

        match (self, other) {
            (Empty, s) => s,
            (s, Empty) => s,
            (Simple(s0), Simple(s1)) => Simple(s0.merge(s1)),

            (Composite((s0, c0)), Composite((s1, c1))) => {
                let s = match (s0, s1) {
                    (Some(s0), Some(s1)) => Some(s0.merge(s1)),
                    (s0, None) => s0,
                    (None, s1) => s1,
                };
                Composite((
                    s,
                    c0.into_iter()
                        .zip_longest(c1)
                        .map(|x| match x {
                            Both(c0, c1) => c0.merge(c1),
                            Left(c0) => c0,
                            Right(c1) => c1,
                        })
                        .collect::<Vec<_>>(),
                ))
            }
            (Simple(s0), Composite((None, c1))) | (Composite((None, c1)), Simple(s0)) => {
                Composite((Some(s0), c1))
            }
            (Simple(s0), Composite((Some(s1), c1))) | (Composite((Some(s1), c1)), Simple(s0)) => {
                Composite((Some(s0.merge(s1)), c1))
            }
        }
    }

    // return total with, including the column separators
    fn width(&self, column_separator_width: usize) -> usize {
        use ColSpec::*;

        match self {
            Empty => 0,
            Simple(s) => s.width(),
            Composite((s, c)) => {
                let separator_widths = if c.is_empty() {
                    0
                } else {
                    column_separator_width * (c.len() - 1)
                };
                max(
                    s.as_ref().map_or(0, |s| s.width()),
                    c.iter()
                        .map(|c| c.width(column_separator_width))
                        .sum::<usize>()
                        + separator_widths,
                )
            }
        }
    }

    fn anchor(&self, column_separator_width: usize) -> (usize, usize) {
        use ColSpec::*;

        match self {
            Empty => (0, 0),
            Simple(s) => s.anchor(),
            Composite((s, _)) => {
                let width = self.width(column_separator_width);
                s.as_ref()
                    .map(|s| s.anchor())
                    .unwrap_or_else(|| degenerate_anchor(Some(width)))
            }
        }
    }
}

impl<'a> From<&Cell<'a>> for ColSpec {
    fn from(value: &Cell<'a>) -> Self {
        use Cell::*;
        use ColSpec::*;

        match value {
            Aligned(s, _) => Simple(SimpleColSpec::from_width(s.width())),
            Anchored(s, idx) => {
                let w = s.width();
                Simple(SimpleColSpec::from_anchor(*idx, w - idx))
            }
            Row(cells) => {
                let cs = cells.iter().map(Into::<ColSpec>::into).collect::<Vec<_>>();
                Composite((None, cs))
            }
            Column(cells) => cells
                .iter()
                .fold(ColSpec::default(), |spec, cell| spec.merge(cell.into())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;
    use Align::*;
    use Cell::*;

    #[test_case(("Letsa go Mario!", Left).into(), r#"Letsa go Mario!"#)]
    #[test_case(Row(vec![("Letsa go Mario!", Left).into(), ("OK", Left).into()]), r#"Letsa go Mario! OK"#)]
    fn default_style(cell: Cell, expected: &str) {
        let result = cell.to_string();
        assert_eq!(&result, expected);
    }

    #[test_case(vec![
        vec!["A123", "B", "C1" ],
        vec!["D", "E1", "F999" ],
    ], r#"A123 B  C1  
D    E1 F999"#)]
    fn left_justified_strings(rows: Vec<Vec<&str>>, expected: &str) {
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
    ], r#"A123  B   C1
   D E1 F999"#)]
    fn right_justified_strings(rows: Vec<Vec<&str>>, expected: &str) {
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
        "            ",
        "A123  B   C1",
        "            ",
        "   D E1 F999",
        "   G  H     ",
        "            ",
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
    ], r#"A123 B   C1 
 D   E1 F999"#)]
    fn centred_strings(rows: Vec<Vec<&str>>, expected: &str) {
        let cell = Column(
            rows.iter()
                .map(|row| Row(row.iter().map(|s| (*s, Centre).into()).collect::<Vec<_>>()))
                .collect::<Vec<_>>(),
        );
        let result = cell.to_string();
        assert_eq!(result, expected);
    }

    #[test_case(Row(vec![("A", Left).into(), ("B", Left).into()]), r#"A|B"#)]
    fn styled(cell: Cell, expected: &str) {
        let pipe = Style::with_column_separator(Cow::Borrowed("|"));
        let result = cell.styled(&pipe).to_string();
        assert_eq!(&result, expected);
    }

    #[test_case(Column(vec![
        Row(vec![("A", Left).into(), Cell::anchored("1.25", 1), ("A99", Right).into()]),
        Row(vec![("B1", Left).into(), Cell::anchored("12.2", 2), ("B", Right).into()]),
    ]), vec![
        "A   1.25 A99",
        "B1 12.2    B",
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
        "A1 B1 C1  ",
        "A2     B C",
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
        "A1 B1 C1a    D1-abc",
        "A2    12.50  D     ",
        "A3    17.305     D3",
        "   A4     C4     D4",
        ]
)]
    fn merge_with_anchor(cell: Cell, expected_lines: Vec<&str>) {
        let result = cell.to_string();
        let expected = expected_lines.join_with("\n").to_string();
        assert_eq!(&result, &expected);
    }
}

mod conversions;
#[cfg(feature = "rust_decimal")]
mod rust_decimal;
