use lazy_static::lazy_static;
use regex::Regex;
use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
    str::FromStr,
};

fn normalize_book(raw: &str) -> Option<&'static str> {
    lazy_static! {
        static ref book_list: Vec<Vec<&'static str>> = vec![
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
        static ref canonical: HashMap<&'static str, &'static str> = book_list
            .iter()
            .flat_map(|aliases| {
                aliases
                    .iter()
                    .map(|a| (*a, aliases[0]))
                    .collect::<Vec<(&str, &str)>>()
            })
            .collect();
    }

    canonical.get(raw).copied()
}

pub fn extract_bible_refs(text: &str) {
    lazy_static! {
        // TODO a reference may be either:
        // 1. book chapter, which we use for later context
        // 2. book chapter:verses, which we extract
        // 3. verse, which we extract using the stored context
        static ref REFERENCE_RE: Regex =
            Regex::new(r"([1-3]?)\s*([[:alpha:]]+)\s*(\d+:[\d:,\s-]+)").unwrap();
    }

    for cap in REFERENCE_RE.captures_iter(text) {
        let raw_book = if cap[1].is_empty() {
            cap[2].to_string()
        } else {
            format!("{} {}", &cap[1], &cap[2])
        };

        match normalize_book(&raw_book) {
            Some(book) => {
                let c_and_v = ChapterAndVerses::from_str(&cap[3]);

                println!("matched {} {:?}", book, c_and_v);
            }
            None => {
                println!("WARNING: unknown book {}", &raw_book);
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

#[derive(Eq, PartialEq, Debug)]
struct ChapterAndVerses {
    chapter: u8,
    verses: Vec<Verses>,
}

impl FromStr for ChapterAndVerses {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.split_once(':') {
            Some((s1, s2)) => match (s1.trim().parse::<u8>(), extract_verses(s2.trim())) {
                (Ok(c), Ok(vv)) => Ok(ChapterAndVerses {
                    chapter: c,
                    verses: vv,
                }),
                (Err(ec), Err(ev)) => Err(ParseError(format!("chapter: {}, verses: {}", ec, ev))),
                (Err(ec), _) => Err(ParseError::new(ec)),
                (_, Err(ev)) => Err(ParseError::new(ev)),
            },
            None => Err(ParseError::new("missing colon")),
        }
    }
}

#[derive(Eq, PartialEq, Debug)]
enum Verses {
    Single(u8),
    Range(u8, u8),
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

impl Display for Verses {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Verses::Single(v) => write!(f, "{}", v),
            Verses::Range(v1, v2) => write!(f, "{}-{}", v1, v2),
        }
    }
}

fn extract_verses(text: &str) -> Result<Vec<Verses>, ParseError> {
    fn verses_from_str_or_none(s: &str) -> Option<Result<Verses, ParseError>> {
        (!s.trim().is_empty()).then_some(Verses::from_str(s))
    }

    text.split(',')
        .filter_map(verses_from_str_or_none)
        .collect::<Result<Vec<Verses>, ParseError>>()
}

mod tests;
