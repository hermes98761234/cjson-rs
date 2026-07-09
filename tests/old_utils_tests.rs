use cjson::parse;
use cjson::to_string_minified;
use cjson::utils::{
    find_pointer_from_object_to, get_pointer, get_pointer_case_sensitive,
    merge_patch, merge_patch_case_sensitive,
    generate_merge_patch, generate_merge_patch_case_sensitive,
    sort_object, sort_object_case_sensitive,
};

fn parse_json(s: &str) -> cjson::Value {
    parse(s).expect("Failed to parse test JSON")
}

// ---- JSON Pointer Tests ----
// Ported from old_utils_tests.c: json_pointer_tests

#[test]
fn json_pointer_tests() {
    let json = r#"{
        "foo": ["bar", "baz"],
        "": 0,
        "a/b": 1,
        "c%d": 2,
        "e^f": 3,
        "g|h": 4,
        "i\\j": 5,
        "k\"l": 6,
        " ": 7,
        "m~n": 8
    }"#;
    let root = parse_json(json);

    // Root pointer
    assert!(get_pointer(&root, "").unwrap().is_object());
    // Simple object key
    assert!(get_pointer(&root, "/foo").unwrap().is_array());
    // Array indices
    assert_eq!(get_pointer(&root, "/foo/0").unwrap().as_str(), Some("bar"));
    assert_eq!(get_pointer(&root, "/foo/1").unwrap().as_str(), Some("baz"));
    // Empty key
    assert_eq!(get_pointer(&root, "/").unwrap().as_f64(), Some(0.0));
    // Escaped slash (a/b -> a~1b)
    assert_eq!(get_pointer(&root, "/a~1b").unwrap().as_f64(), Some(1.0));
    // Special characters
    assert_eq!(get_pointer(&root, "/c%d").unwrap().as_f64(), Some(2.0));
    assert_eq!(get_pointer(&root, "/e^f").unwrap().as_f64(), Some(3.0));
    assert_eq!(get_pointer(&root, "/g|h").unwrap().as_f64(), Some(4.0));
    assert_eq!(get_pointer(&root, "/i\\j").unwrap().as_f64(), Some(5.0));
    assert_eq!(get_pointer(&root, "/k\"l").unwrap().as_f64(), Some(6.0));
    assert_eq!(get_pointer(&root, "/ ").unwrap().as_f64(), Some(7.0));
    // Escaped tilde (m~n -> m~0n)
    assert_eq!(get_pointer(&root, "/m~0n").unwrap().as_f64(), Some(8.0));
}

// ---- Misc/Find Pointer Tests ----
// Ported from old_utils_tests.c: misc_tests

#[test]
fn find_pointer_from_object_to_test() {
    let json = r#"{"numbers": [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]}"#;
    let root = parse_json(json);

    let num6 = get_pointer(&root, "/numbers/6").unwrap();
    assert_eq!(find_pointer_from_object_to(&root, num6), Some("/numbers/6".into()));

    let nums = get_pointer(&root, "/numbers").unwrap();
    assert_eq!(find_pointer_from_object_to(&root, nums), Some("/numbers".into()));

    assert_eq!(find_pointer_from_object_to(&root, &root), Some("".into()));
}

#[test]
fn find_pointer_escaped_tilde_slash() {
    let root = parse_json(r#"{"m~n": "value"}"#);
    let target = get_pointer(&root, "/m~0n").unwrap();
    assert_eq!(find_pointer_from_object_to(&root, target), Some("/m~0n".into()));

    let root2 = parse_json(r#"{"m/n": "value"}"#);
    let target2 = get_pointer(&root2, "/m~1n").unwrap();
    assert_eq!(find_pointer_from_object_to(&root2, target2), Some("/m~1n".into()));
}

// ---- Sort Tests ----
// Ported from old_utils_tests.c: sort_tests

#[test]
fn sort_object_sorts_keys() {
    let random = "QWERTYUIOPASDFGHJKLZXCVBNM";
    let mut map = std::collections::BTreeMap::new();
    for c in random.chars() {
        map.insert(c.to_string(), cjson::Value::Number(cjson::Number::from_i64(1)));
    }
    let mut v = cjson::Value::Object(map);
    sort_object(&mut v);

    let obj = v.as_object().unwrap();
    let mut prev: Option<&str> = None;
    for key in obj.keys() {
        if let Some(p) = prev {
            assert!(key.as_str() >= p, "sort failed");
        }
        prev = Some(key);
    }
}

// ---- Merge Patch Tests ----
// Ported from old_utils_tests.c: merge_tests

const MERGE_TESTS: &[(&str, &str, &str)] = &[
    (r#"{"a":"b"}"#, r#"{"a":"c"}"#, r#"{"a":"c"}"#),
    (r#"{"a":"b"}"#, r#"{"b":"c"}"#, r#"{"a":"b","b":"c"}"#),
    (r#"{"a":"b"}"#, r#"{"a":null}"#, r#"{}"#),
    (r#"{"a":"b","b":"c"}"#, r#"{"a":null}"#, r#"{"b":"c"}"#),
    (r#"{"a":["b"]}"#, r#"{"a":"c"}"#, r#"{"a":"c"}"#),
    (r#"{"a":"c"}"#, r#"{"a":["b"]}"#, r#"{"a":["b"]}"#),
    (r#"{"a":{"b":"c"}}"#, r#"{"a":{"b":"d","c":null}}"#, r#"{"a":{"b":"d"}}"#),
    (r#"{"a":[{"b":"c"}]}"#, r#"{"a":[1]}"#, r#"{"a":[1]}"#),
    (r#"["a","b"]"#, r#"["c","d"]"#, r#"["c","d"]"#),
    (r#"{"a":"b"}"#, r#"["c"]"#, r#"["c"]"#),
    (r#"{"a":"foo"}"#, "null", "null"),
    (r#"{"a":"foo"}"#, r#""bar""#, r#""bar""#),
    (r#"{"e":null}"#, r#"{"a":1}"#, r#"{"a":1,"e":null}"#),
    (r#"[1,2]"#, r#"{"a":"b","c":null}"#, r#"{"a":"b"}"#),
    (r#"{}"#, r#"{"a":{"bb":{"ccc":null}}}"#, r#"{"a":{"bb":{}}}"#),
];

#[test]
fn merge_patch_tests() {
    for (i, (from, patch_str, expected)) in MERGE_TESTS.iter().enumerate() {
        let mut target = parse_json(from);
        let patch = parse_json(patch_str);
        merge_patch(&mut target, &patch);
        let result = to_string_minified(&target);
        assert_eq!(result, *expected, "merge test {} failed: {} + {} != {}", i, from, patch_str, expected);
    }
}

#[test]
fn generate_merge_patch_tests() {
    for (i, (from, _, to)) in MERGE_TESTS.iter().enumerate() {
        let from_val = parse_json(from);
        let to_val = parse_json(to);
        let patch = generate_merge_patch(&from_val, &to_val);
        let mut target = from_val.clone();
        merge_patch(&mut target, &patch);
        let result = to_string_minified(&target);
        assert_eq!(result, *to, "generate merge test {} failed", i);
    }
}
