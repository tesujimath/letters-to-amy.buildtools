#![cfg(test)]

use super::*;

#[test]
fn test_skip_header() {
    assert_eq!(skip_header("abc"), "abc");
    assert_eq!(skip_header("+++ some header fields +++abc"), "abc");
    assert_eq!(
        skip_header(
            r###"+++
title: some title
date: 2023-02-07
+++
this is the body
"###
        ),
        r###"
this is the body
"###
    );
}
