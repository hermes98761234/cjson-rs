use cjson::parse;
use cjson::to_string_minified;
use cjson::utils::{
    apply_patches, apply_patches_case_sensitive,
    generate_patches, generate_patches_case_sensitive,
};

fn load_test_data(filename: &str) -> cjson::Value {
    let path = std::path::Path::new("/tmp/cJSON/tests/json-patch-tests").join(filename);
    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("Failed to read test file: {:?}", path));
    parse(&content).unwrap_or_else(|e| panic!("Failed to parse test file {}: {}", filename, e))
}

fn test_apply_patch(test: &cjson::Value) -> bool {
    let doc = test.get("doc").expect("No 'doc' in test");
    let patch = test.get("patch").expect("No 'patch' in test");

    if let Some(disabled) = test.get("disabled") {
        if disabled.as_bool() == Some(true) {
            return true;
        }
    }

    let mut object = doc.clone();
    let has_error = test.get("error").is_some();

    if has_error {
        if apply_patches_case_sensitive(&mut object, patch).is_ok() {
            eprintln!("  FAILED: expected error but got success");
            return false;
        }
        return true;
    }

    if let Err(e) = apply_patches_case_sensitive(&mut object, patch) {
        eprintln!("  FAILED: apply error: {}", e);
        return false;
    }

    if let Some(expected) = test.get("expected") {
        if !cjson::compare(&object, expected, true) {
            let got = to_string_minified(&object);
            let exp = to_string_minified(expected);
            eprintln!("  FAILED: got '{}', expected '{}'", got, exp);
            return false;
        }
    }
    true
}

fn test_generate_patch(test: &cjson::Value) -> bool {
    let doc = test.get("doc").expect("No 'doc' in test");

    if let Some(disabled) = test.get("disabled") {
        if disabled.as_bool() == Some(true) {
            return true;
        }
    }

    let expected = match test.get("expected") {
        Some(e) => e,
        None => return true,
    };

    if test.get("error").is_some() {
        return true;
    }

    let mut object = doc.clone();
    let patch = generate_patches_case_sensitive(doc, expected);
    assert!(patch.is_array(), "Failed to generate patches");

    if let Err(e) = apply_patches_case_sensitive(&mut object, &patch) {
        eprintln!("  FAILED to apply generated patch: {}", e);
        return false;
    }

    if !cjson::compare(&object, expected, true) {
        let got = to_string_minified(&object);
        let exp = to_string_minified(expected);
        eprintln!("  Generated patch FAILED: got '{}', expected '{}'", got, exp);
        return false;
    }
    true
}

fn run_test_file(filename: &str) {
    let tests = load_test_data(filename);
    assert!(tests.is_array(), "Test data is not an array: {}", filename);

    let mut failed = false;
    for test in tests.as_array().unwrap().iter() {
        if !test_apply_patch(test) {
            failed = true;
        }
        if !test_generate_patch(test) {
            failed = true;
        }
    }
    assert!(!failed, "Some tests failed in {}", filename);
}

#[test]
fn json_patch_tests_tests() {
    run_test_file("tests.json");
}

#[test]
fn json_patch_spec_tests() {
    run_test_file("spec_tests.json");
}

#[test]
fn json_patch_cjson_utils_tests() {
    run_test_file("cjson-utils-tests.json");
}
