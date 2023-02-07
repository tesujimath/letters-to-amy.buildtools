use lazy_static::lazy_static;
use regex::Regex;
use std::{
    fmt::{self, Display, Formatter},
    str::FromStr,
};

pub fn extract_bible_refs(text: &str) {
    lazy_static! {
        static ref FULL_REFERENCE_RE: Regex =
            Regex::new(r"([1-3]?)\s*([[:alpha:]]+)\s*(\d+:[\d:,\s-]+)").unwrap();
    }

    for cap in FULL_REFERENCE_RE.captures_iter(text) {
        let book = if cap[1].is_empty() {
            cap[2].to_string()
        } else {
            format!("{} {}", &cap[1], &cap[2])
        };

        let c_and_v = ChapterAndVerses::from_str(&cap[3]);

        println!("matched {} {:?}", book, c_and_v);
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
