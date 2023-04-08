#![cfg(test)]

use super::*;
use test_case::test_case;
use Ordering::*;

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

fn create_chapter_verses(c: CInt, vs: Vec<VInt>) -> ChapterVerses {
    let vspans = VSpans(vs.into_iter().map(VSpan::Point).collect::<Vec<VSpan>>());

    ChapterVerses::new(Some(Chapter(c)), vspans)
}

#[test_case(
    (1, vec![10, 11]),
    (2, vec![1]),
    Less;
    "all less")]
#[test_case(
    (1, vec![10, 11]),
    (1, vec![12]),
    Less;
    "verses all less")]
#[test_case(
    (1, vec![10, 12]),
    (1, vec![11]),
    Less;
    "initial verse less 1")]
#[test_case(
    (1, vec![10, 12, 13]),
    (1, vec![11, 15, 16]),
    Less;
    "initial verse less 2")]
fn test_chapter_verses_cmp(cv1: (CInt, Vec<VInt>), cv2: (CInt, Vec<VInt>), expected: Ordering) {
    assert_eq!(
        &create_chapter_verses(cv1.0, cv1.1).cmp(&create_chapter_verses(cv2.0, cv2.1)),
        &expected
    );
}

#[test_case(
    vec![(1, vec![10, 11]), (2, vec![10]), (3, vec![4])],
    vec![(1, vec![10, 11]), (2, vec![10]), (3, vec![4])],
    Some(Equal);
    "same chapters and verses")]
#[test_case(
    vec![(1, vec![9, 11]), (2, vec![10]), (3, vec![4])],
    vec![(1, vec![10, 11]), (2, vec![10]), (3, vec![4])],
    Some(Less);
    "same chapters less verses 1")]
#[test_case(
    vec![(1, vec![10, 11]), (2, vec![10]), (3, vec![3])],
    vec![(1, vec![10, 11]), (2, vec![10]), (3, vec![4])],
    Some(Less);
    "same chapters less verses 2")]
#[test_case(
    vec![(1, vec![10, 11]), (2, vec![10])],
    vec![(3, vec![1])],
    Some(Less);
    "all less")]
#[test_case(
    vec![(1, vec![1]), (3, vec![1])],
    vec![(2, vec![1])],
    None;
    "in-between chapters 1")]
#[test_case(
    vec![(1, vec![1]), (2, vec![1]), (3, vec![1])],
    vec![(2, vec![1])],
    None;
    "in-between chapters 2")]
#[test_case(
    vec![(1, vec![1]), (2, vec![2])],
    vec![(2, vec![1])],
    Some(Less);
    "in-between verses")]
#[test_case(
    vec![(1, vec![1]), (3, vec![1])],
    vec![(2, vec![1]), (4, vec![1])],
    None;
    "interleaved chapters")]
fn test_chapters_verses_partial_cmp(
    cvs1: Vec<(CInt, Vec<VInt>)>,
    cvs2: Vec<(CInt, Vec<VInt>)>,
    expected: Option<Ordering>,
) {
    fn create_chapters_verses(cvs: Vec<(CInt, Vec<VInt>)>) -> ChaptersVerses {
        ChaptersVerses(
            cvs.into_iter()
                .map(|(c, vs)| create_chapter_verses(c, vs))
                .collect::<Vec<ChapterVerses>>(),
        )
    }

    assert_eq!(
        &create_chapters_verses(cvs1).partial_cmp(&create_chapters_verses(cvs2)),
        &expected
    );
}
