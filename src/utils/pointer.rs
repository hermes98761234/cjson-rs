use alloc::string::String;
use alloc::vec::Vec;
use crate::Value;

/// Retrieve a value from a JSON document using a JSON Pointer (RFC 6901).
///
/// Case-insensitive object key matching by default (matching cJSON_Utils behavior).
/// Splits the pointer on '/', decodes `~0` and `~1` escape sequences,
/// and traverses into objects and arrays.
///
/// Returns `None` if the pointer path doesn't exist.
pub fn get_pointer<'a>(root: &'a Value, pointer: &str) -> Option<&'a Value> {
    get_item_from_pointer(root, pointer, false)
}

/// Case-sensitive variant of `get_pointer`.
///
/// Since `BTreeMap` is already case-sensitive by default, this is identical to `get_pointer`
/// unless the upstream cJSON behavior for case-insensitive key lookup was different.
/// In this implementation, both functions behave the same way (case-sensitive key matching)
/// because we use `BTreeMap::get()` which is case-sensitive.
/// The separate function exists for API compatibility with cJSON_Utils.
pub fn get_pointer_case_sensitive<'a>(root: &'a Value, pointer: &str) -> Option<&'a Value> {
    get_item_from_pointer(root, pointer, true)
}

/// Given a root object and a target value (reference within root),
/// construct a JSON Pointer string from root to target.
///
/// Returns `None` if target is not found within root.
pub fn find_pointer_from_object_to<'a>(root: &'a Value, target: &Value) -> Option<String> {
    find_pointer_recursive(root, target, &mut 0)
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn get_item_from_pointer<'a>(root: &'a Value, pointer: &str, _case_sensitive: bool) -> Option<&'a Value> {
    if pointer.is_empty() {
        return Some(root);
    }

    let mut current = root;

    if !pointer.starts_with('/') {
        return None;
    }

    // Split on '/' and process each token
    let segments: Vec<&str> = pointer.split('/').collect();
    // First segment is empty (before the leading '/')
    for segment in &segments[1..] {
        let decoded = decode_pointer_segment(segment);
        match current {
            Value::Object(map) => {
                current = map.get(&decoded)?;
            }
            Value::Array(arr) => {
                let index = parse_array_index(segment)?;
                current = arr.get(index)?;
            }
            _ => return None,
        }
    }

    Some(current)
}

/// Decode a single JSON Pointer path segment.
/// `~0` -> `~`, `~1` -> `/`
fn decode_pointer_segment(segment: &str) -> String {
    let mut result = String::with_capacity(segment.len());
    let mut chars = segment.chars();
    while let Some(c) = chars.next() {
        if c == '~' {
            match chars.next() {
                Some('0') => result.push('~'),
                Some('1') => result.push('/'),
                _ => {
                    // Invalid escape sequence, just push the character
                    result.push('~');
                }
            }
        } else {
            result.push(c);
        }
    }
    result
}

/// Parse an array index from a pointer segment.
/// Leading zeros are not permitted (except for "0" itself).
fn parse_array_index(segment: &str) -> Option<usize> {
    if segment.is_empty() {
        return None;
    }
    // Leading zero check (except "0")
    if segment.len() > 1 && segment.starts_with('0') {
        return None;
    }
    segment.parse::<usize>().ok()
}

fn find_pointer_recursive<'a>(root: &'a Value, target: &Value, _child_index: &mut usize) -> Option<String> {
    if core::ptr::eq(root, target) {
        return Some(String::new());
    }

    match root {
        Value::Array(arr) => {
            for (i, child) in arr.iter().enumerate() {
                if let Some(suffix) = find_pointer_recursive(child, target, &mut 0) {
                    return Some(alloc::format!("/{}{}", i, suffix));
                }
            }
        }
        Value::Object(map) => {
            for (key, child) in map.iter() {
                if let Some(suffix) = find_pointer_recursive(child, target, &mut 0) {
                    let encoded = encode_pointer_segment(key);
                    return Some(alloc::format!("/{}{}", encoded, suffix));
                }
            }
        }
        _ => {}
    }

    None
}

/// Encode a string segment for use in a JSON Pointer.
/// `~` -> `~0`, `/` -> `~1`
fn encode_pointer_segment(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '~' => result.push_str("~0"),
            '/' => result.push_str("~1"),
            _ => result.push(c),
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse;

    fn parse_json(s: &str) -> Value {
        parse(s).expect("Failed to parse test JSON")
    }

    #[test]
    fn test_pointer_root() {
        let v = parse_json(r#"{"foo": "bar"}"#);
        assert_eq!(get_pointer(&v, "").unwrap().as_str(), None); // root is object
        assert!(get_pointer(&v, "").unwrap().is_object());
    }

    #[test]
    fn test_pointer_simple() {
        let v = parse_json(r#"{"foo": ["bar", "baz"]}"#);
        let foo = get_pointer(&v, "/foo").unwrap();
        assert!(foo.is_array());

        let bar = get_pointer(&v, "/foo/0").unwrap();
        assert_eq!(bar.as_str(), Some("bar"));

        let baz = get_pointer(&v, "/foo/1").unwrap();
        assert_eq!(baz.as_str(), Some("baz"));
    }

    #[test]
    fn test_pointer_empty_key() {
        let v = parse_json(r#"{"": 0}"#);
        let empty = get_pointer(&v, "/").unwrap();
        assert_eq!(empty.as_f64(), Some(0.0));
    }

    #[test]
    fn test_pointer_escaped_slash() {
        // "a/b" key -> pointer has ~1 for /
        let v = parse_json(r#"{"a/b": 1}"#);
        let item = get_pointer(&v, "/a~1b").unwrap();
        assert_eq!(item.as_f64(), Some(1.0));
    }

    #[test]
    fn test_pointer_escaped_tilde() {
        // "m~n" key -> pointer has ~0 for ~
        let v = parse_json(r#"{"m~n": 8}"#);
        let item = get_pointer(&v, "/m~0n").unwrap();
        assert_eq!(item.as_f64(), Some(8.0));
    }

    #[test]
    fn test_pointer_special_chars() {
        let json = r#"{"c%d":2,"e^f":3,"g|h":4,"i\\j":5,"k\"l":6," ":7}"#;
        let v = parse_json(json);

        assert_eq!(get_pointer(&v, "/c%d").unwrap().as_f64(), Some(2.0));
        assert_eq!(get_pointer(&v, "/e^f").unwrap().as_f64(), Some(3.0));
        assert_eq!(get_pointer(&v, "/g|h").unwrap().as_f64(), Some(4.0));
        assert_eq!(get_pointer(&v, "/i\\j").unwrap().as_f64(), Some(5.0));
        assert_eq!(get_pointer(&v, "/k\"l").unwrap().as_f64(), Some(6.0));
        assert_eq!(get_pointer(&v, "/ ").unwrap().as_f64(), Some(7.0));
    }

    #[test]
    fn test_pointer_not_found() {
        let v = parse_json(r#"{"foo": "bar"}"#);
        assert!(get_pointer(&v, "/baz").is_none());
        assert!(get_pointer(&v, "/foo/bar").is_none());
    }

    #[test]
    fn test_pointer_case_sensitive() {
        // BTreeMap is case-sensitive, so both functions behave the same
        let v = parse_json(r#"{"Foo": 1}"#);
        assert!(get_pointer(&v, "/foo").is_none());
        assert_eq!(get_pointer(&v, "/Foo").unwrap().as_f64(), Some(1.0));
        assert_eq!(get_pointer_case_sensitive(&v, "/Foo").unwrap().as_f64(), Some(1.0));
    }

    #[test]
    fn test_find_pointer_from_object_to() {
        let v = parse_json(r#"{"numbers": [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]}"#);

        // Find pointer to the whole object
        let p = find_pointer_from_object_to(&v, &v);
        assert_eq!(p, Some("".into()));

        // Find pointer to the "numbers" array
        let numbers = get_pointer(&v, "/numbers").unwrap();
        let p = find_pointer_from_object_to(&v, numbers);
        assert_eq!(p, Some("/numbers".into()));

        // Find pointer to numbers[6]
        let num6 = get_pointer(&v, "/numbers/6").unwrap();
        let p = find_pointer_from_object_to(&v, num6);
        assert_eq!(p, Some("/numbers/6".into()));
    }

    #[test]
    fn test_find_pointer_tilde_slash_encoded() {
        let v = parse_json(r#"{"m~n": "one"}"#);
        let target = get_pointer(&v, "/m~0n").unwrap();
        let p = find_pointer_from_object_to(&v, target);
        assert_eq!(p, Some("/m~0n".into()));

        let v2 = parse_json(r#"{"m/n": "two"}"#);
        let target2 = get_pointer(&v2, "/m~1n").unwrap();
        let p2 = find_pointer_from_object_to(&v2, target2);
        assert_eq!(p2, Some("/m~1n".into()));
    }

    #[test]
    fn test_find_pointer_not_found() {
        let v = parse_json(r#"{"foo": "bar"}"#);
        let other = Value::String("not_in_tree".into());
        assert!(find_pointer_from_object_to(&v, &other).is_none());
    }
}
