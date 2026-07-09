use cjson::{parse, to_string, to_string_minified};

/// Test both minified and pretty-printed object output.
fn assert_print_object(expected_formatted: &str, expected_minified: &str, input: &str) {
    let parsed = parse(input).expect("Failed to parse object");

    // BTreeMap sorts keys, so minified will have sorted keys
    let minified = to_string_minified(&parsed);
    assert_eq!(
        minified, expected_minified,
        "Minified object is not correct"
    );

    // Pretty-printed
    let formatted = to_string(&parsed);
    assert_eq!(
        formatted, expected_formatted,
        "Formatted object is not correct"
    );
}

#[test]
fn print_object_should_print_empty_objects() {
    // Empty objects print compact even in pretty mode
    assert_print_object("{}", "{}", "{}");
}

#[test]
fn print_object_should_print_objects_with_one_element() {
    assert_print_object("{\n  \"one\": 1\n}", r#"{"one":1}"#, r#"{"one":1}"#);
    assert_print_object(
        "{\n  \"hello\": \"world!\"\n}",
        r#"{"hello":"world!"}"#,
        r#"{"hello":"world!"}"#,
    );
    assert_print_object(
        "{\n  \"array\": []\n}",
        r#"{"array":[]}"#,
        r#"{"array":[]}"#,
    );
    assert_print_object(
        "{\n  \"null\": null\n}",
        r#"{"null":null}"#,
        r#"{"null":null}"#,
    );
}

#[test]
fn print_object_should_print_objects_with_multiple_elements() {
    // BTreeMap sorts keys alphabetically, so minified output sorts them
    assert_print_object(
        "{\n  \"one\": 1,\n  \"three\": 3,\n  \"two\": 2\n}",
        r#"{"one":1,"three":3,"two":2}"#,
        r#"{"one":1,"two":2,"three":3}"#,
    );
    // Complex object — BTreeMap sorts all keys
    assert_print_object(
        "{\n  \"FALSE\": false,\n  \"NULL\": null,\n  \"TRUE\": true,\n  \"array\": [],\n  \"object\": {},\n  \"one\": 1,\n  \"world\": \"hello\"\n}",
        r#"{"FALSE":false,"NULL":null,"TRUE":true,"array":[],"object":{},"one":1,"world":"hello"}"#,
        r#"{"one":1,"NULL":null,"TRUE":true,"FALSE":false,"array":[],"world":"hello","object":{}}"#,
    );
}
