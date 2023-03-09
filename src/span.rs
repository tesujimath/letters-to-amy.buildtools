// TODO remove suppression for dead code warning
#![allow(dead_code)] //, unused_variables)]

use super::util::slice_cmp;
use num::One;
use std::cmp::{self, Ordering};
use std::fmt;
use std::ops::Add;

/// An inclusive range, allowed to be a single element, but not allowed to be empty.
#[derive(Eq, PartialEq, Debug)]
pub enum Span<T> {
    Point(T),
    Line(T, T),
}

impl<T> Span<T>
where
    T: PartialOrd + Add<Output = T> + One + Copy,
{
    pub fn at(x: T) -> Span<T> {
        Span::Point(x)
    }

    pub fn between(from: T, to: T) -> Span<T> {
        assert!(from <= to);

        Span::Line(from, to)
    }

    fn lower(&self) -> T {
        use Span::*;
        match self {
            Point(x) => *x,
            Line(x1, _) => *x1,
        }
    }

    fn upper(&self) -> T {
        use Span::*;
        match self {
            Point(x) => *x,
            Line(_, x2) => *x2,
        }
    }

    /// merge in other, which must be touching
    fn merge(&mut self, other: Span<T>)
    where
        T: Ord,
    {
        assert!(self.touches(&other));

        use Span::*;
        match (&self, &other) {
            (Point(x), Point(y)) if x == y => (),
            _ => {
                *self = Line(
                    cmp::min(self.lower(), other.lower()),
                    cmp::max(self.upper(), other.upper()),
                )
            }
        }
    }

    /// whether other touches this, where a distance of 1 counts as touching
    fn touches(&self, other: &Span<T>) -> bool {
        !(self.upper() + T::one() < other.lower() || self.lower() > other.upper() + T::one())
    }
}

impl<T> PartialOrd for Span<T>
where
    T: Add<Output = T> + One + Copy + Ord,
{
    fn partial_cmp(&self, other: &Span<T>) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for Span<T>
where
    T: Add<Output = T> + One + Copy + Ord,
{
    fn cmp(&self, other: &Span<T>) -> Ordering {
        use Ordering::*;

        let lower_cmp = self.lower().cmp(&other.lower());
        if lower_cmp == Equal {
            self.upper().cmp(&other.upper())
        } else {
            lower_cmp
        }
    }
}

impl<T> fmt::Display for Span<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        use Span::*;
        match self {
            Point(x) => write!(f, "{}", x),
            Line(x1, x2) => write!(f, "{}-{}", x1, x2),
        }
    }
}

/// An ordered Vec of Spans, minimally coalesced
#[derive(Eq, PartialEq, Debug)]
pub struct Spans<T>(Vec<Span<T>>);

impl<T> Spans<T> {
    pub fn new() -> Spans<T> {
        Spans(Vec::new())
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// determine leftmost item from i-1 and i
    fn get_leftmost_touching(&self, i: usize, item: &Span<T>) -> Option<usize>
    where
        T: Ord + Add<Output = T> + One + Copy,
    {
        let touching_left = i > 0 && self.0[i - 1].touches(item);
        let touching_this = i < self.0.len() && self.0[i].touches(item);

        match (touching_left, touching_this) {
            (false, false) => None,
            (true, _) => Some(i - 1),
            (false, true) => Some(i),
        }
    }

    pub fn insert(&mut self, item: Span<T>)
    where
        T: Ord + Add<Output = T> + One + Copy + fmt::Display,
    {
        match self.0.binary_search(&item) {
            Ok(i) => {
                // repeated insert, ignore
                assert!(item == self.0[i]);
            }
            Err(i) => {
                match self.get_leftmost_touching(i, &item) {
                    None => self.0.insert(i, item),
                    Some(j) => {
                        self.0[j].merge(item);

                        // coalesce right until no more
                        while self.0.len() > j + 1 && self.0[j].touches(&self.0[j + 1]) {
                            let next = self.0.remove(j + 1);
                            self.0[j].merge(next);
                        }
                    }
                }
            }
        }
    }

    pub fn merge(&mut self, other: Spans<T>)
    where
        T: Ord + Add<Output = T> + One + Copy + fmt::Display,
    {
        for item in other.0 {
            self.insert(item);
        }
    }
}

impl<T> PartialOrd for Spans<T>
where
    T: Add<Output = T> + One + Copy + Ord,
{
    fn partial_cmp(&self, other: &Spans<T>) -> Option<Ordering> {
        Some(self.cmp(&other))
    }
}

impl<T> Ord for Spans<T>
where
    T: Add<Output = T> + One + Copy + Ord,
{
    fn cmp(&self, other: &Spans<T>) -> Ordering {
        slice_cmp(&self.0, &other.0)
    }
}

impl<'a, T> IntoIterator for &'a Spans<T> {
    type Item = &'a Span<T>;
    type IntoIter = std::slice::Iter<'a, Span<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<T> FromIterator<Span<T>> for Spans<T>
where
    T: Ord + Add<Output = T> + One + Copy + fmt::Display,
{
    fn from_iter<I: IntoIterator<Item = Span<T>>>(iter: I) -> Self {
        let mut spans = Spans::new();

        for s in iter {
            spans.insert(s);
        }

        spans
    }
}

impl<T> fmt::Display for Spans<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        if !self.0.is_empty() {
            write!(f, "{}", self.0[0].to_string())?;
        }
        for i in 1..self.0.len() {
            write!(f, ", {}", self.0[i].to_string())?;
        }
        Ok(())
    }
}

mod tests;
