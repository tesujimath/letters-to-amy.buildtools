use super::span::{Span, Spans};
use lazy_static::lazy_static;
use ref_cast::RefCast;
use regex::Regex;
use std::{
    cmp::Ordering,
    collections::HashMap,
    fmt::{self, Display, Formatter},
    str::FromStr,
};

fn get_book(prefix: Option<&str>, alias: Option<&str>) -> Option<&'static str> {
    lazy_static! {
        static ref CANONICAL_MAP: HashMap<&'static str, &'static str> = book_list()
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

#[derive(Clone, Copy, Debug)]
struct ChapterContext<'a> {
    book: &'a str,
    chapter: u8,
}

pub fn dump_chapter_and_verses_by_book(text: &str) {
    lazy_static! {
        // TODO a reference may be either:
        // 1. book chapter, which we use for later context
        // 2. book chapter:verses, which we extract
        // 3. verse, which we extract using the stored context
        static ref REFERENCE_RE: Regex =
            Regex::new(r"(\bv([\d:,\s-]+)[ab]?)|(([1-3]?)\s*([A-Z][[:alpha:]]+)\s*(\d+)(:([\d:,\s-]+)[ab]?)?)").unwrap();
    }

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
                chapter: chapter_str.parse::<u8>().unwrap(),
            })
        }

        let vspans = match (fields[1], fields[7]) {
            (Some(_), Some(_)) => panic!("not possible to have both verse alternatives"),
            (Some(v), None) => get_verses(v),
            (None, Some(v)) => get_verses(v),
            (None, None) => VSpans::new(),
        };

        match chapter_context {
            Some(c) => {
                if vspans.is_empty() {
                    println!("{} {}", c.book, c.chapter);
                } else {
                    println!("{} {}:{}", c.book, c.chapter, vspans);
                }
            }
            None => {
                println!("WARN: missing context for {}", vspans)
            }
        }
    }
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
#[derive(Eq, PartialEq, RefCast, Debug)]
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

impl PartialOrd for VSpan {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for VSpan {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
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
#[derive(Eq, PartialEq, Debug)]
struct VSpans(Spans<u8>);

impl VSpans {
    fn new() -> VSpans {
        VSpans(Spans::new())
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
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
        let mut spans = Spans::new();

        for VSpan(s) in iter {
            spans.insert(s);
        }

        VSpans(spans)
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

fn book_list() -> &'static Vec<Vec<&'static str>> {
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

mod tests;
