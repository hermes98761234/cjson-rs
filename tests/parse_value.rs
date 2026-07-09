use cjson::{parse, Value};
use std::collections::BTreeMap;

#[test]
fn parse_null() {
    assert_eq!(parse("null").unwrap(), Value::Null);
}

#[test]
fn parse_true() {
    assert_eq!(parse("true").unwrap(), Value::Bool(true));
}

#[test]
fn parse_false() {
    assert_eq!(parse("false").unwrap(), Value::Bool(false));
}

#[test]
fn parse_number_simple() {
    let v = parse("1.5").unwrap();
    assert!(v.is_number());
    assert!((v.as_f64().unwrap() - 1.5).abs() < 1e-10);
}

#[test]
fn parse_string_empty() {
    assert_eq!(parse(r#""""#).unwrap(), Value::String("".into()));
}

#[test]
fn parse_string_hello() {
    assert_eq!(parse(r#""hello""#).unwrap(), Value::String("hello".into()));
}

#[test]
fn parse_empty_array() {
    assert_eq!(parse("[]").unwrap(), Value::Array(vec![]));
}

#[test]
fn parse_empty_object() {
    assert_eq!(parse("{}").unwrap(), Value::Object(BTreeMap::new()));
}

#[test]
fn parse_array_numbers() {
    let v = parse("[1,2,3]").unwrap();
    let arr = v.as_array().unwrap();
    assert_eq!(arr.len(), 3);
    assert_eq!(arr[0], Value::Number(1.into()));
    assert_eq!(arr[1], Value::Number(2.into()));
    assert_eq!(arr[2], Value::Number(3.into()));
}

#[test]
fn parse_object_basic() {
    let v = parse(r#"{"a":1,"b":2}"#).unwrap();
    assert_eq!(v.get("a"), Some(&Value::Number(1.into())));
    assert_eq!(v.get("b"), Some(&Value::Number(2.into())));
}

#[test]
fn parse_nested() {
    let v = parse(r#"{"arr":[1,{"x":2}]}"#).unwrap();
    assert_eq!(
        v.get("arr")
            .and_then(|a| a.get_index(1))
            .and_then(|o| o.get("x")),
        Some(&Value::Number(2.into()))
    );
}

#[test]
fn parse_error_trailing_garbage() {
    assert!(parse("null x").is_err());
}

#[test]
fn parse_whitespace_around() {
    assert_eq!(parse("  null  ").unwrap(), Value::Null);
    assert_eq!(parse("  true  ").unwrap(), Value::Bool(true));
    assert_eq!(parse("  42  ").unwrap(), Value::Number(42.into()));
}
