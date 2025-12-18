use std::borrow::Cow;

use serde::Deserialize;

use super::{Align, Cell};

#[derive(Default, Debug)]
#[cfg_attr(feature = "json", derive(Deserialize))]
#[cfg_attr(feature = "json", serde(rename_all = "lowercase"))]
pub enum BorrowedCell<'a> {
    #[default]
    Empty,
    Aligned(&'a str, Align),
    Anchored(&'a str, usize), // index of character which is anchored, e.g. the decimal point
    Row(Vec<BorrowedCell<'a>>, &'a str), // horizontal sequence with gutter
    Stack(Vec<BorrowedCell<'a>>), // vertical stack
}

impl<'a> From<BorrowedCell<'a>> for Cell<'a, 'a> {
    fn from(value: BorrowedCell<'a>) -> Self {
        match value {
            BorrowedCell::Empty => Cell::Empty,
            BorrowedCell::Aligned(s, align) => Cell::Aligned(Cow::Borrowed(s), align),
            BorrowedCell::Anchored(s, idx) => Cell::Anchored(Cow::Borrowed(s), idx),
            BorrowedCell::Row(cells, gutter) => Cell::Row(
                cells
                    .into_iter()
                    .map(Into::<Cell>::into)
                    .collect::<Vec<_>>(),
                gutter,
            ),
            BorrowedCell::Stack(cells) => Cell::Stack(
                cells
                    .into_iter()
                    .map(Into::<Cell>::into)
                    .collect::<Vec<_>>(),
            ),
        }
    }
}

impl<'a> Cell<'a, 'a> {
    pub fn from_json(json_string: &'a str) -> serde_json::Result<Cell<'a, 'a>> {
        let cell: BorrowedCell = serde_json::from_str(json_string)?;

        Ok(cell.into())
    }
}
