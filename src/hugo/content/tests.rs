#![cfg(test)]

use super::*;

#[test]
fn test_header_and_body() {
    assert!(header_and_body("abc").is_err());

    let result2 = header_and_body(
        r###"
    +++
    title = "My Title"
    +++

    abc
    "###,
    );
    assert!(result2.is_ok());
    assert_eq!(
        result2.unwrap(),
        (
            Header {
                title: Some("My Title".to_string()),
                description: None,
            },
            r###"
    +++
    title = "My Title"
    +++"###,
            r###"

    abc
    "###
        )
    );
}
