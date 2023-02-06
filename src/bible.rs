use lazy_static::lazy_static;
use regex::Regex;

pub fn extract_bible_refs(text: &str) {
    lazy_static! {
        static ref BIBLE_REF_RE: Regex =
            Regex::new(r"([1-3]?)\s*([[:alpha:]]+)\s*(\d+:[\d:,\s-]+)").unwrap();
    }

    for cap in BIBLE_REF_RE.captures_iter(text) {
        let book = if cap[1].is_empty() {
            cap[2].to_string()
        } else {
            format!("{} {}", &cap[1], &cap[2])
        };
        let verses = &cap[3];

        println!("matched {} {}", book, verses);
    }
}
