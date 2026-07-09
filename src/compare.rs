use crate::Value;
use alloc::collections::BTreeMap;
use alloc::string::String;

/// Compare two JSON values for equality.
///
/// When `case_sensitive` is `true`, object keys must match exactly.
/// When `case_sensitive` is `false`, object keys are compared case-insensitively
/// (i.e. `"Key"` and `"key"` match the same key in both objects).
pub fn compare(a: &Value, b: &Value, case_sensitive: bool) -> bool {
    match (a, b) {
        (Value::Null, Value::Null) => true,
        (Value::Bool(a), Value::Bool(b)) => a == b,
        (Value::Number(a), Value::Number(b)) => a.0 == b.0,
        (Value::String(a), Value::String(b)) => a == b,
        (Value::Raw(a), Value::Raw(b)) => a == b,
        (Value::Array(a), Value::Array(b)) => {
            if a.len() != b.len() {
                return false;
            }
            a.iter()
                .zip(b.iter())
                .all(|(x, y)| compare(x, y, case_sensitive))
        }
        (Value::Object(a), Value::Object(b)) => compare_objects(a, b, case_sensitive),
        _ => false,
    }
}

fn compare_objects(
    a: &BTreeMap<String, Value>,
    b: &BTreeMap<String, Value>,
    case_sensitive: bool,
) -> bool {
    if a.len() != b.len() {
        return false;
    }

    if case_sensitive {
        // Exact key matching — iterate keys from 'a' and look up in 'b'
        for (key, val_a) in a.iter() {
            match b.get(key) {
                Some(val_b) => {
                    if !compare(val_a, val_b, case_sensitive) {
                        return false;
                    }
                }
                None => return false,
            }
        }
        true
    } else {
        // Case-insensitive key matching
        // Build a map of lowercase key -> original key for 'b'
        let b_lower: BTreeMap<String, String> =
            b.keys().map(|k| (k.to_lowercase(), k.clone())).collect();

        for (key_a, val_a) in a.iter() {
            let lower_a = key_a.to_lowercase();
            match b_lower.get(&lower_a) {
                Some(orig_key_b) => {
                    let val_b = &b[orig_key_b];
                    if !compare(val_a, val_b, case_sensitive) {
                        return false;
                    }
                }
                None => return false,
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse;

    fn compare_from_string(json_a: &str, json_b: &str, case_sensitive: bool) -> bool {
        let a = parse(json_a).expect("Failed to parse a");
        let b = parse(json_b).expect("Failed to parse b");
        compare(&a, &b, case_sensitive)
    }

    #[test]
    fn compare_numbers_equal() {
        assert!(compare_from_string("1", "1", true));
        assert!(compare_from_string("1", "1", false));
        assert!(compare_from_string("0.0001", "0.0001", true));
        assert!(compare_from_string("0.0001", "0.0001", false));
        assert!(compare_from_string("1E100", "10E99", false));
    }

    #[test]
    fn compare_numbers_not_equal() {
        assert!(!compare_from_string("0.5E-100", "0.5E-101", false));
        assert!(!compare_from_string("1", "2", true));
        assert!(!compare_from_string("1", "2", false));
    }

    #[test]
    fn compare_booleans() {
        assert!(compare_from_string("true", "true", true));
        assert!(compare_from_string("true", "true", false));
        assert!(compare_from_string("false", "false", true));
        assert!(compare_from_string("false", "false", false));
        assert!(!compare_from_string("true", "false", true));
        assert!(!compare_from_string("true", "false", false));
        assert!(!compare_from_string("false", "true", true));
        assert!(!compare_from_string("false", "true", false));
    }

    #[test]
    fn compare_null() {
        assert!(compare_from_string("null", "null", true));
        assert!(compare_from_string("null", "null", false));
        assert!(!compare_from_string("null", "true", true));
        assert!(!compare_from_string("null", "true", false));
    }

    #[test]
    fn compare_strings() {
        assert!(compare_from_string(r#""abcdefg""#, r#""abcdefg""#, true));
        assert!(compare_from_string(r#""abcdefg""#, r#""abcdefg""#, false));
        assert!(!compare_from_string(r#""ABCDEFG""#, r#""abcdefg""#, true));
        // Case-insensitive string values still compare literally
        assert!(!compare_from_string(r#""ABCDEFG""#, r#""abcdefg""#, false));
    }

    #[test]
    fn compare_arrays() {
        assert!(compare_from_string("[]", "[]", true));
        assert!(compare_from_string("[]", "[]", false));
        assert!(compare_from_string(
            r#"[false,true,null,42,"string",[],{}]"#,
            r#"[false, true, null, 42, "string", [], {}]"#,
            true,
        ));
        assert!(compare_from_string(
            r#"[false,true,null,42,"string",[],{}]"#,
            r#"[false, true, null, 42, "string", [], {}]"#,
            false,
        ));
        assert!(compare_from_string("[[[1], 2]]", "[[[1], 2]]", true));
        assert!(compare_from_string("[[[1], 2]]", "[[[1], 2]]", false));
        assert!(!compare_from_string(
            "[true,null,42,\"string\",[],{}]",
            "[false, true, null, 42, \"string\", [], {}]",
            true
        ));
        assert!(!compare_from_string(
            "[true,null,42,\"string\",[],{}]",
            "[false, true, null, 42, \"string\", [], {}]",
            false
        ));
        // Different length
        assert!(!compare_from_string("[1,2,3]", "[1,2]", true));
        assert!(!compare_from_string("[1,2,3]", "[1,2]", false));
    }

    #[test]
    fn compare_objects() {
        assert!(compare_from_string("{}", "{}", true));
        assert!(compare_from_string("{}", "{}", false));

        // Same objects, different key order (case-sensitive)
        assert!(compare_from_string(
            r#"{"false": false, "true": true, "null": null, "number": 42, "string": "string", "array": [], "object": {}}"#,
            r#"{"true": true, "false": false, "null": null, "number": 42, "string": "string", "array": [], "object": {}}"#,
            true,
        ));

        // Case-sensitive: different case keys should NOT match
        assert!(!compare_from_string(
            r#"{"False": false, "true": true, "null": null, "number": 42, "string": "string", "array": [], "object": {}}"#,
            r#"{"true": true, "false": false, "null": null, "number": 42, "string": "string", "array": [], "object": {}}"#,
            true,
        ));

        // Case-insensitive: different case keys SHOULD match
        assert!(compare_from_string(
            r#"{"False": false, "true": true, "null": null, "number": 42, "string": "string", "array": [], "object": {}}"#,
            r#"{"true": true, "false": false, "null": null, "number": 42, "string": "string", "array": [], "object": {}}"#,
            false,
        ));

        // Subset test
        assert!(!compare_from_string(
            r#"{"one": 1, "two": 2}"#,
            r#"{"one": 1, "two": 2, "three": 3}"#,
            true,
        ));
        assert!(!compare_from_string(
            r#"{"one": 1, "two": 2}"#,
            r#"{"one": 1, "two": 2, "three": 3}"#,
            false,
        ));
    }

    #[test]
    fn compare_different_types() {
        assert!(!compare_from_string("null", "true", true));
        assert!(!compare_from_string("\"hello\"", "42", true));
        assert!(!compare_from_string("[]", "{}", true));
    }

    #[test]
    fn compare_raw() {
        let raw = r#""[true, false]""#;
        assert!(compare_from_string(raw, raw, true));
        assert!(compare_from_string(raw, raw, false));
    }
}
