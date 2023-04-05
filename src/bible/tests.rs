#![cfg(test)]

use super::*;

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

#[test]
fn test_chapters_verses_chapter_leq() {
    assert!(mkcvs(vec![2, 3]).chapter_leq(Chapter(1)));
    assert!(mkcvs(vec![2, 3]).chapter_leq(Chapter(2)));
    assert!(!mkcvs(vec![2, 3]).chapter_leq(Chapter(3)));
    assert!(!mkcvs(vec![2, 3]).chapter_leq(Chapter(4)));
}

#[test]
fn test_chapters_verses_leq_chapters() {
    assert!(mkcvs(vec![1, 3]).leq_chapters(&mkcvs(vec![2, 3])));
    assert!(mkcvs(vec![2, 3]).leq_chapters(&mkcvs(vec![2, 3])));
    assert!(mkcvs(vec![2, 3]).leq_chapters(&mkcvs(vec![2, 4])));
    assert!(mkcvs(vec![2, 3]).leq_chapters(&mkcvs(vec![2, 3, 4])));
    assert!(mkcvs(vec![2]).leq_chapters(&mkcvs(vec![2, 3, 4])));

    assert!(!mkcvs(vec![2, 3]).leq_chapters(&mkcvs(vec![1, 3])));
    assert!(mkcvs(vec![2, 3]).leq_chapters(&mkcvs(vec![2, 3])));
    assert!(!mkcvs(vec![2, 4]).leq_chapters(&mkcvs(vec![2, 3])));
    assert!(!mkcvs(vec![2, 3, 4]).leq_chapters(&mkcvs(vec![2, 3])));
    assert!(!mkcvs(vec![2, 3, 4]).leq_chapters(&mkcvs(vec![2])));
}
