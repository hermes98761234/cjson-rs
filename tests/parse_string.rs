use cjson::parse;

#[test]
fn parse_string_empty() {
    assert_eq!(parse(r#""""#).unwrap().as_str(), Some(""));
}

#[test]
fn parse_string_printable() {
    let input = "\" !\\\"#$%&'()*+,-./\\/0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\\\]^_'abcdefghijklmnopqrstuvwxyz{|}~\"";
    let expected = " !\"#$%&'()*+,-.//0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_'abcdefghijklmnopqrstuvwxyz{|}~";
    assert_eq!(parse(input).unwrap().as_str(), Some(expected));
}

#[test]
fn parse_string_escapes() {
    let input = r#""\"\\\/\b\f\n\r\t\u20AC\u732b""#;
    assert_eq!(
        parse(input).unwrap().as_str(),
        Some("\"\\/\u{8}\u{c}\n\r\t\u{20ac}\u{732b}")
    );
}

#[test]
fn parse_string_control_chars_direct() {
    let input = "\"\u{8}\u{c}\n\r\t\"";
    assert_eq!(parse(input).unwrap().as_str(), Some("\u{8}\u{c}\n\r\t"));
}

#[test]
fn parse_string_utf16_surrogate_pair() {
    // \uD83D\uDC31 -> U+1F431 (cat emoji)
    let input = r#""\uD83D\uDC31""#;
    assert_eq!(parse(input).unwrap().as_str(), Some("\u{1F431}"));
}

#[test]
fn parse_string_not_a_string() {
    // "this\" is not a string\"" is not valid JSON — string must start with "
    assert!(parse(r#""this" is not a string""#).is_err());
}

#[test]
fn parse_string_invalid_backslash_1() {
    assert!(parse(r#""Abcdef\123""#).is_err());
}

#[test]
fn parse_string_invalid_backslash_2() {
    assert!(parse(r#""Abcdef\e23""#).is_err());
}

#[test]
fn parse_bug_94() {
    // cJSON bug 94: string with many alternating backslashes and special chars
    // Verified against serde_json for correct expected value
    let input = r#""~!@\\#$%^&*()\\\\-\\+{}[]:\\;\\\"\\\\<\\\\>?/.,DC=ad,DC=com""#;
    let expected = serde_json::from_str::<String>(input).unwrap();
    let v = parse(input).unwrap();
    assert_eq!(v.as_str(), Some(expected.as_str()));
}

#[test]
fn parse_string_overflow_with_closing_backslash() {
    let input = "\"000000000000000000\\\"";
    assert!(parse(input).is_err());
}
