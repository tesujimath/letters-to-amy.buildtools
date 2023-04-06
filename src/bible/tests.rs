#![cfg(test)]

use super::*;
use test_case::test_case;

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

fn mkcv(c: CInt) -> ChapterVerses {
    ChapterVerses::new(Some(Chapter(c)), VSpans::new())
}

fn mkcvs(cs: Vec<CInt>) -> ChaptersVerses {
    let mut cvs = ChaptersVerses::new(mkcv(cs[0]));
    for c in cs.iter().skip(1) {
        cvs.0.push(mkcv(*c))
    }
    cvs
}

fn mkcvv(c: CInt, vs: Vec<VInt>) -> ChapterVerses {
    let vspans = VSpans(vs.into_iter().map(VSpan::Point).collect::<Vec<VSpan>>());

    ChapterVerses::new(Some(Chapter(c)), vspans)
}

fn mkcvvs(cvs: Vec<(CInt, Vec<VInt>)>) -> ChaptersVerses {
    ChaptersVerses(
        cvs.into_iter()
            .map(|(c, vs)| mkcvv(c, vs))
            .collect::<Vec<ChapterVerses>>(),
    )
}

#[test_case(
    (1, vec![10, 11]),
    (2, vec![1]),
    Ordering::Less;
    "all less")]
#[test_case(
    (1, vec![10, 11]),
    (1, vec![12]),
    Ordering::Less;
    "verses all less")]
#[test_case(
    (1, vec![10, 12]),
    (1, vec![11]),
    Ordering::Less;
    "initial verse less")]
fn test_chapter_verses_order(cv1: (CInt, Vec<VInt>), cv2: (CInt, Vec<VInt>), expected: Ordering) {
    assert_eq!(&mkcvv(cv1.0, cv1.1).cmp(&mkcvv(cv2.0, cv2.1)), &expected);
}

#[test_case(
    vec![(1, vec![10, 11]), (2, vec![10])],
    vec![(3, vec![1])],
    Ordering::Less;
    "all less")]
fn test_chapters_verses_order(
    cvs1: Vec<(CInt, Vec<VInt>)>,
    cvs2: Vec<(CInt, Vec<VInt>)>,
    expected: Ordering,
) {
    assert_eq!(&mkcvvs(cvs1).cmp(&mkcvvs(cvs2)), &expected);
}

#[test]
fn test_chapters_verses_chapter_leq() {
    assert!(mkcvs(vec![2, 3]).chapter_leq(Chapter(1)));
    assert!(mkcvs(vec![2, 3]).chapter_leq(Chapter(2)));
    assert!(!mkcvs(vec![2, 3]).chapter_leq(Chapter(3)));
    assert!(!mkcvs(vec![2, 3]).chapter_leq(Chapter(4)));
}

#[test]
fn test_chapters_verses_leq_chapters() {
    assert!(mkcvs(vec![1]).leq_chapters(&mkcvs(vec![1])));
    assert!(mkcvs(vec![1]).leq_chapters(&mkcvs(vec![2])));
    assert!(mkcvs(vec![1, 2]).leq_chapters(&mkcvs(vec![2, 3])));
    assert!(mkcvs(vec![2, 3]).leq_chapters(&mkcvs(vec![2, 3])));
    assert!(mkcvs(vec![2, 3]).leq_chapters(&mkcvs(vec![2, 4])));
    assert!(mkcvs(vec![2, 3]).leq_chapters(&mkcvs(vec![2, 3, 4])));
    assert!(mkcvs(vec![2]).leq_chapters(&mkcvs(vec![2, 3, 4])));

    assert!(!mkcvs(vec![1, 3]).leq_chapters(&mkcvs(vec![2, 3])));
    assert!(!mkcvs(vec![2, 3]).leq_chapters(&mkcvs(vec![1, 3])));
    assert!(!mkcvs(vec![2, 4]).leq_chapters(&mkcvs(vec![2, 3])));
    assert!(!mkcvs(vec![2, 3, 4]).leq_chapters(&mkcvs(vec![2, 3])));
    assert!(!mkcvs(vec![2, 3, 4]).leq_chapters(&mkcvs(vec![2])));

    assert!(!mkcvs(vec![10, 12]).leq_chapters(&mkcvs(vec![11])));
}
