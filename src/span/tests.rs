#![cfg(test)]

use super::*;

#[test]
fn test_span_order() {
    use Span::*;

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
fn test_spans_insert_maintains_order() {
    let mut sut = Spans::new();

    sut.insert(Span::at(4));
    sut.insert(Span::between(1, 2));
    sut.insert(Span::at(4));
    sut.insert(Span::between(1, 2));

    let expected = vec![Span::between(1, 2), Span::at(4)];
    let result = (&sut).into_iter().collect::<Vec<&Span<i32>>>();

    assert_eq!(result, expected.iter().collect::<Vec<&Span<i32>>>());
}

#[test]
fn test_spans_insert_coalesces_left() {
    let mut sut = Spans::new();

    sut.insert(Span::at(2));
    sut.insert(Span::at(3));

    let expected = vec![Span::between(2, 3)];
    let result = (&sut).into_iter().collect::<Vec<&Span<i32>>>();

    assert_eq!(result, expected.iter().collect::<Vec<&Span<i32>>>());
}

#[test]
fn test_spans_insert_coalesces_right() {
    let mut sut = Spans::new();

    sut.insert(Span::at(3));
    sut.insert(Span::between(1, 2));

    let expected = vec![Span::between(1, 3)];
    let result = (&sut).into_iter().collect::<Vec<&Span<i32>>>();

    assert_eq!(result, expected.iter().collect::<Vec<&Span<i32>>>());
}

#[test]
fn test_spans_insert_coalesces_both() {
    let mut sut = Spans::new();

    sut.insert(Span::at(-10));
    sut.insert(Span::at(10));
    sut.insert(Span::at(3));
    sut.insert(Span::at(0));
    sut.insert(Span::between(1, 2));

    let expected = vec![Span::at(-10), Span::between(0, 3), Span::at(10)];
    let result = (&sut).into_iter().collect::<Vec<&Span<i32>>>();

    assert_eq!(result, expected.iter().collect::<Vec<&Span<i32>>>());
}
