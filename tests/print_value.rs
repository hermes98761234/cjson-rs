use cjson::{parse, to_string_minified};

fn assert_print_value(input: &str) {
    let parsed = parse(input).expect("Failed to parse value");
    let printed = to_string_minified(&parsed);
    assert_eq!(printed, input, "Printed value is not as expected for input: {input}");
}

#[test]
fn print_value_should_print_null() {
    assert_print_value("null");
}

#[test]
fn print_value_should_print_true() {
    assert_print_value("true");
}

#[test]
fn print_value_should_print_false() {
    assert_print_value("false");
}

#[test]
fn print_value_should_print_number() {
    assert_print_value("1.5");
}

#[test]
fn print_value_should_print_string_empty() {
    assert_print_value(r#""""#);
}

#[test]
fn print_value_should_print_string_hello() {
    assert_print_value(r#""hello""#);
}

#[test]
fn print_value_should_print_empty_array() {
    assert_print_value("[]");
}

#[test]
fn print_value_should_print_empty_object() {
    assert_print_value("{}");
}
