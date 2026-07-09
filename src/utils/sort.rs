use alloc::string::String;
use alloc::vec::Vec;
use crate::Value;

/// Recursively sort the keys of all objects in a JSON value.
pub fn sort_object(value: &mut Value) {
    sort_object_inner(value, false);
}

/// Case-sensitive variant of `sort_object`.
pub fn sort_object_case_sensitive(value: &mut Value) {
    sort_object_inner(value, true);
}

fn sort_object_inner(value: &mut Value, _case_sensitive: bool) {
    match value {
        Value::Object(ref mut map) => {
            // Take ownership of the map contents to rebuild it sorted
            let entries: Vec<(String, Value)> = core::mem::take(map).into_iter().collect();
            for (key, mut val) in entries {
                sort_object_inner(&mut val, _case_sensitive);
                map.insert(key, val);
            }
        }
        Value::Array(ref mut arr) => {
            for item in arr.iter_mut() {
                sort_object_inner(item, _case_sensitive);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse;

    #[test]
    fn test_sort_empty_object() {
        let mut v = Value::Object(alloc::collections::BTreeMap::new());
        sort_object(&mut v);
        assert!(v.is_object());
    }

    #[test]
    fn test_sort_object_keys() {
        let json = r#"{"z": 1, "a": 2, "m": 3}"#;
        let mut v = parse(json).unwrap();
        sort_object(&mut v);
        let keys: Vec<&str> = v.as_object().unwrap().keys().map(|k| k.as_str()).collect();
        assert_eq!(keys, vec!["a", "m", "z"]);
    }

    #[test]
    fn test_sort_nested() {
        let json = r#"{"z": {"b": 2, "a": 1}, "a": {"d": 4, "c": 3}}"#;
        let mut v = parse(json).unwrap();
        sort_object(&mut v);
        let outer_keys: Vec<&str> = v.as_object().unwrap().keys().map(|k| k.as_str()).collect();
        assert_eq!(outer_keys, vec!["a", "z"]);
        let inner_keys_a: Vec<&str> = v.as_object().unwrap()
            .get("a").unwrap().as_object().unwrap()
            .keys().map(|k| k.as_str()).collect();
        assert_eq!(inner_keys_a, vec!["c", "d"]);
        let inner_keys_z: Vec<&str> = v.as_object().unwrap()
            .get("z").unwrap().as_object().unwrap()
            .keys().map(|k| k.as_str()).collect();
        assert_eq!(inner_keys_z, vec!["a", "b"]);
    }
}
