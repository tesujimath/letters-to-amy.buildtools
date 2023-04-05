use lazy_static::lazy_static;
use std::collections::{HashMap, HashSet};

#[derive(Hash, Eq, PartialEq, Copy, Clone, Debug)]
pub enum Testament {
    Old,
    New,
}

impl Testament {
    pub fn name(&self) -> &'static str {
        use Testament::*;

        match self {
            Old => "Old Testament",
            New => "New Testament",
        }
    }

    pub fn all() -> impl Iterator<Item = Testament> {
        vec![Testament::Old, Testament::New].into_iter()
    }

    pub fn books(&self) -> impl Iterator<Item = &'static str> {
        book_alias_iter(*self).map(|aliases| (aliases[0]))
    }

    pub fn books_with_abbrev(&self) -> impl Iterator<Item = (&'static str, &'static str)> {
        book_alias_iter(*self).map(|aliases| (aliases[0], aliases[1]))
    }
}

pub fn book(prefix: Option<&str>, alias: Option<&str>) -> Option<&'static str> {
    lazy_static! {
        static ref CANONICAL_MAP: HashMap<&'static str, &'static str> = all_book_alias_iter()
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

pub fn is_single_chapter_book(book: &str) -> bool {
    lazy_static! {
        static ref SINGLE_CHAPTER_BOOK_SET: HashSet<&'static str> =
            single_chapter_book_data().iter().copied().collect();
    }

    SINGLE_CHAPTER_BOOK_SET.contains(book)
}

fn all_book_alias_iter() -> impl Iterator<Item = &'static Vec<&'static str>> {
    book_alias_iter(Testament::Old).chain(book_alias_iter(Testament::New))
}

fn book_alias_iter(testament: Testament) -> impl Iterator<Item = &'static Vec<&'static str>> {
    book_alias_data()[&testament].iter()
}

fn book_alias_data() -> &'static HashMap<Testament, Vec<Vec<&'static str>>> {
    lazy_static! {
        static ref BOOKS_BY_TESTAMENT: HashMap<Testament, Vec<Vec<&'static str>>> =
            HashMap::from([
                (
                    Testament::Old,
                    vec![
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
                    ]
                ),
                (
                    Testament::New,
                    vec![
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
                    ]
                )
            ]);
    }

    &BOOKS_BY_TESTAMENT
}

fn single_chapter_book_data() -> &'static Vec<&'static str> {
    lazy_static! {
        static ref BOOKS: Vec<&'static str> =
            vec!["Obadiah", "Philemon", "2 John", "3 John", "Jude",];
    }

    &BOOKS
}
