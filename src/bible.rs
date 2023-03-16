use super::util::slice_cmp;
use itertools::Itertools;
use lazy_static::lazy_static;
use regex::Regex;
use std::{
    cmp::{self, Ordering},
    collections::HashMap,
    fmt::{self, Display, Formatter},
    num::ParseIntError,
    str::FromStr,
};

fn book(prefix: Option<&str>, alias: Option<&str>) -> Option<&'static str> {
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

pub fn abbrev(name: &str) -> Option<&'static str> {
    lazy_static! {
        static ref ABBREV_MAP: HashMap<&'static str, &'static str> = book_aliases()
            .iter()
            .map(|aliases| (aliases[0], aliases[1]))
            .collect();
    }

    ABBREV_MAP.get(name).copied()
}

/// integer used for chapter index
type CInt = u8;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
struct Chapter(CInt);

impl FromStr for Chapter {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        CInt::from_str(s).map(Self)
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

pub fn get_references(text: &str) -> (References, Vec<String>) {
    lazy_static! {
        // 1. book chapter, which we use for later context
        // 2. book chapter:verses, which we extract, and store the context
        // 3. bare verse, which we extract using the stored context
        // 4. book verse
        static ref REFERENCE_RE: Regex =
            //           (bare verse          )(  prefix     book                  chapter verses)
            Regex::new(r"(\bv([\d:,\s-]+)[ab]?)|(([1-3]?)\s*([A-Z][[:alpha:]]+)\s*(\d{1,3}\b)(:(\d[ab\d:,\s-]*))?)").unwrap();
    }

    let mut references = References::new();
    let mut warnings = Vec::new();

    let mut chapter_context: Option<ChapterContext> = None;

    for cap in REFERENCE_RE.captures_iter(text) {
        let fields = cap
            .iter()
            .map(|m_o| m_o.map(|m| m.as_str()))
            .collect::<Vec<Option<&str>>>();

        let book = book(fields[4], fields[5]);
        let chapter_str = fields[6];
        if let (Some(book), Some(chapter_str)) = (book, chapter_str) {
            chapter_context = Some(ChapterContext {
                book,
                chapter: chapter_str.parse::<Chapter>().unwrap(),
            })
        }

        let vspans = match (fields[2], fields[8]) {
            (Some(_), Some(_)) => panic!("not possible to have both verse alternatives"),
            (Some(v), None) => get_verses(v),
            (None, Some(v)) => get_verses(v),
            (None, None) => VSpans::new(),
        };

        match chapter_context {
            Some(ctx) => {
                let cv = ChapterVerses::new(ctx.chapter, vspans);
                // useful for generating test data
                // println!(
                //     "{} -> {} {}: {:?}",
                //     fields[0].unwrap_or(" "),
                //     &ctx.book,
                //     &cv,
                //     &fields
                // );
                references.insert(ctx.book, cv);
            }
            None => {
                warnings.push(format!("missing context for '{}'", fields[0].unwrap_or("")));
            }
        }
    }

    (references, warnings)
}

#[derive(Eq, PartialEq, Debug)]
pub struct ParseError(String);

impl ParseError {
    fn new<T>(message: T) -> Self
    where
        T: Display,
    {
        Self(message.to_string())
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "parse error: {}", self.0)
    }
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
    fn at(x: VInt) -> Self {
        VSpan::Point(x)
    }

    fn between(from: VInt, to: VInt) -> Self {
        assert!(from <= to);

        VSpan::Line(from, to)
    }

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

    /// merge in other, which must be touching
    fn merge(&mut self, other: Self) {
        assert!(self.touches(&other));

        use VSpan::*;
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
    fn touches(&self, other: &Self) -> bool {
        !(self.upper() + 1 < other.lower() || self.lower() > other.upper() + 1)
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

/// trim whitespace and trailing a, b suffix
fn sanitize_verse(s: &str) -> &str {
    s.trim().trim_end_matches(|c| c == 'a' || c == 'b')
}

impl FromStr for VSpan {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.split_once('-') {
            Some((s1, s2)) => match (
                sanitize_verse(s1).parse::<VInt>(),
                sanitize_verse(s2).parse::<VInt>(),
            ) {
                (Ok(v1), Ok(v2)) => Ok(VSpan::between(v1, v2)),
                (Err(e1), Err(e2)) => Err(ParseError(format!(
                    "Verses::from_str error: {}, {}",
                    e1, e2
                ))),
                (Err(e1), _) => Err(ParseError::new(e1)),
                (_, Err(e2)) => Err(ParseError::new(e2)),
            },
            None => match sanitize_verse(s).parse::<VInt>() {
                Ok(v) => Ok(VSpan::at(v)),
                Err(e) => Err(ParseError::new(e)),
            },
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

    /// determine leftmost item from i-1 and i
    fn get_leftmost_touching(&self, i: usize, item: &VSpan) -> Option<usize> {
        let touching_left = i > 0 && self.0[i - 1].touches(item);
        let touching_this = i < self.0.len() && self.0[i].touches(item);

        match (touching_left, touching_this) {
            (false, false) => None,
            (true, _) => Some(i - 1),
            (false, true) => Some(i),
        }
    }

    fn insert(&mut self, item: VSpan) {
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

    fn merge(&mut self, other: Self) {
        for item in other.0 {
            self.insert(item);
        }
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

impl FromIterator<VSpan> for VSpans {
    fn from_iter<I: IntoIterator<Item = VSpan>>(iter: I) -> Self {
        let mut spans = Self::new();

        for s in iter {
            spans.insert(s);
        }

        spans
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
    fn new(chapter: Chapter, verses: VSpans) -> Self {
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
    fn new(item: ChapterVerses) -> Self {
        Self(vec![item])
    }

    fn insert(&mut self, item: ChapterVerses) {
        match self.0.binary_search_by_key(&item.chapter, |cv| cv.chapter) {
            Ok(i) => self.0[i].verses.merge(item.verses),
            Err(i) => self.0.insert(i, item),
        }
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

impl Display for ChaptersVerses {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        write!(
            f,
            "{}",
            Itertools::intersperse(self.0.iter().map(|cv| cv.to_string()), "; ".to_string())
                .collect::<String>()
        )
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct References(HashMap<&'static str, ChaptersVerses>);

impl References {
    fn new() -> Self {
        Self(HashMap::new())
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

    /// non-consuming iterator
    pub fn iter(&self) -> std::collections::hash_map::Iter<&'static str, ChaptersVerses> {
        self.0.iter()
    }
}

/// consuming iterator
impl IntoIterator for References {
    type Item = (&'static str, ChaptersVerses);
    type IntoIter = std::collections::hash_map::IntoIter<&'static str, ChaptersVerses>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

fn book_aliases() -> &'static Vec<Vec<&'static str>> {
    lazy_static! {
        static ref BOOK_LIST: Vec<Vec<&'static str>> = vec![
            vec!["Genesis", "Gen"],
            vec!["Exodus", "Ex"],
            vec!["Leviticus", "Lev"],
            vec!["Numbers", "Num"],
            vec!["Deuteronomy", "Deut"],
            vec!["Joshua", "Josh"],
            vec!["Judges", "Judg"],
            vec!["Ruth", "Ruth"],
            vec!["1 Samuel", "1 Sam"],
            vec!["2 Samuel", "2 Sam"],
            vec!["1 Kings", "1 Kgs"],
            vec!["2 Kings", "2 Kgs"],
            vec!["1 Chronicles", "1 Chr"],
            vec!["2 Chronicles", "2 Chr"],
            vec!["Ezra", "Ezra"],
            vec!["Nehemiah", "Neh"],
            vec!["Esther", "Est"],
            vec!["Job", "Job"],
            vec!["Psalms", "Ps", "Psalm"],
            vec!["Proverbs", "Prv"],
            vec!["Ecclesiastes", "Ecc"],
            vec!["Song of Solomon", "Song"],
            vec!["Isaiah", "Is"],
            vec!["Jeremiah", "Jer"],
            vec!["Lamentations", "Lam"],
            vec!["Ezekiel", "Ezk"],
            vec!["Daniel", "Dan"],
            vec!["Hosea", "Hos"],
            vec!["Joel", "Joel"],
            vec!["Amos", "Amos"],
            vec!["Obadiah", "Ob"],
            vec!["Jonah", "Jnh"],
            vec!["Micah", "Mic"],
            vec!["Nahum", "Nah"],
            vec!["Habakkuk", "Hab"],
            vec!["Zephaniah", "Zeph"],
            vec!["Haggai", "Hag"],
            vec!["Zechariah", "Zech"],
            vec!["Malachi", "Mal"],
            vec!["Matthew", "Mt"],
            vec!["Mark", "Mk"],
            vec!["Luke", "Lk"],
            vec!["John", "Jn"],
            vec!["Acts", "Acts"],
            vec!["Romans", "Rom"],
            vec!["1 Corinthians", "1 Cor"],
            vec!["2 Corinthians", "2 Cor"],
            vec!["Galatians", "Gal"],
            vec!["Ephesians", "Eph"],
            vec!["Philippians", "Phil"],
            vec!["Colossians", "Col"],
            vec!["1 Thessalonians", "1 Thes"],
            vec!["2 Thessalonians", "2 Thes"],
            vec!["1 Timothy", "1 Tim"],
            vec!["2 Timothy", "2 Tim"],
            vec!["Titus", "Ti"],
            vec!["Philemon", "Phm"],
            vec!["Hebrews", "Heb"],
            vec!["James", "Jam"],
            vec!["1 Peter", "1 Pet"],
            vec!["2 Peter", "2 Pet"],
            vec!["1 John", "1 Jn"],
            vec!["2 John", "2 Jn"],
            vec!["3 John", "3 Jn"],
            vec!["Jude", "Jude"],
            vec!["Revelation", "Rev"],
        ];
    }

    &BOOK_LIST
}

pub fn books() -> impl Iterator<Item = &'static str> {
    book_aliases().iter().map(|aliases| aliases[0])
}

mod tests;
