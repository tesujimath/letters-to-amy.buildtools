#![cfg(test)]

use super::*;

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
