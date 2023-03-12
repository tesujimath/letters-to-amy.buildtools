#![cfg(test)]

use super::*;

#[test]
fn test_verses_from_str() {
    assert_eq!(VSpan::from_str("7"), Ok(VSpan::at(7)));
    assert_eq!(VSpan::from_str("3-5"), Ok(VSpan::between(3, 5)));
    assert_eq!(VSpan::from_str(" 8  "), Ok(VSpan::at(8)));
    assert_eq!(VSpan::from_str(" 13   - 17 "), Ok(VSpan::between(13, 17)));
    assert!(VSpan::from_str("abc").is_err());
}

// helper for VSpan creation for tests
fn vspan(s: &str) -> VSpan {
    VSpan::from_str(s).unwrap()
}

#[test]
fn test_vspans_from_iter() {
    let result = VSpans::from_iter(vec![vspan("9"), vspan("1-7")]);
    let expected = VSpans(vec![vspan("1-7"), vspan("9")]);
    assert_eq!(result, expected);
}

#[test]
fn test_vspan_order() {
    use VSpan::*;

    assert!(Point(1) == Point(1));
    assert!(Point(1) < Point(2));
    assert!(Point(1) < Line(2, 3));
    assert!(Point(1) < Line(1, 3));
    assert!(Line(1, 2) < Line(2, 3));
    assert!(Line(1, 2) < Line(1, 3));
    assert!(Line(1, 2) < Line(1, 3));
    assert!(Line(1, 2) == Line(1, 2));
    assert!(Line(1, 2) < Point(4));
    assert!(Line(1, 3) < Point(2));
    assert!(Point(2) < Line(3, 5));
}

#[test]
fn test_vspans_order() {
    use VSpan::*;

    assert!(VSpans(vec![Point(1)]) == VSpans(vec![Point(1)]));
    assert!(VSpans(vec![Point(1)]) < VSpans(vec![Point(2)]));
    assert!(VSpans(vec![Point(1)]) < VSpans(vec![Line(1, 2)]));
    assert!(VSpans(vec![Point(1)]) < VSpans(vec![Point(1), Point(3)]));
    assert!(
        VSpans(vec![Point(1), Line(3, 4), Point(6)])
            == VSpans(vec![Point(1), Line(3, 4), Point(6)])
    );
    assert!(VSpans(vec![Point(1), Line(3, 4)]) < VSpans(vec![Point(1), Line(3, 5)]));
}

#[test]
fn test_vspans_insert_maintains_order() {
    let mut result = VSpans::new();

    result.insert(vspan("4"));
    result.insert(vspan("1-2"));
    result.insert(vspan("4"));
    result.insert(vspan("1-2"));

    let expected = VSpans(vec![vspan("1-2"), vspan("4")]);

    assert_eq!(result, expected);
}

#[test]
fn test_vspans_insert_coalesces_left() {
    let mut result = VSpans::new();

    result.insert(vspan("2"));
    result.insert(vspan("3"));

    let expected = VSpans(vec![vspan("2-3")]);

    assert_eq!(result, expected);
}

#[test]
fn test_vspans_insert_coalesces_right() {
    let mut result = VSpans::new();

    result.insert(vspan("3"));
    result.insert(vspan("1-2"));

    let expected = VSpans(vec![vspan("1-3")]);

    assert_eq!(result, expected);
}

#[test]
fn test_vspans_insert_coalesces_both() {
    let mut result = VSpans::new();

    result.insert(VSpan::at(1));
    result.insert(VSpan::at(20));
    result.insert(VSpan::at(13));
    result.insert(VSpan::at(10));
    result.insert(VSpan::between(11, 12));

    let expected = VSpans(vec![vspan("1"), vspan("10-13"), vspan("20")]);

    assert_eq!(result, expected);
}

#[test]
fn test_vspans_insert_coalesces_both_2() {
    let mut result = VSpans::new();

    result.insert(vspan("4"));
    result.insert(vspan("9"));
    result.insert(vspan("2"));
    result.insert(vspan("1-7"));

    let expected = VSpans(vec![vspan("1-7"), vspan("9")]);

    assert_eq!(result, expected);
}

#[test]
fn test_vspans_merge() {
    let mut s0 = VSpans::from_iter(vec![vspan("9"), vspan("1-7")]);
    let s1 = VSpans::from_iter(vec![
        VSpan::at(1),
        VSpan::at(2),
        VSpan::at(8),
        VSpan::between(11, 12),
    ]);

    s0.merge(s1);
    let expected = VSpans(vec![vspan("1-9"), vspan("11-12")]);
    assert_eq!(s0, expected);
}

#[test]
fn test_get_verses() {
    assert_eq!(
        get_verses("4, 9, 2, 1-7"),
        VSpans(vec![vspan("1-7"), vspan("9")])
    );
}

#[test]
fn test_chapters_verses_insert() {
    let mut cv = ChaptersVerses::new(ChapterVerses {
        chapter: Chapter(1),
        verses: get_verses("1-3"),
    });

    cv.insert(ChapterVerses {
        chapter: Chapter(2),
        verses: get_verses("4"),
    });
    assert_eq!(
        cv,
        ChaptersVerses(vec![
            ChapterVerses {
                chapter: Chapter(1),
                verses: get_verses("1-3"),
            },
            ChapterVerses {
                chapter: Chapter(2),
                verses: get_verses("4"),
            }
        ])
    );

    cv.insert(ChapterVerses {
        chapter: Chapter(1),
        verses: get_verses("4"),
    });
    assert_eq!(
        cv,
        ChaptersVerses(vec![
            ChapterVerses {
                chapter: Chapter(1),
                verses: get_verses("1-4"),
            },
            ChapterVerses {
                chapter: Chapter(2),
                verses: get_verses("4"),
            }
        ])
    );

    cv.insert(ChapterVerses {
        chapter: Chapter(2),
        verses: get_verses("6"),
    });
    assert_eq!(
        cv,
        ChaptersVerses(vec![
            ChapterVerses {
                chapter: Chapter(1),
                verses: get_verses("1-4"),
            },
            ChapterVerses {
                chapter: Chapter(2),
                verses: get_verses("4, 6"),
            }
        ])
    );
}
