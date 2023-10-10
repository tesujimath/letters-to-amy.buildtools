#![cfg(test)]

use super::*;

#[test]
fn test_header_and_body() {
    assert!(header_and_body("abc").is_err());

    let result2 = header_and_body(
        r###"
    +++
    title = "My Title"
    date = "2023-08-30T06:25:00+12:00"
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
                date: Some("2023-08-30T06:25:00+12:00".to_string())
            },
            r###"
    +++
    title = "My Title"
    date = "2023-08-30T06:25:00+12:00"
    +++"###,
            r###"

    abc
    "###
        )
    );
}
