use cjson::minify;

// Tests ported from minify_tests.c
// Note: Our minify only removes whitespace outside strings (no C-style comment removal)

#[test]
fn minify_should_remove_spaces() {
    let input = "{ \"key\":\ttrue\r\n    }";
    assert_eq!(minify(input), "{\"key\":true}");
}

#[test]
fn minify_should_not_modify_strings() {
    let input = r#""this is a string \" \t bla""#;
    assert_eq!(minify(input), input);
}

#[test]
fn minify_should_minify_json() {
    let input = r#"{
    "glossary": {
        "title": "example glossary",
        "GlossDiv": {
            "title": "S",
            "GlossList": {
                "GlossEntry": {
                    "ID": "SGML",
                    "SortAs": "SGML",
                    "Acronym": "SGML",
                    "GlossDef": {
                        "GlossSeeAlso": ["GML", "XML"]
                    }
                }
            }
        }
    }
}"#;
    let expected = r#"{"glossary":{"title":"example glossary","GlossDiv":{"title":"S","GlossList":{"GlossEntry":{"ID":"SGML","SortAs":"SGML","Acronym":"SGML","GlossDef":{"GlossSeeAlso":["GML","XML"]}}}}}}"#;
    assert_eq!(minify(input), expected);
}

#[test]
fn minify_should_handle_empty_input() {
    assert_eq!(minify(""), "");
}

#[test]
fn minify_should_handle_whitespace_only() {
    assert_eq!(minify("   \t\n\r   "), "");
}

#[test]
fn minify_should_preserve_quoted_whitespace() {
    let input = r#"{"key": "value with spaces"}"#;
    let expected = r#"{"key":"value with spaces"}"#;
    assert_eq!(minify(input), expected);
}

#[test]
fn minify_should_handle_escape_sequences() {
    let input = r#"{"key": "escaped\"quote"}"#;
    let expected = r#"{"key":"escaped\"quote"}"#;
    assert_eq!(minify(input), expected);
}
