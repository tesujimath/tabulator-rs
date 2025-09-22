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
    borrow::Cow,
    cmp::{max, min},
    fmt::Display,
};
use strum::IntoEnumIterator;
use strum_macros::{EnumCount, EnumIter};
use unicode_width::UnicodeWidthStr;

#[derive(Debug)]
pub enum Align {
    Left,
    Right,
    Centre,
}

#[derive(Copy, Clone, EnumCount, EnumIter, Default, Debug)]
pub enum Space {
    Flush,
    Minor,
    #[default]
    Medium,
    Major,
}

#[derive(Clone, Default, Debug)]
/// Spacing is flexible.  If children are not specified, they default to primary.
/// And the last child spacing is reused for any children beyond what was specified.
pub struct Spacing {
    primary: Space,
    children: Option<Vec<Option<Spacing>>>,
}

impl From<Space> for Spacing {
    fn from(value: Space) -> Self {
        Spacing {
            primary: value,
            children: None,
        }
    }
}

impl Spacing {
    pub fn with_children(primary: Space, children: Vec<Option<Spacing>>) -> Spacing {
        Spacing {
            primary,
            children: Some(children),
        }
    }

    // return best-fit spacing for i_child, which is the latest one which has a value, or self
    fn get_child_spacing(&self, i_child: usize) -> &Spacing {
        match &self.children {
            None => self,
            Some(children) => {
                if children.is_empty() {
                    self
                } else {
                    // loop back until we find something
                    for i in (0..min(i_child + 1, children.len())).rev() {
                        if let Some(spacing) = &children[i] {
                            return spacing;
                        }
                    }

                    // fallback value if nothing found
                    self
                }
            }
        }
    }
}

#[macro_export]
macro_rules! spacing {
    ([$val:expr; $($child:tt),* $(,)?]) => {
        Spacing::with_children( $val, vec![ $( $crate::optional_spacing!($child) ),* ] )
    };

    ($val:expr) => {
        Into::<Spacing>::into($val)
    };
}

// helper, not public
#[doc(hidden)]
#[macro_export]
macro_rules! optional_spacing {
    (_) => {
        None
    };

    ([$val:expr; $($child:tt),* $(,)?]) => {
        Some(Spacing::with_children( $val, vec![ $( $crate::optional_spacing!($child) ),* ] ))
    };

    ($val:expr) => {
        Some(Into::<Spacing>::into($val))
    };
}

#[derive(Copy, Clone, Default, Debug)]
pub enum Style {
    #[default]
    Spaced,
    Piped,
}

/// Indexed by Spacing as usize
struct Styled(Vec<&'static str>);

impl From<Style> for Styled {
    fn from(value: Style) -> Self {
        use Space::*;
        use Style::*;

        Self(match value {
            Spaced => Space::iter()
                .map(|spacing| match spacing {
                    Flush => "",
                    Minor => " ",
                    Medium => "  ",
                    Major => "   ",
                })
                .collect::<Vec<_>>(),
            Piped => Space::iter()
                .map(|spacing| match spacing {
                    Flush => "",
                    Minor => "|",
                    Medium => "||",
                    Major => "|||",
                })
                .collect::<Vec<_>>(),
        })
    }
}

impl Styled {
    fn space(&self, spacing: Space) -> &'static str {
        self.0[spacing as usize]
    }

    fn width(&self, spacing: Space) -> usize {
        self.space(spacing).len()
    }
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
    pub fn empty() -> Self {
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

impl<'a> Cell<'a> {
    pub fn layout(&self, spacing: &Spacing, style: Style) -> impl Display {
        let spec: ColSpec = self.into();

        let styled = style.into();
        make_lazy_format!(|f| self.format(f, spacing, &styled, &spec))
    }

    fn format(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        spacing: &Spacing,
        styled: &Styled,
        spec: &ColSpec,
    ) -> std::fmt::Result {
        use Cell::*;
        use ColSpec::*;

        let spec_width = spec.width(spacing, styled);
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
                let (spec_idx, spec_trailing) = spec.anchor(spacing, styled);
                let trailing = s.width() - idx;
                let pad_l = spec_idx - idx;
                let pad_r = spec_trailing - trailing;
                write!(f, "{}{}{}", pad(pad_l), s, pad(pad_r))
            }
            (Row(cells), Composite((_spec_s, spec_c))) => {
                use itertools::EitherOrBoth::*;

                let mut sep = false;
                let empty_cell = Cell::empty();

                // println!("laying out {:?} using {:?}", cells, spacing);

                for (i_child, (cell, spec)) in cells
                    .iter()
                    .zip_longest(spec_c)
                    .map(|x| match x {
                        Both(cell, spec) => (cell, spec),
                        Left(_cell) => todo!("cell without spec"),
                        Right(spec) => (&empty_cell, spec),
                    })
                    .enumerate()
                {
                    if sep {
                        write!(f, "{}", styled.space(spacing.primary))?
                    }
                    sep = true;

                    let child_spacing = spacing.get_child_spacing(i_child);
                    // println!(
                    //     "laying out child {:?} {:?} using {:?}",
                    //     i_child, cell, child_spacing
                    // );
                    cell.format(f, child_spacing, styled, spec)?;
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
                    cell.format(f, spacing, styled, spec)?;
                }
                Ok(())
            }
            _ => todo!("mismatched cell {:?} and spec {:?}", self, &spec),
        }
    }
}

impl<'a> Display for Cell<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let spacing = Spacing::default();
        write!(f, "{}", self.layout(&spacing, Style::default()))
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
struct SingleColSpec {
    width: Option<usize>,
    anchor: Option<(usize, usize)>,
}

fn degenerate_anchor(width: Option<usize>) -> (usize, usize) {
    (width.unwrap_or(0), 0)
}

impl SingleColSpec {
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

    fn merge(self, other: SingleColSpec) -> Self {
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
    Simple(SingleColSpec),
    Composite((Option<SingleColSpec>, Vec<ColSpec>)),
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
    fn width(&self, spacing: &Spacing, styled: &Styled) -> usize {
        use ColSpec::*;

        match self {
            Empty => 0,
            Simple(s) => s.width(),
            Composite((s, c)) => {
                let column_separator_width = styled.width(spacing.primary);
                let separator_widths = if c.is_empty() {
                    0
                } else {
                    column_separator_width * (c.len() - 1)
                };
                max(
                    s.as_ref().map_or(0, |s| s.width()),
                    c.iter()
                        .enumerate()
                        .map(|(i_child, c)| c.width(spacing.get_child_spacing(i_child), styled))
                        .sum::<usize>()
                        + separator_widths,
                )
            }
        }
    }

    fn anchor(&self, spacing: &Spacing, styled: &Styled) -> (usize, usize) {
        use ColSpec::*;

        match self {
            Empty => (0, 0),
            Simple(s) => s.anchor(),
            Composite((s, _)) => {
                let width = self.width(spacing, styled);
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
            Aligned(s, _) => Simple(SingleColSpec::from_width(s.width())),
            Anchored(s, idx) => {
                let w = s.width();
                Simple(SingleColSpec::from_anchor(*idx, w - idx))
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

mod conversions;
#[cfg(feature = "num-bigint")]
mod num_bigint;
#[cfg(feature = "rust_decimal")]
mod rust_decimal;
