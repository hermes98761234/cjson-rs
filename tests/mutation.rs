use cjson::{parse, Value};

// Tests ported from cjson_add.c and misc_tests.c (mutation / add operations)

#[test]
fn add_null_to_object() {
    let mut root = Value::object();
    root.insert("null", Value::Null);
    assert!(root.get("null").unwrap().is_null());
}

#[test]
fn add_true_to_object() {
    let mut root = Value::object();
    root.insert("true", Value::Bool(true));
    assert!(root.get("true").unwrap().is_true());
}

#[test]
fn add_false_to_object() {
    let mut root = Value::object();
    root.insert("false", Value::Bool(false));
    assert!(root.get("false").unwrap().is_false());
}

#[test]
fn add_number_to_object() {
    let mut root = Value::object();
    root.insert("number", Value::number(42));
    let found = root.get("number").unwrap();
    assert!(found.is_number());
    assert!((found.as_f64().unwrap() - 42.0).abs() < 1e-10);
}

#[test]
fn add_string_to_object() {
    let mut root = Value::object();
    root.insert("string", Value::string("Hello World!"));
    let found = root.get("string").unwrap();
    assert!(found.is_string());
    assert_eq!(found.as_str().unwrap(), "Hello World!");
}

#[test]
fn add_object_to_object() {
    let mut root = Value::object();
    root.insert("object", Value::object());
    assert!(root.get("object").unwrap().is_object());
}

#[test]
fn add_array_to_object() {
    let mut root = Value::object();
    root.insert("array", Value::array());
    assert!(root.get("array").unwrap().is_array());
}

// Detach/remove tests

#[test]
fn remove_index_from_array() {
    let mut arr = Value::array();
    arr.push(Value::number(1));
    arr.push(Value::number(2));
    arr.push(Value::number(3));

    // Remove middle
    let removed = arr.remove_index(1);
    assert_eq!(removed, Some(Value::number(2)));
    assert_eq!(arr.len(), 2);
    assert_eq!(arr.get_index(0), Some(&Value::number(1)));
    assert_eq!(arr.get_index(1), Some(&Value::number(3)));

    // Remove first
    let removed = arr.remove_index(0);
    assert_eq!(removed, Some(Value::number(1)));
    assert_eq!(arr.len(), 1);

    // Remove last
    let removed = arr.remove_index(0);
    assert_eq!(removed, Some(Value::number(3)));
    assert_eq!(arr.len(), 0);

    // Out of bounds
    assert_eq!(arr.remove_index(0), None);
}

#[test]
fn remove_index_out_of_bounds() {
    let mut arr = Value::array();
    assert_eq!(arr.remove_index(0), None);
    assert_eq!(arr.remove_index(5), None);
}

#[test]
fn remove_key_from_object() {
    let mut obj = parse(r#"{"a": 1, "b": 2, "c": 3}"#).unwrap();
    let removed = obj.remove_key("b");
    assert_eq!(removed, Some(Value::number(2)));
    assert_eq!(obj.len(), 2);
    assert_eq!(obj.get("a"), Some(&Value::number(1)));
    assert_eq!(obj.get("c"), Some(&Value::number(3)));
    assert!(obj.get("b").is_none());
}

#[test]
fn remove_key_nonexistent() {
    let mut obj = parse(r#"{"a": 1}"#).unwrap();
    assert_eq!(obj.remove_key("b"), None);
    assert_eq!(obj.len(), 1);
}

#[test]
fn remove_key_wrong_type() {
    let mut arr = Value::array();
    assert_eq!(arr.remove_key("x"), None);
}

#[test]
fn insert_in_array_middle() {
    let mut arr = Value::array();
    arr.push(Value::number(1));
    arr.push(Value::number(3));

    arr.insert_in_array(1, Value::number(2));
    assert_eq!(arr.len(), 3);
    assert_eq!(arr.get_index(0), Some(&Value::number(1)));
    assert_eq!(arr.get_index(1), Some(&Value::number(2)));
    assert_eq!(arr.get_index(2), Some(&Value::number(3)));
}

#[test]
fn insert_in_array_beginning() {
    let mut arr = Value::array();
    arr.push(Value::number(2));
    arr.push(Value::number(3));

    arr.insert_in_array(0, Value::number(1));
    assert_eq!(arr.len(), 3);
    assert_eq!(arr.get_index(0), Some(&Value::number(1)));
}

#[test]
fn insert_in_array_end() {
    let mut arr = Value::array();
    arr.push(Value::number(1));

    arr.insert_in_array(1, Value::number(2));
    assert_eq!(arr.len(), 2);
    assert_eq!(arr.get_index(1), Some(&Value::number(2)));
}

#[test]
fn insert_in_array_wrong_type() {
    let mut obj = Value::object();
    // Should not panic
    obj.insert_in_array(0, Value::Null);
    assert!(obj.is_object());
}

#[test]
fn replace_index_in_array() {
    let mut arr = Value::array();
    arr.push(Value::number(1));
    arr.push(Value::number(2));
    arr.push(Value::number(3));

    let old = arr.replace_index(1, Value::number(99));
    assert_eq!(old, Some(Value::number(2)));
    assert_eq!(arr.get_index(1), Some(&Value::number(99)));
}

#[test]
fn replace_index_out_of_bounds() {
    let mut arr = Value::array();
    assert_eq!(arr.replace_index(0, Value::Null), None);

    arr.push(Value::number(1));
    assert_eq!(arr.replace_index(5, Value::Null), None);
}

#[test]
fn replace_key_in_object() {
    let mut obj = parse(r#"{"name": "old", "keep": "me"}"#).unwrap();
    let old = obj.replace_key("name", Value::string("new"));
    assert_eq!(old, Some(Value::string("old")));
    assert_eq!(obj.get("name").and_then(|v| v.as_str()), Some("new"));
    assert_eq!(obj.get("keep").and_then(|v| v.as_str()), Some("me"));
}

#[test]
fn replace_key_new_key() {
    let mut obj = Value::object();
    let old = obj.replace_key("new_key", Value::number(42));
    assert_eq!(old, None);
    assert_eq!(obj.get("new_key").and_then(|v| v.as_f64()), Some(42.0));
}

#[test]
fn replace_key_wrong_type() {
    let mut arr = Value::array();
    assert_eq!(arr.replace_key("x", Value::Null), None);
}

// Tests ported from misc_tests.c

#[test]
fn type_check_functions() {
    assert!(Value::Null.is_null());
    assert!(Value::Bool(true).is_bool());
    assert!(Value::Bool(true).is_true());
    assert!(Value::Bool(false).is_false());
    assert!(Value::number(42).is_number());
    assert!(Value::string("hello").is_string());
    assert!(Value::array().is_array());
    assert!(Value::object().is_object());
}

#[test]
fn get_object_item_case_insensitive() {
    let v = parse(r#"{"one": 1, "Two": 2, "tHree": 3}"#).unwrap();
    // Our `get` is case-sensitive by default (BTreeMap)
    assert!(v.get("one").is_some());
    assert!(v.get("Two").is_some());
    assert!(v.get("tHree").is_some());
    // Case-sensitive: won't find with wrong case
    assert!(v.get("One").is_none());
    assert!(v.get("four").is_none());
}

#[test]
fn get_object_item_should_not_crash_with_array() {
    let arr = parse("[1]").unwrap();
    assert_eq!(arr.get("name"), None);
}

#[test]
fn deeply_nested_json_should_fail() {
    let mut deep = String::with_capacity(2000);
    for _ in 0..1001 {
        deep.push('[');
    }
    deep.push_str("null");
    for _ in 0..1001 {
        deep.push(']');
    }
    assert!(parse(&deep).is_err());
}

#[test]
fn delete_item_from_array_should_not_broken_list() {
    let mut root = parse(r#"{"rd": []}"#).unwrap();
    let rd = root.get_mut("rd").unwrap();

    let item1 = parse(r#"{"a": "123"}"#).unwrap();
    let item2 = parse(r#"{"b": "456"}"#).unwrap();

    rd.push(item1);
    assert_eq!(rd.len(), 1);
    assert_eq!(
        rd.get_index(0)
            .and_then(|v| v.get("a"))
            .and_then(|v| v.as_str()),
        Some("123")
    );

    rd.push(item2);
    assert_eq!(rd.len(), 2);

    // Delete item from array
    let removed = rd.remove_index(0);
    assert!(removed.is_some());
    assert_eq!(rd.len(), 1);
    assert_eq!(
        rd.get_index(0)
            .and_then(|v| v.get("b"))
            .and_then(|v| v.as_str()),
        Some("456")
    );
}
