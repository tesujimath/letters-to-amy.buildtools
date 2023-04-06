// TODO remove suppression for dead code warning
#![allow(dead_code)] //, unused_variables)]

use super::hugo::Metadata;
use super::util::slice_cmp;
use books::{book, is_single_chapter_book};
pub use extraction::references;
use itertools::Itertools;
use std::{
    cmp::{self, Ordering},
    collections::HashMap,
    fmt::{self, Display, Formatter},
    iter::{once, Peekable},
    num::ParseIntError,
    ops::{Deref, DerefMut},
    str::FromStr,
};
pub use tabulation::{BookReferences, BookReferences1, PostReferences, PostReferences1, Writer};

/// integer used for chapter index
type CInt = u8;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
struct Chapter(CInt);

impl Display for Chapter {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{}", self.0)
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
struct ChapterContext<'a> {
    book: &'a str,
    chapter: Option<Chapter>,
}

/// integer used for verse index
type VInt = u8;

/// Span used for verses
#[derive(PartialEq, Eq, Debug)]
pub enum VSpan {
    Point(VInt),
    Line(VInt, VInt),
}

impl VSpan {
    fn lower(&self) -> VInt {
        use VSpan::*;
        match self {
            Point(x) => *x,
            Line(x1, _) => *x1,
        }
    }

    fn upper(&self) -> VInt {
        use VSpan::*;
        match self {
            Point(x) => *x,
            Line(_, x2) => *x2,
        }
    }
}

impl PartialOrd for VSpan {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for VSpan {
    fn cmp(&self, other: &Self) -> Ordering {
        use Ordering::*;

        let lower_cmp = self.lower().cmp(&other.lower());
        if lower_cmp == Equal {
            self.upper().cmp(&other.upper())
        } else {
            lower_cmp
        }
    }
}

impl fmt::Display for VSpan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        use VSpan::*;
        match self {
            Point(x) => write!(f, "{}", x),
            Line(x1, x2) => write!(f, "{}-{}", x1, x2),
        }
    }
}

/// Spans used for verses
#[derive(PartialEq, Eq, Debug)]
pub struct VSpans(Vec<VSpan>);

impl VSpans {
    fn new() -> Self {
        Self(Vec::new())
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl PartialOrd for VSpans {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for VSpans {
    fn cmp(&self, other: &Self) -> Ordering {
        slice_cmp(&self.0, &other.0)
    }
}

impl<'a> IntoIterator for &'a VSpans {
    type Item = &'a VSpan;
    type IntoIter = std::slice::Iter<'a, VSpan>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl fmt::Display for VSpans {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(
            f,
            "{}",
            Itertools::intersperse(self.0.iter().map(|s| s.to_string()), ",".to_string())
                .collect::<String>()
        )
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct ChapterVerses {
    chapter: Option<Chapter>, // missing only in the case of single chapter books like Jude
    verses: VSpans,
}

impl ChapterVerses {
    fn new(chapter: Option<Chapter>, verses: VSpans) -> Self {
        Self { chapter, verses }
    }
}

impl PartialOrd for ChapterVerses {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ChapterVerses {
    fn cmp(&self, other: &Self) -> Ordering {
        use Ordering::*;

        let chapter_cmp = self.chapter.cmp(&other.chapter);
        if chapter_cmp != Equal {
            chapter_cmp
        } else {
            self.verses.cmp(&other.verses)
        }
    }
}

impl Display for ChapterVerses {
    // we really do want to write out a warning to stderr here so:
    #[allow(clippy::print_in_format_impl)]
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        match self.chapter {
            Some(chapter) => {
                if self.verses.is_empty() {
                    write!(f, "{}", chapter)
                } else {
                    write!(f, "{}:{}", chapter, self.verses)
                }
            }
            None => {
                if self.verses.is_empty() {
                    eprintln!("WARNING: no chapter or verses for ChapterVerses::fmt");
                    Ok(())
                } else {
                    write!(f, "v{}", self.verses)
                }
            }
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
// a non-empty list of chapter/verses references, with chapters strictly increasing
pub struct ChaptersVerses(Vec<ChapterVerses>);

impl ChaptersVerses {
    fn new(item: ChapterVerses) -> Self {
        Self(vec![item])
    }

    /// whether `c1 <= self` with respect to chapters, for all
    fn chapter_leq(&self, c1: Chapter) -> bool {
        self.0.iter().all(|cv| match cv.chapter {
            None => false,
            Some(c) => c1 <= c,
        })
    }

    fn leq_chapters_it<I1, I2>(mut it0: Peekable<I1>, mut it1: Peekable<I2>) -> bool
    where
        I1: Iterator<Item = Chapter>,
        I2: Iterator<Item = Chapter>,
    {
        loop {
            match (it0.peek(), it1.peek()) {
                (Some(c0), Some(c1)) if c0 < c1 => {
                    // skip and keep comparing
                    it0.next();
                }
                (Some(c0), Some(c1)) if c0 == c1 => {
                    // skip both and keep comparing
                    it0.next();
                    it1.next();
                }
                (Some(_), Some(_)) => {
                    return false;
                }
                (Some(_), None) => return false,
                (None, _) => return true,
            }
        }
    }

    /// whether `self <= other` with respect to chapters
    fn leq_chapters(&self, other: &ChaptersVerses) -> bool {
        Self::leq_chapters_it(
            self.0.iter().filter_map(|cv| cv.chapter).peekable(),
            other.0.iter().filter_map(|cv| cv.chapter).peekable(),
        )
    }

    /// whether `self + extra <= other` with respect to chapters
    fn leq_chapters_with(&self, extra: Chapter, other: &ChaptersVerses) -> bool {
        Self::leq_chapters_it(
            self.0
                .iter()
                .filter_map(|cv| cv.chapter)
                .chain(once(extra))
                .peekable(),
            other.0.iter().filter_map(|cv| cv.chapter).peekable(),
        )
    }
}

impl Deref for ChaptersVerses {
    type Target = Vec<ChapterVerses>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ChaptersVerses {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl PartialOrd for ChaptersVerses {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ChaptersVerses {
    fn cmp(&self, other: &Self) -> Ordering {
        slice_cmp(&self.0, &other.0)
    }
}

impl<'a> IntoIterator for &'a ChaptersVerses {
    type Item = &'a ChapterVerses;
    type IntoIter = std::slice::Iter<'a, ChapterVerses>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl IntoIterator for ChaptersVerses {
    type Item = ChapterVerses;
    type IntoIter = std::vec::IntoIter<ChapterVerses>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl Display for ChaptersVerses {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        let separator = if f.alternate() { " <br/> " } else { "; " };
        write!(
            f,
            "{}",
            Itertools::intersperse(
                self.0.iter().map(|cv| cv.to_string()),
                separator.to_string()
            )
            .collect::<String>()
        )
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct References(HashMap<&'static str, ChaptersVerses>);

/// consuming iterator
impl IntoIterator for References {
    type Item = (&'static str, ChaptersVerses);
    type IntoIter = std::collections::hash_map::IntoIter<&'static str, ChaptersVerses>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

pub struct AllReferences {
    metadata: Vec<Metadata>,
    separated_refs_by_book: HashMap<&'static str, BookReferences1>,
    refs_by_book: HashMap<&'static str, BookReferences>,
}

impl AllReferences {
    // extract bible references for a post and return any warnings
    pub fn extract_from_post(&mut self, post_metadata: Metadata, post_body: &str) -> Vec<String> {
        let (refs, warnings) = references(post_body);

        let m = self.insert(post_metadata, refs);

        let annotated_warnings = warnings.into_iter().map(|w| format!("{}: {}", &m.url, w));

        annotated_warnings.collect()
    }
}

mod books;
mod extraction;
mod tabulation;
mod tests;
