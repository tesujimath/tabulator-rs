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
//!    let cell = Stack(vec![
//!        Row(vec![("A", Left).into(), 1.25.into(), ("A99", Right).into()], "  "),
//!        Row(vec![("B1", Left).into(), 12.5.into(), ("B", Right).into()], "  "),
//!    ]);
//!
//!    let output = cell.to_string();
//!}

use itertools::Itertools;
#[allow(unused_imports)] // unsure why there's otherwise a warning here
use joinery::Joinable;
use lazy_format::make_lazy_format;
use std::{borrow::Cow, cmp::max, collections::VecDeque, fmt::Display};
use unicode_width::UnicodeWidthStr;

#[derive(Copy, Clone, Debug)]
pub enum Align {
    Left,
    Right,
    Centre,
}

#[derive(Default, Debug)]
pub enum Cell<'a> {
    #[default]
    Empty,
    Aligned(Cow<'a, str>, Align),
    Anchored(Cow<'a, str>, usize), // index of character which is anchored, e.g. the decimal point
    Row(Vec<Cell<'a>>, &'static str), // horizontal sequence with gutter
    Stack(Vec<Cell<'a>>),          // vertical stack
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
    /// Return the string anchored at position `idx`.
    pub fn anchored<S>(s: S, idx: usize) -> Self
    where
        S: Into<Cow<'a, str>>,
    {
        Cell::Anchored(s.into(), idx)
    }
}

#[derive(Clone, Debug)]
struct Graticule {
    width: usize,
    anchor: Option<(usize, usize)>,                   // idx, trailing
    children: Option<(Vec<Graticule>, &'static str)>, // horizontal sequence with gutter
}

#[derive(Clone, Debug)]
enum Remaining<'a> {
    Empty,
    Aligned(&'a str, Align),
    Anchored(&'a str, usize), // index of character which is anchored, e.g. the decimal point
    Row(Vec<Remaining<'a>>),  // horizontal sequence only, gutter removed to Graticule
    Stack(VecDeque<Remaining<'a>>), // vertical stack
}

impl From<&Cell<'_>> for Graticule {
    fn from(value: &Cell<'_>) -> Self {
        use Cell::*;

        match value {
            Empty => Graticule {
                width: 0,
                anchor: None,
                children: None,
            },

            Aligned(s, _) => Graticule {
                width: s.width(),
                anchor: None,
                children: None,
            },

            Anchored(s, idx) => {
                let width = s.width();
                Graticule {
                    width,
                    anchor: Some((*idx, width - *idx)),
                    children: None,
                }
            }

            Row(cells, gutter) => {
                let children = cells
                    .iter()
                    .map(Into::<Graticule>::into)
                    .collect::<Vec<_>>();
                let width = children.iter().map(|child| child.width).sum::<usize>()
                    + total_gutter(gutter, children.len());
                Graticule {
                    width,
                    anchor: None,
                    children: Some((children, gutter)),
                }
            }

            Stack(cells) => cells
                .iter()
                .map(Into::<Graticule>::into)
                .fold(Graticule::empty(), Graticule::merge),
        }
    }
}

impl<'a, 'c> From<&'c Cell<'a>> for Remaining<'a>
where
    'c: 'a,
{
    fn from(value: &'c Cell<'a>) -> Self {
        use Cell::*;

        match value {
            Empty => Remaining::Empty,

            Aligned(s, align) => Remaining::Aligned(s.as_ref(), *align),

            Anchored(s, idx) => Remaining::Anchored(s.as_ref(), *idx),

            Row(cells, _gutter) => {
                let children = cells
                    .iter()
                    .map(Into::<Remaining>::into)
                    .collect::<Vec<_>>();

                Remaining::Row(children)
            }

            Stack(cells) => {
                let children = cells
                    .iter()
                    .map(Into::<Remaining>::into)
                    .collect::<VecDeque<_>>();

                Remaining::Stack(children)
            }
        }
    }
}

impl<'a> Display for Cell<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let graticule: Graticule = self.into();
        let mut remaining: Remaining = self.into();

        // TODO remove
        println!(
            "Cell {:?} into Graticule {:?}, Remaining {:?}",
            self, &graticule, &remaining
        );

        let mut first_line = true;
        while !remaining.is_empty() {
            if !first_line {
                f.write_str("\n")?;
            }
            first_line = false;
            remaining = remaining.format(f, &graticule)?;
        }

        Ok(())
    }
}

impl<'a> Remaining<'a> {
    fn format(
        self,
        f: &mut std::fmt::Formatter<'_>,
        g: &Graticule,
    ) -> Result<Remaining<'a>, std::fmt::Error> {
        use Remaining::*;

        Ok(match self {
            Empty => {
                write!(f, "{}", pad(g.width))?;
                Empty
            }

            Aligned(s, align) => {
                use Align::*;

                let total_pad = g.width - s.width();
                let (pad_l, pad_r) = match align {
                    Left => (0, total_pad),
                    Right => (total_pad, 0),
                    Centre => {
                        let pad_l = total_pad / 2;
                        (pad_l, total_pad - pad_l)
                    }
                };
                write!(f, "{}{}{}", pad(pad_l), s, pad(pad_r))?;
                Empty
            }

            Anchored(s, idx) => {
                let (layout_idx, layout_trailing) = g.anchor.unwrap(); // TODO don't think this can fail
                let trailing = s.width() - idx;
                let pad_l = layout_idx - idx;
                let pad_r = layout_trailing - trailing;
                write!(f, "{}{}{}", pad(pad_l), s, pad(pad_r))?;
                Empty
            }

            Row(children) => {
                if children.is_empty() {
                    write!(f, "{}", pad(g.width))?;
                    Empty
                } else {
                    use itertools::EitherOrBoth::*;

                    let (g_children, gutter) = g.children.as_ref().unwrap();
                    let remaining = children
                        .into_iter()
                        .zip_longest(g_children)
                        .enumerate()
                        .map(|(i, x)| match x {
                            Both(child, g_child) => (i, (child, g_child)),
                            Left(_child) => panic!("impossible"),
                            Right(g_child) => (i, (Empty, g_child)), // graticule is longer than row, so extend with empty
                        })
                        .map(|(i, (child, g_child))| {
                            if i > 0 {
                                write!(f, "{}", gutter)
                            } else {
                                Ok(())
                            }
                            .and_then(|_| child.format(f, g_child))
                        })
                        .collect::<Result<Vec<_>, std::fmt::Error>>()?;

                    if remaining.iter().all(|layout| layout.is_empty()) {
                        Empty
                    } else {
                        Row(remaining)
                    }
                }
            }

            Stack(mut children) => {
                if let Some(child) = children.pop_front() {
                    let remaining = child.format(f, g)?;
                    if !remaining.is_empty() {
                        children.push_front(remaining);
                    }
                    if children.iter().all(|layout| layout.is_empty()) {
                        Empty
                    } else {
                        Stack(children)
                    }
                } else {
                    write!(f, "{}", pad(g.width))?;
                    Empty
                }
            }
        })
    }

    fn is_empty(&self) -> bool {
        matches!(&self, Remaining::Empty)
    }
}

impl Graticule {
    fn empty() -> Graticule {
        Graticule {
            width: 0,
            anchor: None,
            children: None,
        }
    }

    fn merge(self: Graticule, other: Graticule) -> Graticule {
        let Graticule {
            width: w0,
            anchor: a0,
            children: children0,
        } = self;
        let Graticule {
            width: w1,
            anchor: a1,
            children: children1,
        } = other;

        let children = match (children0, children1) {
            (Some((g0s, gutter0)), Some((g1s, gutter1))) => {
                use itertools::EitherOrBoth::*;

                let children = g0s
                    .into_iter()
                    .zip_longest(g1s)
                    .map(|x| match x {
                        Both(g0, g1) => g0.merge(g1),
                        Left(g) => g,
                        Right(g) => g,
                    })
                    .collect::<Vec<_>>();

                Some((children, longer(gutter0, gutter1)))
            }
            (c, None) => c,
            (None, c) => c,
        };
        let width = if let Some((gs, gutter)) = children.as_ref() {
            gs.iter().map(|g| g.width).sum::<usize>() + total_gutter(gutter, gs.len())
        } else {
            max(w0, w1)
        };
        let anchor = match (a0, a1) {
            (Some((idx0, trailing0)), Some((idx1, trailing1))) => {
                Some((max(idx0, idx1), max(trailing0, trailing1)))
            }
            (a, None) => a,
            (None, a) => a,
        };

        Graticule {
            width,
            anchor,
            children,
        }
    }
}

fn total_gutter(column_gutter: &'static str, columns: usize) -> usize {
    if columns > 0 {
        (columns - 1) * column_gutter.width()
    } else {
        0
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

fn longer<'a>(s1: &'a str, s2: &'a str) -> &'a str {
    if s1.width() >= s2.width() {
        s1
    } else {
        s2
    }
}

mod conversions;
#[cfg(feature = "num-bigint")]
mod num_bigint;
#[cfg(feature = "rust_decimal")]
mod rust_decimal;
