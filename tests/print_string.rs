use cjson::{to_string_minified, Value};

fn assert_print_string(expected: &str, input: &str) {
    let v = Value::String(input.into());
    let printed = to_string_minified(&v);
    assert_eq!(
        printed, expected,
        "Printed string doesn't match expected for input: {input:?}"
    );
}

#[test]
fn print_string_should_print_empty_strings() {
    assert_print_string(r#""""#, "");
}

#[test]
fn print_string_should_print_ascii() {
    // Build all printable ASCII characters (1..0x7F)
    let mut ascii = String::new();
    for i in 1..0x7F {
        ascii.push(char::from_u32(i).unwrap());
    }

    // Build expected output programmatically
    let mut expected = String::from("\"");
    for i in 1u32..0x7Fu32 {
        match i {
            0x01..=0x07 | 0x0B | 0x0E..=0x1F => {
                expected.push_str(&format!("\\u{:04x}", i));
            }
            0x08 => expected.push_str("\\b"),
            0x09 => expected.push_str("\\t"),
            0x0A => expected.push_str("\\n"),
            0x0C => expected.push_str("\\f"),
            0x0D => expected.push_str("\\r"),
            0x22 => expected.push_str("\\\""),
            0x5C => expected.push_str("\\\\"),
            c => expected.push(char::from_u32(c).unwrap()),
        }
    }
    expected.push('"');

    let v = Value::String(ascii);
    let printed = to_string_minified(&v);
    assert_eq!(printed, expected, "ASCII table printed incorrectly");
}

#[test]
fn print_string_should_print_utf8() {
    let v = Value::String("ü猫慕".into());
    let printed = to_string_minified(&v);
    assert_eq!(printed, "\"ü猫慕\"", "UTF-8 string printed incorrectly");
}
