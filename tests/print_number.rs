use cjson::{Number, to_string_minified, Value};

#[test]
fn print_number_should_print_zero() {
    let v = Value::Number(Number::from_i64(0));
    assert_eq!(to_string_minified(&v), "0");
}

#[test]
fn print_number_should_print_negative_integers() {
    assert_eq!(to_string_minified(&Value::Number(Number::from_i64(-1))), "-1");
    assert_eq!(to_string_minified(&Value::Number(Number::from_i64(-32768))), "-32768");
    assert_eq!(to_string_minified(&Value::Number(Number::from_i64(-2147483648))), "-2147483648");
}

#[test]
fn print_number_should_print_positive_integers() {
    assert_eq!(to_string_minified(&Value::Number(Number::from_i64(1))), "1");
    assert_eq!(to_string_minified(&Value::Number(Number::from_i64(32767))), "32767");
    assert_eq!(to_string_minified(&Value::Number(Number::from_i64(2147483647))), "2147483647");
}

#[test]
fn print_number_should_print_float_values() {
    // Parse and re-print to verify round-trip of various number representations
    let cases = ["0", "1", "-1", "3.14", "1.5e2", "1e-09", "1.23e+129", "1.23e-126", "0.123", "-0.0123"];
    for input in cases {
        let parsed = cjson::parse(input).expect("should parse");
        let printed = to_string_minified(&parsed);
        // The output should be valid JSON and parse back to the same numeric value
        let reparsed = cjson::parse(&printed).expect("round-trip should parse");
        assert!(reparsed.is_number(), "round-trip should produce number");
        // Values should be equivalent as f64
        let a = parsed.as_f64().unwrap();
        let b = reparsed.as_f64().unwrap();
        let diff = (a - b).abs();
        let rel = diff / a.abs().max(1.0);
        assert!(diff < 1e-10 || rel < 1e-10, "round-trip {input} -> {printed}: values differ {a} vs {b}");
    }
}

#[test]
fn print_number_should_print_non_number() {
    // NaN and Infinity are rejected by Number::from_f64
    assert!(Number::from_f64(f64::NAN).is_none());
    assert!(Number::from_f64(f64::INFINITY).is_none());
    assert!(Number::from_f64(f64::NEG_INFINITY).is_none());
}
