use cjson::{parse, to_string, to_string_minified};

/// Test both minified and pretty-printed array output.
fn assert_print_array(expected_formatted: &str, input: &str) {
    let parsed = parse(input).expect("Failed to parse array");

    // Minified should match the input exactly
    let minified = to_string_minified(&parsed);
    assert_eq!(minified, input, "Minified array is not correct");

    // Pretty-printed
    let formatted = to_string(&parsed);
    assert_eq!(formatted, expected_formatted, "Formatted array is not correct");
}

#[test]
fn print_array_should_print_empty_arrays() {
    assert_print_array("[]", "[]");
}

#[test]
fn print_array_should_print_arrays_with_one_element() {
    assert_print_array(
        "[\n  1\n]",
        "[1]",
    );
    assert_print_array(
        "[\n  \"hello!\"\n]",
        r#"["hello!"]"#,
    );
    assert_print_array(
        "[\n  []\n]",
        "[[]]",
    );
    assert_print_array(
        "[\n  null\n]",
        "[null]",
    );
}

#[test]
fn print_array_should_print_arrays_with_multiple_elements() {
    assert_print_array(
        "[\n  1,\n  2,\n  3\n]",
        "[1,2,3]",
    );
    assert_print_array(
        "[\n  1,\n  null,\n  true,\n  false,\n  [],\n  \"hello\",\n  {}\n]",
        r#"[1,null,true,false,[],"hello",{}]"#,
    );
}
