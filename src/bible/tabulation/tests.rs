#![cfg(test)]

use super::super::*;
use super::*;
use test_case::test_case;

fn unpack(refs: BookReferences) -> Vec<(usize, Vec<CInt>)> {
    refs.iter()
        .map(|p| {
            (
                p.post_index,
                p.cvs.0.iter().map(|cv| cv.chapter.unwrap().0).collect(),
            )
        })
        .collect()
}

#[test_case(
    vec![(1, 10, 1), (1, 11, 1)],
    vec![(1, vec![10, 11])];
    "simple concatenation")]
#[test_case(
    vec![(1, 10, 1), (2, 11, 1), (1, 12, 1)],
    vec![(1, vec![10]), (2, vec![11]), (1, vec![12])];
    "simple concatenation, two posts")]
#[test_case(
    vec![(1, 10, 1), (2, 11, 1), (1, 11, 1)],
    vec![(1, vec![10, 11]), (2, vec![11])];
    "merge in place")]
#[test_case(
    vec![(1, 10, 1), (2, 10, 1), (1, 11, 1)],
    vec![(2, vec![10]), (1, vec![10, 11])];
    "move to end and merge")]
#[test_case(
    vec![(1, 10, 1), (2, 10, 1), (3, 11, 1), (1, 11, 1)],
    vec![(2, vec![10]), (1, vec![10, 11]), (3, vec![11])];
    "move to middle and merge")]
#[test_case(
    vec![(1, 10, 1), (2, 10, 1), (2, 11, 1), (1, 11, 1)],
    vec![(1, vec![10, 11]), (2, vec![10, 11])];
    "merge with already merged")]
#[test_case(
    vec![(1, 10, 1), (2, 10, 1), (3, 10, 1), (3, 11, 1), (1, 11, 1)],
    vec![(2, vec![10]), (1, vec![10, 11]), (3, vec![10, 11])];
    "move to middle and merge with already merged")]
#[test_case(
    vec![(1, 10, 1), (2, 10, 2), (1, 11, 1), (2, 11, 3)],
    vec![(1, vec![10, 11]), (2, vec![10, 11])];
    "interleaved 1")]
#[test_case(
    vec![(1, 10, 2), (2, 10, 1), (1, 11, 4), (2, 11, 3)],
    vec![(2, vec![10, 11]), (1, vec![10, 11])];
    "interleaved 2")]
fn test_book_references_from_separated(
    refs1: Vec<(usize, CInt, VInt)>,
    expected: Vec<(usize, Vec<CInt>)>,
) {
    fn create_chapter_verses(c: CInt, v: VInt) -> ChapterVerses {
        let vs = VSpans(vec![VSpan::Point(v)]);
        ChapterVerses::new(Some(Chapter(c)), vs)
    }

    fn create_book_references_1(pcs: Vec<(usize, CInt, VInt)>) -> BookReferences1 {
        let mut refs1 = BookReferences1::new(pcs[0].0, create_chapter_verses(pcs[0].1, pcs[0].2));

        for pc in pcs.iter().skip(1) {
            refs1.push(PostReferences1::new(
                pc.0,
                create_chapter_verses(pc.1, pc.2),
            ));
        }

        refs1
    }

    assert_eq!(
        unpack(BookReferences::from_separated(create_book_references_1(
            refs1
        ))),
        expected
    );
}
