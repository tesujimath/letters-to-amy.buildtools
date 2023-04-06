#![cfg(test)]

use super::*;

#[test]
fn test_slice_cmp() {
    use Ordering::*;

    assert_eq!(slice_cmp(&[1], &[1]), Equal);
    assert_eq!(slice_cmp(&[1], &[2]), Less);
    assert_eq!(slice_cmp(&[2], &[1]), Greater);
    assert_eq!(slice_cmp(&[1, 2], &[1, 2, 3]), Less);
    assert_eq!(slice_cmp(&[1, 2, 3], &[1, 2, 3]), Equal);
    assert_eq!(slice_cmp(&[1, 2], &[1, 3]), Less);
    assert_eq!(slice_cmp::<i32>(&[], &[]), Equal);
}

#[test]
fn test_insert_in_order() {
    let mut v = Vec::new();
    insert_in_order(&mut v, 2);
    insert_in_order(&mut v, 1);
    insert_in_order(&mut v, 2);
    insert_in_order(&mut v, 6);
    insert_in_order(&mut v, 3);

    let expected = vec![1, 2, 3, 6];
    assert_eq!(&v, &expected);
}
