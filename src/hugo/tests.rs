#![cfg(test)]

use super::*;

#[test]
fn test_header_and_body() {
    assert_eq!(header_and_body("abc"), Err(GetHeaderAndBodyErr::NoHeader));

    assert_eq!(
        header_and_body(
            r###"
    +++
    title = "My Title"
    +++

    abc
    "###,
        ),
        Ok((
            Header {
                title: "My Title".to_string()
            },
            r###"

    abc
    "###
        ))
    );
    /*
        let (h, b) = header_and_body(
            r###"+++
    title: some title
    date: 2023-02-07
    +++
    this is the body
    "###,
        );
        assert_eq!(h.title, "some title");
        assert_eq!(
            b,
            r###"
    this is the body
    "###
        );
        */
}
