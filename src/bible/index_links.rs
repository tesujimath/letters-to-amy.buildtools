use std::{borrow::Cow, collections::BTreeMap, ops::Range};

use super::*;
use lazy_static::lazy_static;
use regex::Regex;

/// Return a potentially edited copy of the content with added index links
pub fn with_index_links(raw_header: &str, text: &str) -> Option<String> {
    let mut segments = vec![Cow::Borrowed(raw_header)];
    let mut done = 0_usize;
    let mut updated = false;

    for (span, mut quote) in quotes(text) {
        segments.push(Cow::Borrowed(&text[done..span.start]));
        done = span.start;

        if let Some(source) = quote.source() {
            if let Some(book) = book(source) {
                let url = format!("/ref/{}", slug::slugify(book));
                if match quote.url() {
                    Some(original_url) => url != *original_url,
                    None => true,
                } {
                    quote.set_url(&url);
                    segments.push(Cow::Owned(format!("{}", &quote)));
                    done = span.end;
                    updated = true;
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

fn quotes(text: &str) -> impl Iterator<Item = (Range<usize>, Quote)> {
    lazy_static! {
        static ref QUOTE_RE: Regex = Regex::new(r"\{\{<\s*quote\s*([^>]*)>}}").unwrap();
        static ref FIELDS_RE: Regex = Regex::new(r#"([a-z]+)="([^"]*)""#).unwrap();
    }

    QUOTE_RE.captures_iter(text).map(|c| {
        let mut fields = BTreeMap::new();

        for cap in FIELDS_RE.captures_iter(c.get(1).unwrap().as_str()) {
            fields.insert(cap.get(1).unwrap().as_str(), cap.get(2).unwrap().as_str());
        }
        (c.get(0).unwrap().range(), Quote(fields))
    })
}

struct Quote<'a>(BTreeMap<&'a str, &'a str>);

impl<'a> Quote<'a> {
    fn source(&self) -> Option<&str> {
        self.0.get("source").copied()
    }

    fn url(&self) -> Option<&str> {
        self.0.get("url").copied()
    }

    fn set_url(&mut self, value: &'a str) {
        self.0.insert("url", value);
    }
}

impl<'a> Display for Quote<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{{{{< quote")?;
        for (k, v) in &self.0 {
            write!(f, r#" {}="{}""#, k, v)?;
        }
        write!(f, " >}}}}")
    }
}
