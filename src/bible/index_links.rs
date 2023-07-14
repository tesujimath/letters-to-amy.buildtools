use std::{borrow::Cow, collections::BTreeMap, ops::Range};

use super::*;
use lazy_static::lazy_static;
use regex::Regex;

/// Return a potentially edited copy of the content with added index links
pub fn with_index_links(raw_header: &str, text: &str) -> Option<String> {
    lazy_static! {
        static ref CONTAINER_RE: Regex =
            Regex::new(r"\{\{<\s*quote\s*([^>]*)>}}|\(\s*([^)]*)\)").unwrap();

        static ref BOOK_RE: Regex =
            //            prefix     book
            Regex::new(r"([1-3]?)\s*([A-Z][[:alpha:]]+)").unwrap();
    }

    let mut segments = vec![Cow::Borrowed(raw_header)];
    let mut done = 0_usize;
    let mut updated = false;

    for (span, container) in containers(text) {
        use Container::*;

        segments.push(Cow::Borrowed(&text[done..span.start]));
        done = span.start;

        match container {
            Quoted(mut fields) => {
                if let Some(source) = fields.get("source") {
                    if let Some(book) = book(source) {
                        let url = format!("/ref/{}", slug::slugify(book));
                        if match fields.get("url") {
                            Some(original_url) => url != *original_url,
                            None => true,
                        } {
                            fields.insert("url", &url);
                            segments.push(Cow::Owned(Quoted(fields).to_string()));
                            done = span.end;
                            updated = true;
                        }
                    }
                }
            }

            Bracketed(bracketed_text) => {
                if let Some(_book) = book(bracketed_text) {
                    // decided not to put references in these for now
                    println!("WARNING: skipping ({})", bracketed_text);
                }
            }
        }
    }
    segments.push(Cow::Borrowed(&text[done..]));

    // only return string if we changed anything
    if updated {
        Some(segments.join(""))
    } else {
        None
    }
}

/// return the book if any found in text
fn book(text: &str) -> Option<&'static str> {
    lazy_static! {
        static ref BOOK_RE: Regex =
            //            prefix     book
            Regex::new(r"([1-3]?)\s*([A-Z][[:alpha:]]+)").unwrap();
    }
    BOOK_RE.captures(text).and_then(|cap| {
        super::books::book(
            cap.get(1).map(|m| m.as_str()),
            cap.get(2).map(|m| m.as_str()),
        )
    })
}

enum Container<'a> {
    Bracketed(&'a str),
    Quoted(BTreeMap<&'a str, &'a str>),
}

impl<'a> Display for Container<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        use Container::*;

        match self {
            Bracketed(bracketed_text) => write!(f, "({})", bracketed_text),
            Quoted(fields) => {
                write!(f, "{{{{< quote")?;
                for (k, v) in fields {
                    write!(f, r#" {}="{}""#, k, v)?;
                }
                write!(f, " >}}}}")
            }
        }
    }
}

fn containers(text: &str) -> impl Iterator<Item = (Range<usize>, Container)> {
    lazy_static! {
        static ref CONTAINER_RE: Regex =
            Regex::new(r"\{\{<\s*quote\s*([^>]*)>}}|\(\s*([^)]*)\)").unwrap();

        static ref BOOK_RE: Regex =
            //            prefix     book
            Regex::new(r"([1-3]?)\s*([A-Z][[:alpha:]]+)").unwrap();
    }

    CONTAINER_RE.captures_iter(text).map(|c| {
        if c.get(1).is_some() {
            // quote
            (
                c.get(0).unwrap().range(),
                Container::Quoted(fields(c.get(1).unwrap().as_str())),
            )
        } else {
            (
                c.get(0).unwrap().range(),
                Container::Bracketed(c.get(2).unwrap().as_str()),
            )
        }
    })
}

fn fields(text: &str) -> BTreeMap<&str, &str> {
    lazy_static! {
        static ref FIELDS_RE: Regex = Regex::new(r#"([a-z]+)="([^"]*)""#).unwrap();
    }

    let mut fields = BTreeMap::new();

    for cap in FIELDS_RE.captures_iter(text) {
        fields.insert(cap.get(1).unwrap().as_str(), cap.get(2).unwrap().as_str());
    }

    fields
}
