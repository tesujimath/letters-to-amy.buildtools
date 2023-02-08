use lazy_static::lazy_static;
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

pub fn get_references(text: &str) -> References {
    lazy_static! {
        // TODO a reference may be either:
        // 1. book chapter, which we use for later context
        // 2. book chapter:verses, which we extract
        // 3. verse, which we extract using the stored context
        static ref REFERENCE_RE: Regex =
            Regex::new(r"(\bv([\d:,\s-]+)[ab]?)|(([1-3]?)\s*([A-Z][[:alpha:]]+)\s*(\d+)(:([\d:,\s-]+)[ab]?)?)").unwrap();
    }

    let mut references = References::new();
    let mut chapter_context: Option<ChapterContext> = None;

    for cap in REFERENCE_RE.captures_iter(text) {
        let fields = cap
            .iter()
            .skip(1)
            .map(|m_o| m_o.map(|m| m.as_str()))
            .collect::<Vec<Option<&str>>>();

        println!("{:?}", fields);

        let book = get_book(fields[3], fields[4]);
        let chapter_str = fields[5];
        if let (Some(book), Some(chapter_str)) = (book, chapter_str) {
            chapter_context = Some(ChapterContext {
                book,
                chapter: chapter_str.parse::<u8>().unwrap(),
            })
        }

        let verses = match (fields[1], fields[7]) {
            (Some(_), Some(_)) => panic!("not possible to have both verse alternatives"),
            (Some(v), None) => get_verses(v),
            (None, Some(v)) => get_verses(v),
            (None, None) => Ok(vec![]),
        };

        println!("B: {:?} {:?}", &chapter_context, &verses);

        match (chapter_context, verses) {
            (Some(ref ctx), Ok(verses)) => {
                if !verses.is_empty() {
                    references.insert(ctx.book, ChapterAndVerses::new(ctx.chapter, verses));
                }
            }
            (None, Ok(verses)) => {
                println!("WARNING: no context for verses {:?}", verses);
            }
            (_, Err(e)) => {
                println!("WARNING: error getting verses {}", e);
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

/// accumulated references, by book
#[derive(PartialEq, Eq, Debug)]
pub struct References(HashMap<&'static str, Vec<ChapterAndVerses>>);

impl References {
    fn new() -> References {
        References(HashMap::new())
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// insert verses for book, maintaining order
    fn insert(&mut self, book: &'static str, verses: ChapterAndVerses) {
        match self.0.get_mut(book) {
            Some(v) => match v.binary_search(&verses) {
                Ok(u) | Err(u) => v.insert(u, verses),
            },
            None => {
                self.0.insert(book, vec![verses]);
            }
        }
    }
}

#[derive(Eq, PartialEq, Debug)]
struct ChapterAndVerses {
    chapter: u8,
    verses: Vec<Verses>,
}

impl ChapterAndVerses {
    fn new(chapter: u8, verses: Vec<Verses>) -> ChapterAndVerses {
        assert!(!verses.is_empty());
        ChapterAndVerses { chapter, verses }
    }
}

impl PartialOrd for ChapterAndVerses {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.chapter.cmp(&other.chapter) {
            Ordering::Equal => Some(self.verses[0].cmp(&other.verses[0])),
            o => Some(o),
        }
    }
}

impl Ord for ChapterAndVerses {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.chapter.cmp(&other.chapter) {
            Ordering::Equal => self.verses[0].cmp(&other.verses[0]),
            o => o,
        }
    }
}

#[derive(Eq, PartialEq, Debug)]
enum Verses {
    Single(u8),
    Range(u8, u8),
}

impl Verses {
    fn first(&self) -> u8 {
        match self {
            Verses::Single(u) => *u,
            Verses::Range(u, _) => *u,
        }
    }
}

impl FromStr for Verses {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.split_once('-') {
            Some((s1, s2)) => match (s1.trim().parse::<u8>(), s2.trim().parse::<u8>()) {
                (Ok(v1), Ok(v2)) => Ok(Verses::Range(v1, v2)),
                (Err(e1), Err(e2)) => Err(ParseError(format!(
                    "Verses::from_str error: {}, {}",
                    e1, e2
                ))),
                (Err(e1), _) => Err(ParseError::new(e1)),
                (_, Err(e2)) => Err(ParseError::new(e2)),
            },
            None => match s.trim().parse::<u8>() {
                Ok(v) => Ok(Verses::Single(v)),
                Err(e) => Err(ParseError::new(e)),
            },
        }
    }
}

impl PartialOrd for Verses {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.first().cmp(&other.first()))
    }
}

impl Ord for Verses {
    fn cmp(&self, other: &Self) -> Ordering {
        self.first().cmp(&other.first())
    }
}

impl Display for Verses {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Verses::Single(v) => write!(f, "{}", v),
            Verses::Range(v1, v2) => write!(f, "{}-{}", v1, v2),
        }
    }
}

/// get verses from the text, and return in order
fn get_verses(text: &str) -> Result<Vec<Verses>, ParseError> {
    fn verses_from_str_or_none(s: &str) -> Option<Result<Verses, ParseError>> {
        (!s.trim().is_empty()).then_some(Verses::from_str(s))
    }

    text.split(',')
        .filter_map(verses_from_str_or_none)
        .collect::<Result<Vec<Verses>, ParseError>>()
        .map(|mut v| {
            v.sort();
            v
        })
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
