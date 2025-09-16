use joinery::JoinableIterator;
use lazy_format::make_lazy_format;
use std::{borrow::Cow, fmt::Display};

pub enum Cell<'a> {
    Left(Cow<'a, str>),
    Right(Cow<'a, str>),
    Centre(Cow<'a, str>),
    Anchor(Cow<'a, str>, usize),
    Row(Vec<Cell<'a>>),
    Column(Vec<Cell<'a>>),
}

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

pub struct Table<'a, S> {
    content: Cell<'a>,
    style: Style<S>,
}

impl<'a> Cell<'a> {
    pub fn styled<S>(&self, style: &Style<S>) -> impl Display
    where
        S: Display,
    {
        use Cell::*;

        make_lazy_format!(|f| match self {
            Left(s) => f.write_str(s.as_ref()),
            Right(s) => f.write_str(s.as_ref()),
            Centre(s) => f.write_str(s.as_ref()),
            Anchor(_s, _a) => todo!(),
            Row(cells) => write!(f, "{}", cells.iter().join_with(&style.column_separator)),
            Column(cells) => write!(f, "{}", cells.iter().join_with("\n")),
        })
    }
}

impl<'a> Display for Cell<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.styled(&Style::default()))
    }
}

pub fn add(left: u64, right: u64) -> u64 {
    left + right
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
