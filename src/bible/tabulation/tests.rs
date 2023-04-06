#![cfg(test)]

use super::super::*;
use super::*;
use test_case::test_case;

/*
#[test_case(
    vec![(1, 10), (1, 11)],
    vec![(1, vec![10, 11])];
    "simple concatenation")]
*/
#[test_case(
    vec![(1, 10), (2, 11), (1, 12)],
    vec![(1, vec![10]), (2, vec![11]), (1, vec![12])];
    "simple concatenation, two posts")]
/*
#[test_case(
    vec![(1, 10), (2, 11), (1, 11)],
    vec![(1, vec![10, 11]), (2, vec![11])];
    "sliding up")]
#[test_case(
    vec![(1, 10), (2, 10), (1, 11)],
    vec![(2, vec![10]), (1, vec![10, 11])];
    "sliding down")]
#[test_case(
    vec![(1, 10), (2, 10), (3, 11), (1, 11)],
    vec![(2, vec![10]), (1, vec![10, 11]), (3, vec![11])];
    "sliding up and down")]
#[test_case(
    vec![(1, 10), (2, 10), (2, 11), (1, 11)],
    vec![(1, vec![10, 11]), (2, vec![10, 11])];
    "sliding up across merged chapters")]
#[test_case(
    vec![(1, 10), (2, 10), (3, 10), (3, 11), (1, 11)],
    vec![(2, vec![10]), (1, vec![10, 11]), (3, vec![10, 11])];
    "sliding up and down across merged chapters")]
*/
fn test_book_references_from_separated(
    refs1: Vec<(usize, CInt)>,
    expected: Vec<(usize, Vec<CInt>)>,
) {
    fn mkcv(c: CInt) -> ChapterVerses {
        ChapterVerses::new(Some(Chapter(c)), VSpans::new())
    }

    fn mkrefs1(pcs: Vec<(usize, CInt)>) -> BookReferences1 {
        let mut refs1 = BookReferences1::new(pcs[0].0, mkcv(pcs[0].1));

        for pc in pcs.iter().skip(1) {
            refs1.push(PostReferences1::new(pc.0, mkcv(pc.1)));
        }

        refs1
    }

    fn unpack(refs: BookReferences) -> Vec<(usize, Vec<CInt>)> {
        refs.iter()
            .map(|p| {
                (
                    p.post_index,
                    p.cvs.iter().map(|cv| cv.chapter.unwrap().0).collect(),
                )
            })
            .collect()
    }

    assert_eq!(
        unpack(BookReferences::from_separated(mkrefs1(refs1))),
        expected
    );
}
