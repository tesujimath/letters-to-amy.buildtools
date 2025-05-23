use super::hugo::Metadata;
use super::util::slice_cmp;
use books::{book, is_single_chapter_book};
use itertools::Itertools;
use std::{
    cmp::{self, Ordering},
    collections::{BTreeMap, HashMap},
    fmt::{self, Display, Formatter},
    num::ParseIntError,
    str::FromStr,
};
pub use tabulation::Writer;
use tabulation::{BookReferences, BookReferences1};

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
#[derive(PartialEq, Eq, Clone, Debug)]
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
#[derive(PartialEq, Eq, Clone, Debug)]
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
            Itertools::intersperse(self.0.iter().map(|s| s.to_string()), ",\u{200A}".to_string())
                .collect::<String>()
        )
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
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

#[derive(PartialEq, Eq, Clone, Debug)]
// a non-empty list of chapter/verses references, with chapters strictly increasing
pub struct ChaptersVerses(Vec<ChapterVerses>);

impl ChaptersVerses {
    fn new(item: ChapterVerses) -> Self {
        Self(vec![item])
    }
}

impl PartialOrd for ChaptersVerses {
    /// To determine whether there is an ordering, verses are ignored,
    /// only chapters matter, but these must be strictly in order, with no interleaving.
    /// If an ordering is possible, the earliest verses that differ influence the order.
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        fn compatible(o1: Ordering, o2: Ordering) -> bool {
            !matches!((o1, o2), (Less, Greater) | (Greater, Less))
        }

        fn all_compatible(o: Ordering, cvs0: &[&ChapterVerses], cvs1: &[&ChapterVerses]) -> bool {
            cvs0.iter().all(|cv0| {
                cvs1.iter()
                    .all(|cv1| compatible(o, cv0.chapter.cmp(&cv1.chapter)))
            })
        }

        let mut it0 = self.0.iter().peekable();
        let mut it1 = other.0.iter().peekable();

        use Ordering::*;
        let mut verse_order: Option<Ordering> = None;

        loop {
            match (it0.peek(), it1.peek()) {
                (Some(cv0), Some(cv1)) => {
                    match cv0.chapter.cmp(&cv1.chapter) {
                        Equal => {
                            if verse_order.is_none() || verse_order == Some(Equal) {
                                verse_order = Some(cv0.cmp(cv1));
                            }

                            // skip both and keep comparing
                            it0.next();
                            it1.next();
                        }
                        candidate => {
                            let v0 = it0.collect::<Vec<&ChapterVerses>>();
                            let v1 = it1.collect::<Vec<&ChapterVerses>>();
                            if all_compatible(candidate, &v0, &v1) {
                                return Some(candidate);
                            } else {
                                return None;
                            }
                        }
                    }
                }
                (Some(_), None) => return Some(Greater),
                (None, Some(_)) => return Some(Less),
                (None, None) => return verse_order.or(Some(Equal)),
            }
        }
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

#[derive(Debug)]
pub struct AllReferences {
    metadata: Vec<Metadata>,
    post_index_by_epoch: BTreeMap<i64, usize>,
    post_sequence_number_by_index: Vec<Option<usize>>,
    separated_refs_by_book: HashMap<&'static str, BookReferences1>,
    refs_by_book: HashMap<&'static str, BookReferences>,
}

mod books;
mod extraction;
pub use extraction::references;
mod index_links;
pub use index_links::with_index_links;
mod tabulation;
mod tests;
