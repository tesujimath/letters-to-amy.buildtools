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
    ops::{Deref, DerefMut},
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
// a non-empty list of chapter/verses references
pub struct ChaptersVerses(Vec<ChapterVerses>);

impl ChaptersVerses {
    fn new(item: ChapterVerses) -> Self {
        Self(vec![item])
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

#[derive(PartialEq, Eq, Debug)]
// a post with just one chapters worth of references
pub struct PostReferences1 {
    pub post_index: usize,
    pub cv: ChapterVerses,
}

impl PostReferences1 {
    fn new(post_index: usize, cv: ChapterVerses) -> Self {
        Self { post_index, cv }
    }
}

impl PartialOrd for PostReferences1 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PostReferences1 {
    fn cmp(&self, other: &Self) -> Ordering {
        use Ordering::*;
        match self.cv.cmp(&other.cv) {
            Equal => self.post_index.cmp(&other.post_index),
            cmp => cmp,
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
// a post with all its chapters' references
pub struct PostReferences {
    pub post_index: usize,
    pub cvs: ChaptersVerses,
}

impl PostReferences {
    fn from1(refs1: PostReferences1) -> Self {
        Self {
            post_index: refs1.post_index,
            cvs: ChaptersVerses::new(refs1.cv),
        }
    }

    fn push(&mut self, refs1: PostReferences1) {
        self.cvs.push(refs1.cv);
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

// separated references to a single book
pub struct BookReferences1(Vec<PostReferences1>);

impl BookReferences1 {
    fn new(post_index: usize, cv: ChapterVerses) -> BookReferences1 {
        BookReferences1(vec![PostReferences1::new(post_index, cv)])
    }
}

impl Deref for BookReferences1 {
    type Target = Vec<PostReferences1>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for BookReferences1 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

// all the references to a single book
pub struct BookReferences(Vec<PostReferences>);

impl BookReferences {
    fn from_separated(refs1: BookReferences1) -> BookReferences {
        let mut refs: Vec<PostReferences> = Vec::new();
        for r1 in refs1.0.into_iter() {
            let mut unmerged = None;
            match refs.last_mut() {
                Some(r0) if r0.post_index == r1.post_index => {
                    r0.push(r1);
                }
                _ => unmerged = Some(r1),
            };

            if let Some(unmerged) = unmerged {
                refs.push(PostReferences::from1(unmerged))
            }
        }

        BookReferences(refs)
    }
}

impl Deref for BookReferences {
    type Target = Vec<PostReferences>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct AllReferences {
    pub metadata: Vec<Metadata>,
    pub separated_refs_by_book: HashMap<&'static str, BookReferences1>,
    pub refs_by_book: HashMap<&'static str, BookReferences>,
}

impl AllReferences {
    pub fn new() -> Self {
        AllReferences {
            metadata: Vec::new(),
            separated_refs_by_book: HashMap::new(),
            refs_by_book: HashMap::new(),
        }
    }

    // insert the post references separately and return a stable reference to its metadata
    fn insert(&mut self, metadata: Metadata, refs: References) -> &Metadata {
        self.metadata.push(metadata);
        let post_index = self.metadata.len() - 1;

        for (book, cvs) in refs.into_iter() {
            for cv in cvs.into_iter() {
                use hash_map::Entry::*;
                match self.separated_refs_by_book.entry(book) {
                    Occupied(mut o) => {
                        insert_in_order(o.get_mut(), PostReferences1::new(post_index, cv));
                    }
                    Vacant(v) => {
                        v.insert(BookReferences1::new(post_index, cv));
                    }
                }
            }
        }

        self.metadata.last().unwrap()
    }

    // extract bible references for a post and return any warnings
    pub fn extract_from_post(&mut self, post_metadata: Metadata, post_body: &str) -> Vec<String> {
        let (refs, warnings) = references(post_body);

        let m = self.insert(post_metadata, refs);

        let annotated_warnings = warnings.into_iter().map(|w| format!("{}: {}", &m.url, w));

        annotated_warnings.collect()
    }

    pub fn coelesce(&mut self) {
        self.refs_by_book = HashMap::<&str, BookReferences>::from_iter(
            self.separated_refs_by_book
                .drain()
                .map(|(k, v)| (k, BookReferences::from_separated(v))),
        );
    }
}

mod books;
mod extraction;
mod tabulation;
mod tests;
