use super::span::{Span, Spans};
use super::util::slice_cmp;
use itertools::Itertools;
use lazy_static::lazy_static;
use ref_cast::RefCast;
use regex::Regex;
use std::{
    cmp::Ordering,
    collections::HashMap,
    fmt::{self, Display, Formatter},
    num::ParseIntError,
    str::FromStr,
};

fn get_book(prefix: Option<&str>, alias: Option<&str>) -> Option<&'static str> {
    lazy_static! {
        static ref CANONICAL_MAP: HashMap<&'static str, &'static str> = book_aliases()
            .iter()
            .flat_map(|aliases| {
                aliases
                    .iter()
                    .map(|a| (*a, aliases[0]))
                    .collect::<Vec<(&str, &str)>>()
            })
            .collect();
    }

    match (prefix, alias) {
        (Some(prefix), Some(alias)) => {
            let raw_book = if prefix.is_empty() {
                alias.to_string()
            } else {
                format!("{} {}", prefix, alias)
            };

            CANONICAL_MAP.get(&raw_book as &str).copied()
        }
        _ => None,
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
struct Chapter(u8);

impl FromStr for Chapter {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        u8::from_str(s).map(Chapter)
    }
}

impl Display for Chapter {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Copy, Debug)]
struct ChapterContext<'a> {
    book: &'a str,
    chapter: Chapter,
}

pub fn get_chapter_and_verses_by_book(text: &str) -> BookChaptersVerses {
    lazy_static! {
        // TODO a reference may be either:
        // 1. book chapter, which we use for later context
        // 2. book chapter:verses, which we extract
        // 3. verse, which we extract using the stored context
        static ref REFERENCE_RE: Regex =
            Regex::new(r"(\bv([\d:,\s-]+)[ab]?)|(([1-3]?)\s*([A-Z][[:alpha:]]+)\s*(\d+)(:([\d:,\s-]+)[ab]?)?)").unwrap();
    }

    let mut references = BookChaptersVerses::new();
    let mut chapter_context: Option<ChapterContext> = None;

    for cap in REFERENCE_RE.captures_iter(text) {
        let fields = cap
            .iter()
            .skip(1)
            .map(|m_o| m_o.map(|m| m.as_str()))
            .collect::<Vec<Option<&str>>>();

        //println!("{:?}", fields);

        let book = get_book(fields[3], fields[4]);
        let chapter_str = fields[5];
        if let (Some(book), Some(chapter_str)) = (book, chapter_str) {
            chapter_context = Some(ChapterContext {
                book,
                chapter: chapter_str.parse::<Chapter>().unwrap(),
            })
        }

        let vspans = match (fields[1], fields[7]) {
            (Some(_), Some(_)) => panic!("not possible to have both verse alternatives"),
            (Some(v), None) => get_verses(v),
            (None, Some(v)) => get_verses(v),
            (None, None) => VSpans::new(),
        };

        match chapter_context {
            Some(ctx) => {
                references.insert(ctx.book, ChapterVerses::new(ctx.chapter, vspans));
            }
            None => {
                println!("WARN: missing context for {}", vspans)
            }
        }
    }

    references
}

#[derive(Eq, PartialEq, Debug)]
struct ParseError(String);

impl ParseError {
    fn new<T>(message: T) -> ParseError
    where
        T: Display,
    {
        ParseError(message.to_string())
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "parse error: {}", self.0)
    }
}

/// Span used for verses
#[derive(PartialEq, Eq, PartialOrd, Ord, RefCast, Debug)]
#[repr(transparent)]
struct VSpan(Span<u8>);

impl VSpan {
    fn at(x: u8) -> VSpan {
        VSpan(Span::at(x))
    }

    fn between(x: u8, y: u8) -> VSpan {
        VSpan(Span::between(x, y))
    }
}

impl FromStr for VSpan {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.split_once('-') {
            Some((s1, s2)) => match (s1.trim().parse::<u8>(), s2.trim().parse::<u8>()) {
                (Ok(v1), Ok(v2)) => Ok(VSpan::between(v1, v2)),
                (Err(e1), Err(e2)) => Err(ParseError(format!(
                    "Verses::from_str error: {}, {}",
                    e1, e2
                ))),
                (Err(e1), _) => Err(ParseError::new(e1)),
                (_, Err(e2)) => Err(ParseError::new(e2)),
            },
            None => match s.trim().parse::<u8>() {
                Ok(v) => Ok(VSpan::at(v)),
                Err(e) => Err(ParseError::new(e)),
            },
        }
    }
}

/// Spans used for verses
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
struct VSpans(Spans<u8>);

impl VSpans {
    fn new() -> VSpans {
        VSpans(Spans::new())
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn merge(&mut self, other: VSpans) {
        self.0.merge(other.0);
    }
}

impl<'a> IntoIterator for &'a VSpans {
    type Item = &'a VSpan;
    type IntoIter = std::iter::Map<std::slice::Iter<'a, Span<u8>>, fn(&Span<u8>) -> &VSpan>;

    fn into_iter(self) -> Self::IntoIter {
        fn wrap(unwrapped: &Span<u8>) -> &VSpan {
            VSpan::ref_cast(unwrapped)
        }
        self.0.into_iter().map(wrap)
    }
}

impl FromIterator<VSpan> for VSpans {
    fn from_iter<T: IntoIterator<Item = VSpan>>(iter: T) -> Self {
        fn unwrap(wrapped: VSpan) -> Span<u8> {
            wrapped.0
        }

        VSpans(Spans::from_iter(iter.into_iter().map(unwrap)))
    }
}

impl fmt::Display for VSpans {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        self.0.fmt(f)
    }
}

/// get verses from the text, and return in order
fn get_verses(text: &str) -> VSpans {
    fn vspan_from_str(s: &str) -> Option<VSpan> {
        VSpan::from_str(s).ok()
    }

    text.split(',')
        .filter_map(vspan_from_str)
        .collect::<VSpans>()
}

#[derive(PartialEq, Eq, Debug)]
pub struct ChapterVerses {
    chapter: Chapter,
    verses: VSpans,
}

impl ChapterVerses {
    fn new(chapter: Chapter, verses: VSpans) -> ChapterVerses {
        ChapterVerses { chapter, verses }
    }
}

impl PartialOrd for ChapterVerses {
    fn partial_cmp(&self, other: &ChapterVerses) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ChapterVerses {
    fn cmp(&self, other: &ChapterVerses) -> Ordering {
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
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        if self.verses.is_empty() {
            write!(f, "{}", self.chapter)
        } else {
            write!(f, "{}:{}", self.chapter, self.verses)
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct ChaptersVerses(Vec<ChapterVerses>);

impl ChaptersVerses {
    fn new(item: ChapterVerses) -> ChaptersVerses {
        ChaptersVerses(vec![item])
    }

    fn insert(&mut self, item: ChapterVerses) {
        match self.0.binary_search_by_key(&item.chapter, |cv| cv.chapter) {
            Ok(i) => self.0[i].verses.merge(item.verses),
            Err(i) => self.0.insert(i, item),
        }
    }
}

impl PartialOrd for ChaptersVerses {
    fn partial_cmp(&self, other: &ChaptersVerses) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ChaptersVerses {
    fn cmp(&self, other: &ChaptersVerses) -> Ordering {
        slice_cmp(&self.0, &other.0)
    }
}

impl<'a> IntoIterator for &'a ChaptersVerses {
    type Item = &'a ChapterVerses;
    type IntoIter = std::slice::Iter<'a, ChapterVerses>;

    fn into_iter(self) -> Self::IntoIter {
        (&self.0).into_iter()
    }
}

impl Display for ChaptersVerses {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        write!(
            f,
            "{}",
            self.0
                .iter()
                .map(|cv| cv.to_string())
                .intersperse("; ".to_string())
                .collect::<String>()
        )
    }
}

pub struct BookChaptersVerses(HashMap<&'static str, ChaptersVerses>);

impl BookChaptersVerses {
    fn new() -> BookChaptersVerses {
        BookChaptersVerses(HashMap::new())
    }

    pub fn get(&self, book: &'static str) -> Option<&ChaptersVerses> {
        self.0.get(book)
    }

    fn insert(&mut self, book: &'static str, cv: ChapterVerses) {
        match self.0.get_mut(book) {
            Some(entry) => entry.insert(cv),
            None => {
                self.0.insert(book, ChaptersVerses::new(cv));
            }
        }
    }
}

// TODO implement IntoIterator for BookChaptersVerses, maybe to take ownership

fn book_aliases() -> &'static Vec<Vec<&'static str>> {
    lazy_static! {
        static ref BOOK_LIST: Vec<Vec<&'static str>> = vec![
            vec!["Genesis", "Gen"],
            vec!["Exodus"],
            vec!["Leviticus"],
            vec!["Numbers"],
            vec!["Deuteronomy"],
            vec!["Joshua", "Josh"],
            vec!["Judges"],
            vec!["Ruth"],
            vec!["1 Samuel"],
            vec!["2 Samuel"],
            vec!["1 Kings"],
            vec!["2 Kings"],
            vec!["1 Chronicles"],
            vec!["2 Chronicles"],
            vec!["Ezra"],
            vec!["Nehemiah"],
            vec!["Esther"],
            vec!["Job"],
            vec!["Psalms", "Psalm"],
            vec!["Proverbs"],
            vec!["Ecclesiastes"],
            vec!["Song of Solomon"],
            vec!["Isaiah"],
            vec!["Jeremiah", "Jer"],
            vec!["Lamentations"],
            vec!["Ezekiel"],
            vec!["Daniel"],
            vec!["Hosea"],
            vec!["Joel"],
            vec!["Amos"],
            vec!["Obadiah"],
            vec!["Jonah"],
            vec!["Micah"],
            vec!["Nahum"],
            vec!["Habakkuk"],
            vec!["Zephaniah"],
            vec!["Haggai"],
            vec!["Zechariah"],
            vec!["Malachi"],
            vec!["Matthew"],
            vec!["Mark"],
            vec!["Luke"],
            vec!["John"],
            vec!["Acts"],
            vec!["Romans", "Rom"],
            vec!["1 Corinthians", "1 Cor"],
            vec!["2 Corinthians", "2 Cor"],
            vec!["Galatians"],
            vec!["Ephesians", "Eph"],
            vec!["Philippians", "Phil"],
            vec!["Colossians", "Col"],
            vec!["1 Thessalonians", "1 Thes"],
            vec!["2 Thessalonians", "2 Thes"],
            vec!["1 Timothy", "1 Tim"],
            vec!["2 Timothy", "2 Tim"],
            vec!["Titus"],
            vec!["Philemon"],
            vec!["Hebrews", "Heb"],
            vec!["James"],
            vec!["1 Peter"],
            vec!["2 Peter"],
            vec!["1 John", "1 Jn"],
            vec!["2 John"],
            vec!["3 John"],
            vec!["Jude"],
            vec!["Revelation", "Rev"],
        ];
    }

    &BOOK_LIST
}

pub fn books() -> impl Iterator<Item = &'static str> {
    book_aliases().iter().map(|aliases| aliases[0])
}

mod tests;
