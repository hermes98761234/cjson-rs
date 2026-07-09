use cjson::utils::{
    add_patch_to_array, apply_patches, apply_patches_case_sensitive,
    find_pointer_from_object_to, generate_merge_patch, generate_merge_patch_case_sensitive,
    generate_patches, generate_patches_case_sensitive,
    get_pointer, get_pointer_case_sensitive,
    merge_patch, merge_patch_case_sensitive,
    sort_object, sort_object_case_sensitive,
};

/// Test that utils functions don't crash with null/invalid inputs.
/// Ported from misc_utils_tests.c
#[test]
fn cjson_utils_functions_shouldnt_crash_with_null_pointers() {
    let item = cjson::Value::string("item");

    // GetPointer with valid root
    assert!(get_pointer(&item, "").is_some());
    assert!(get_pointer(&item, "/nonexistent").is_none());

    // GeneratePatches on identical values
    let patches = generate_patches(&item, &item);
    assert!(patches.is_array());
    assert_eq!(patches.len(), 0);

    // ApplyPatches - empty patch list
    let empty_patches = cjson::Value::Array(vec![]);
    let mut root = cjson::Value::Null;
    assert!(apply_patches(&mut root, &empty_patches).is_ok());

    // ApplyPatches with invalid patch should error
    let invalid_patch = cjson::Value::Number(42.into());
    let mut root = cjson::Value::object();
    assert!(apply_patches(&mut root, &invalid_patch).is_err());

    // AddPatchToArray
    let mut arr = cjson::Value::Array(vec![]);
    add_patch_to_array(&mut arr, "add", "/foo", &cjson::Value::Null);
    assert_eq!(arr.len(), 1);

    // MergePatch with non-object patch should replace
    let mut target = cjson::Value::string("hello");
    let patch = cjson::Value::string("world");
    merge_patch(&mut target, &patch);
    assert_eq!(target.as_str(), Some("world"));

    // SortObject on null should not crash
    let mut null_val = cjson::Value::Null;
    sort_object(&mut null_val);
    assert!(null_val.is_null());

    // FindPointerFromObjectTo with unrelated values
    let root = cjson::Value::object();
    assert!(find_pointer_from_object_to(&root, &cjson::Value::Null).is_none());
}
