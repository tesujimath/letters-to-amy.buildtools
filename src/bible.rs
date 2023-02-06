use lazy_static::lazy_static;
use regex::Regex;

pub fn extract_bible_refs(text: &str) {
    lazy_static! {
        static ref BIBLE_REF_RE: Regex =
            Regex::new(r"([1-3]?)\s*([[:alpha:]]+)\s*(\d+:[\d:,\s-]+)").unwrap();
    }

    for cap in BIBLE_REF_RE.captures_iter(text) {
        println!("matched {}-{} {}", &cap[1], &cap[2], &cap[3])
    }
}
