use std::{borrow::Cow, fmt::Display};

use super::{Align, Cell};

#[derive(Clone, Debug)]
pub struct PsvfError {
    line: usize,
    kind: PsvfErrorKind,
}

#[derive(Clone, Debug)]
pub enum PsvfErrorKind {
    MissingLeadingPipe,
    MissingTrailingPipe,
}

impl Display for PsvfError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Bad PSVF at line {}: {}", self.line, self.kind)
    }
}

impl Display for PsvfErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use PsvfErrorKind::*;
        match self {
            MissingLeadingPipe => f.write_str("missing leading pipe"),
            MissingTrailingPipe => f.write_str("missing trailing pipe"),
        }
    }
}

impl<'a, 'g> Cell<'a, 'g> {
    /// Split into lines, trim whitespac, and split each line on pipe.
    ///
    /// Cells which look like integers or decimals are anchored, otherwise aligned.
    pub fn from_psv(psv_string: &'a str, align: Align, gutter: &'g str) -> Cell<'a, 'g> {
        use Cell::*;

        Stack(
            psv_string
                .lines()
                .map(|line| {
                    line.trim()
                        .split(PIPE)
                        .map(|s| str_to_cell(s, align))
                        .collect::<Vec<_>>()
                })
                .map(|cells| Row(cells, gutter))
                .collect::<Vec<_>>(),
        )
    }
}

impl<'a, 'g> Cell<'a, 'g> {
    /// Split into lines, and split each line on pipe.
    /// Each line is required to begin and end with a pipe, after trimming whitespace.
    ///
    /// Cells which look like integers or decimals are anchored, otherwise aligned.
    pub fn from_psvf(
        psv_string: &'a str,
        align: Align,
        gutter: &'g str,
    ) -> std::result::Result<Cell<'a, 'g>, PsvfError> {
        use Cell::*;

        let lines = psv_string
            .lines()
            .enumerate()
            .map(|(i, line)| {
                let line = line.trim();
                if !line.starts_with(PIPE) {
                    Err(PsvfError {
                        line: i + 1,
                        kind: PsvfErrorKind::MissingLeadingPipe,
                    })
                } else if !line.ends_with(PIPE) {
                    Err(PsvfError {
                        line: i + 1,
                        kind: PsvfErrorKind::MissingTrailingPipe,
                    })
                } else {
                    Ok(line.strip_prefix(PIPE).unwrap().strip_suffix(PIPE).unwrap())
                }
            })
            .collect::<Result<Vec<_>, PsvfError>>()?;

        Ok(Stack(
            lines
                .into_iter()
                .map(|line| {
                    line.split(PIPE)
                        .map(|s| str_to_cell(s, align))
                        .collect::<Vec<_>>()
                })
                .map(|cells| Row(cells, gutter))
                .collect::<Vec<_>>(),
        ))
    }
}

fn str_to_cell<'a>(s: &'a str, align: Align) -> Cell<'a, 'static> {
    let s = s.trim();
    if s.is_empty() {
        Cell::Empty
    } else if s.chars().all(|c| c.is_ascii_digit()) {
        Cell::Anchored(Cow::Borrowed(s), s.len())
    } else {
        let maybe_decimal = s.chars().all(|c| c.is_ascii_digit() || c == '.');
        let n_dps = s.chars().filter(|c| *c == '.').count();
        if maybe_decimal && n_dps == 1 {
            let i_dp = s.find('.').unwrap();
            Cell::Anchored(Cow::Borrowed(s), i_dp)
        } else {
            Cell::Aligned(Cow::Borrowed(s), align)
        }
    }
}

const PIPE: &str = "|";
