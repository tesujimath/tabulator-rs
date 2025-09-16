use itertools::Itertools;
use lazy_format::make_lazy_format;
use std::{
    cmp::max,
    {borrow::Cow, fmt::Display},
};
use unicode_width::UnicodeWidthStr;

#[derive(Debug)]
pub enum Cell<'a> {
    Left(Cow<'a, str>),
    Right(Cow<'a, str>),
    Centre(Cow<'a, str>),
    Anchor(Cow<'a, str>, usize),
    Row(Vec<Cell<'a>>),
    Column(Vec<Cell<'a>>),
}

#[derive(Debug)]
pub struct Style<S> {
    column_separator: S,
}

impl Default for Style<&str> {
    fn default() -> Self {
        Self {
            column_separator: " ",
        }
    }
}

impl<S> Style<S> {
    pub fn with_column_separator(sep: S) -> Self {
        Self {
            column_separator: sep,
        }
    }
}

impl<'a> Cell<'a> {
    pub fn styled<S>(&self, style: &Style<S>) -> impl Display
    where
        S: Display,
    {
        let spec: ColSpec = self.into();

        make_lazy_format!(|f| self.format(f, style, &spec))
    }

    fn format<S>(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        style: &Style<S>,
        spec: &ColSpec,
    ) -> std::fmt::Result
    where
        S: Display,
    {
        use Cell::*;
        use ColSpec::*;

        match (self, spec) {
            (Left(s), spec) => {
                let pad_w = spec.width() - s.width();
                write!(f, "{}{}", s, pad(pad_w))
            }
            (Right(s), spec) => {
                let pad_w = spec.width() - s.width();
                write!(f, "{}{}", pad(pad_w), s)
            }
            (Centre(s), spec) => {
                let pad_w = spec.width() - s.width();
                let pad_l = pad_w / 2;
                let pad_r = pad_w - pad_l;
                write!(f, "{}{}{}", pad(pad_l), s, pad(pad_r))
            }
            (Anchor(_s, _a), spec) => todo!("anchor not yet implemented"),
            (Row(cells), Composite(spec)) => {
                use itertools::EitherOrBoth::*;

                let mut sep = false;
                for (cell, spec) in cells.iter().zip_longest(spec).map(|x| match x {
                    Both(cell, spec) => (cell, spec),
                    Left(cell) => todo!("cell without spec"),
                    Right(spec) => todo!("spec without cell"),
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

impl SimpleColSpec {
    fn from_width(width: usize) -> Self {
        Self {
            width: Some(width),
            anchor: None,
        }
    }

    fn from_anchor(left: usize, right: usize) -> Self {
        Self {
            width: None,
            anchor: Some((left, right)),
        }
    }

    fn width(&self) -> usize {
        max(
            self.width.unwrap_or(0),
            self.anchor
                .map(|(left, right)| left + right + 1)
                .unwrap_or(0),
        )
    }

    fn anchor(&self) -> Option<(usize, usize)> {
        self.anchor
    }

    fn merge(self, other: SimpleColSpec) -> Self {
        let width = match (self.width, other.width) {
            (Some(w0), Some(w1)) => Some(max(w0, w1)),
            (w0, None) => w0,
            (None, w1) => w1,
        };

        let anchor = match (self.anchor, other.anchor) {
            (Some((left0, right0)), Some((left1, right1))) => {
                Some((max(left0, left1), max(right0, right1)))
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
    Composite(Vec<ColSpec>),
}

impl ColSpec {
    fn merge(self, other: ColSpec) -> Self {
        use itertools::EitherOrBoth::*;
        use ColSpec::*;

        match (self, other) {
            (Empty, s) => s,
            (s, Empty) => s,
            (Simple(s0), Simple(s1)) => Simple(s0.merge(s1)),
            (Composite(c0), Composite(c1)) => Composite(
                c0.into_iter()
                    .zip_longest(c1)
                    .map(|x| match x {
                        Both(c0, c1) => c0.merge(c1),
                        Left(c0) => c0,
                        Right(c1) => c1,
                    })
                    .collect::<Vec<_>>(),
            ),
            _ => todo!("merging simple and composite not yet implemented"),
        }
    }

    fn width(&self) -> usize {
        use ColSpec::*;

        match self {
            Empty => 0,
            Simple(s0) => s0.width(),
            Composite(c0) => c0.iter().map(|c| c.width()).sum(),
        }
    }
}

impl<'a> From<&Cell<'a>> for ColSpec {
    fn from(value: &Cell<'a>) -> Self {
        use Cell::*;
        use ColSpec::*;

        match value {
            Left(s) => Simple(SimpleColSpec::from_width(s.width())),
            Right(s) => Simple(SimpleColSpec::from_width(s.width())),
            Centre(s) => Simple(SimpleColSpec::from_width(s.width())),
            Anchor(s, anchor) => {
                let w = s.width();
                Simple(SimpleColSpec::from_anchor(*anchor, w - anchor))
            }
            Row(cells) => Composite(cells.iter().map(Into::<ColSpec>::into).collect::<Vec<_>>()),
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
    use Cell::*;
    use Cow::*;

    #[test_case(Left(Borrowed("Letsa go Mario!")), r#"Letsa go Mario!"#)]
    #[test_case(Row(vec![Left(Borrowed("Letsa go Mario!")), Left(Borrowed("OK"))]), r#"Letsa go Mario! OK"#)]
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
                .map(|row| Row(row.iter().map(|s| Left(Borrowed(*s))).collect::<Vec<_>>()))
                .collect::<Vec<_>>(),
        );
        let result = cell.to_string();
        assert_eq!(result, expected);
    }

    #[test_case(Row(vec![Left(Borrowed("A")), Left(Borrowed("B"))]), r#"A|B"#)]
    fn styled(cell: Cell, expected: &str) {
        let pipe = Style::with_column_separator("|");
        let result = cell.styled(&pipe).to_string();
        assert_eq!(&result, expected);
    }
}
