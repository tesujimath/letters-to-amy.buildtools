// TODO remove suppression for dead code warning
#![allow(dead_code)] //, unused_variables)]

use super::hugo::Metadata;
use super::util::insert_in_order;
use super::util::slice_cmp;
use books::{book, is_single_chapter_book};
pub use extraction::references;
use itertools::Itertools;
use std::collections::hash_map;
use std::{
    cmp::{self, Ordering},
    collections::HashMap,
    fmt::{self, Display, Formatter},
    num::ParseIntError,
    str::FromStr,
};
pub use tabulation::Writer;

/// integer used for chapter index
type CInt = u8;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
struct Chapter(CInt);

impl Display for Chapter {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Copy, Debug)]
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
    chapter: Option<Chapter>,
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
pub struct ChaptersVerses(Vec<ChapterVerses>);

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

#[derive(PartialEq, Eq, Debug)]
pub struct PostReferences {
    pub post_index: usize,
    pub cvs: ChaptersVerses,
}

impl PostReferences {
    fn new(post_index: usize, cvs: ChaptersVerses) -> Self {
        Self { post_index, cvs }
    }
}

impl Display for PostReferences {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{:#}", &self.cvs)
    }
}

impl PartialOrd for PostReferences {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PostReferences {
    fn cmp(&self, other: &Self) -> Ordering {
        use Ordering::*;
        match self.cvs.cmp(&other.cvs) {
            Equal => self.post_index.cmp(&other.post_index),
            cmp => cmp,
        }
    }
}

pub struct AllReferences {
    pub metadata: Vec<Metadata>,
    pub refs_by_book: HashMap<&'static str, Vec<PostReferences>>,
}

impl AllReferences {
    pub fn new() -> Self {
        AllReferences {
            metadata: Vec::new(),
            refs_by_book: HashMap::new(),
        }
    }

    pub fn insert(&mut self, metadata: Metadata, refs: References) {
        self.metadata.push(metadata);
        let post_index = self.metadata.len() - 1;

        for (book, cvs) in refs.into_iter() {
            let post_refs = PostReferences::new(post_index, cvs);

            use hash_map::Entry::*;
            match self.refs_by_book.entry(book) {
                Occupied(mut o) => {
                    insert_in_order(o.get_mut(), post_refs);
                }
                Vacant(v) => {
                    v.insert(vec![post_refs]);
                }
            }
        }
    }
}

mod books;
mod extraction;
mod tabulation;
mod tests;
