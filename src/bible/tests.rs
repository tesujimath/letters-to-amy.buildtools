#![cfg(test)]

use super::*;

#[test]
fn test_references_insert() {
    fn insert(r: &mut References, book: &'static str, verses: &str) {
        r.insert(book, ChapterAndVerses::from_str(verses).unwrap());
    }

    const B1: &str = "Genesis";
    const B2: &str = "Exodus";

    let mut r = References::new();

    insert(&mut r, B1, "12:7");
    insert(&mut r, B1, "12:6");
    insert(&mut r, B2, "10:3-9");
    insert(&mut r, B2, "10:4");

    assert_eq!(
        r,
        References(HashMap::from([
            (
                B1,
                vec![
                    ChapterAndVerses {
                        chapter: 12,
                        verses: vec![Verses::Single(6)]
                    },
                    ChapterAndVerses {
                        chapter: 12,
                        verses: vec![Verses::Single(7)]
                    },
                ]
            ),
            (
                B2,
                vec![
                    ChapterAndVerses {
                        chapter: 10,
                        verses: vec![Verses::Range(3, 9)]
                    },
                    ChapterAndVerses {
                        chapter: 10,
                        verses: vec![Verses::Single(4)]
                    },
                ]
            ),
        ]))
    );
}

#[test]
fn test_chapter_and_verses_from_str() {
    assert_eq!(
        ChapterAndVerses::from_str("4:8"),
        Ok(ChapterAndVerses {
            chapter: 4,
            verses: vec![Verses::Single(8)]
        })
    );

    assert_eq!(
        ChapterAndVerses::from_str("4:8,"),
        Ok(ChapterAndVerses {
            chapter: 4,
            verses: vec![Verses::Single(8)]
        })
    );

    assert_eq!(
        ChapterAndVerses::from_str("17:8-9"),
        Ok(ChapterAndVerses {
            chapter: 17,
            verses: vec![Verses::Range(8, 9)]
        })
    );

    assert_eq!(
        ChapterAndVerses::from_str("11:1, 4, 8-11, 15"),
        Ok(ChapterAndVerses {
            chapter: 11,
            verses: vec![
                Verses::Single(1),
                Verses::Single(4),
                Verses::Range(8, 11),
                Verses::Single(15)
            ]
        })
    );
}

#[test]
fn test_verses_from_str() {
    assert_eq!(Verses::from_str("7"), Ok(Verses::Single(7)));
    assert_eq!(Verses::from_str("3-5"), Ok(Verses::Range(3, 5)));
    assert_eq!(Verses::from_str(" 8  "), Ok(Verses::Single(8)));
    assert_eq!(Verses::from_str(" 13   - 17 "), Ok(Verses::Range(13, 17)));
    assert!(Verses::from_str("abc").is_err());
}

#[test]
fn test_get_verses() {
    assert_eq!(
        get_verses("4, 2, 1-7"),
        Ok(vec![
            Verses::Range(1, 7),
            Verses::Single(2),
            Verses::Single(4)
        ])
    );
}
