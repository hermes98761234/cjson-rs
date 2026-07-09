use crate::Value;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

/// Error type for JSON Patch operations.
#[derive(Clone, Debug, PartialEq)]
pub struct PatchError {
    pub kind: PatchErrorKind,
}

impl fmt::Display for PatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            PatchErrorKind::InvalidPatch => write!(f, "invalid patch"),
            PatchErrorKind::UnknownOperation => write!(f, "unknown operation"),
            PatchErrorKind::FromNotFound => write!(f, "from pointer not found"),
            PatchErrorKind::PathNotFound => write!(f, "path not found"),
            PatchErrorKind::MissingValue => write!(f, "missing value"),
            PatchErrorKind::TestFailed => write!(f, "test failed"),
            PatchErrorKind::IndexOutOfBounds => write!(f, "index out of bounds"),
            PatchErrorKind::NotAnArray => write!(f, "target is not an array"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for PatchError {}

#[derive(Clone, Debug, PartialEq)]
pub enum PatchErrorKind {
    InvalidPatch,
    UnknownOperation,
    FromNotFound,
    PathNotFound,
    MissingValue,
    TestFailed,
    IndexOutOfBounds,
    NotAnArray,
}

/// Apply JSON Patch operations (RFC 6902) to a JSON value.
pub fn apply_patches(root: &mut Value, patches: &Value) -> Result<(), PatchError> {
    apply_patches_inner(root, patches, false)
}

/// Case-sensitive variant of `apply_patches`.
pub fn apply_patches_case_sensitive(root: &mut Value, patches: &Value) -> Result<(), PatchError> {
    apply_patches_inner(root, patches, true)
}

/// Generate a list of JSON Patch operations (RFC 6902) that would transform
/// `from` into `to`.
pub fn generate_patches(from: &Value, to: &Value) -> Value {
    generate_patches_inner(from, to, false)
}

/// Case-sensitive variant of `generate_patches`.
pub fn generate_patches_case_sensitive(from: &Value, to: &Value) -> Value {
    generate_patches_inner(from, to, true)
}

/// Add a patch object to a patch array.
pub fn add_patch_to_array(array: &mut Value, op: &str, path: &str, value: &Value) {
    let mut patch = BTreeMap::new();
    patch.insert("op".into(), Value::String(op.into()));
    patch.insert("path".into(), Value::String(path.into()));
    patch.insert("value".into(), value.clone());
    if let Value::Array(ref mut arr) = array {
        arr.push(Value::Object(patch));
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn apply_patches_inner(
    root: &mut Value,
    patches: &Value,
    case_sensitive: bool,
) -> Result<(), PatchError> {
    let arr = match patches {
        Value::Array(a) => a,
        _ => {
            return Err(PatchError {
                kind: PatchErrorKind::InvalidPatch,
            })
        }
    };
    for patch in arr {
        apply_single_patch(root, patch, case_sensitive)?;
    }
    Ok(())
}

fn apply_single_patch(
    root: &mut Value,
    patch: &Value,
    case_sensitive: bool,
) -> Result<(), PatchError> {
    let obj = match patch {
        Value::Object(m) => m,
        _ => {
            return Err(PatchError {
                kind: PatchErrorKind::InvalidPatch,
            })
        }
    };
    let op = get_string_field(obj, "op").ok_or(PatchError {
        kind: PatchErrorKind::InvalidPatch,
    })?;
    let path = get_string_field(obj, "path").ok_or(PatchError {
        kind: PatchErrorKind::InvalidPatch,
    })?;
    match op {
        "add" => apply_add(root, path, obj, case_sensitive),
        "remove" => apply_remove(root, path, case_sensitive),
        "replace" => apply_replace(root, path, obj, case_sensitive),
        "move" => apply_move(root, path, obj, case_sensitive),
        "copy" => apply_copy(root, path, obj, case_sensitive),
        "test" => apply_test(root, path, obj, case_sensitive),
        _ => Err(PatchError {
            kind: PatchErrorKind::UnknownOperation,
        }),
    }
}

fn get_string_field<'a>(obj: &'a BTreeMap<String, Value>, key: &str) -> Option<&'a str> {
    obj.get(key).and_then(|v| v.as_str())
}

fn get_value_field<'a>(obj: &'a BTreeMap<String, Value>, key: &str) -> Option<&'a Value> {
    obj.get(key)
}

fn decode_pointer_segment(segment: &str) -> String {
    let mut result = String::with_capacity(segment.len());
    let mut chars = segment.chars();
    while let Some(c) = chars.next() {
        if c == '~' {
            match chars.next() {
                Some('0') => result.push('~'),
                Some('1') => result.push('/'),
                _ => result.push('~'),
            }
        } else {
            result.push(c);
        }
    }
    result
}

fn parse_array_index(segment: &str) -> Option<usize> {
    if segment.is_empty() || (segment.len() > 1 && segment.starts_with('0')) {
        return None;
    }
    segment.parse::<usize>().ok()
}

// ---- Safe recursive mutable pointer traversal ----

/// Traverse into a JSON value mutably following a pointer path.
/// Returns a mutable reference to the value at the end of the path.
fn traverse_mut<'a>(root: &'a mut Value, pointer: &str) -> Option<&'a mut Value> {
    if pointer.is_empty() {
        return Some(root);
    }
    if !pointer.starts_with('/') {
        return None;
    }
    let segments: Vec<&str> = pointer.split('/').collect();
    // segments[0] is empty, skip it
    traverse_mut_recursive(root, &segments[1..])
}

fn traverse_mut_recursive<'a>(current: &'a mut Value, segments: &[&str]) -> Option<&'a mut Value> {
    if segments.is_empty() {
        return Some(current);
    }
    let (first, rest) = (segments[0], &segments[1..]);
    let decoded = decode_pointer_segment(first);
    match current {
        Value::Object(ref mut map) => {
            let child = map.get_mut(&decoded)?;
            traverse_mut_recursive(child, rest)
        }
        Value::Array(ref mut arr) => {
            let index = parse_array_index(first)?;
            let child = arr.get_mut(index)?;
            traverse_mut_recursive(child, rest)
        }
        _ => None,
    }
}

/// Split a JSON Pointer path into parent path and last child token.
/// For "/foo/1", returns ("/foo", "1").
/// For "/foo", returns ("", "foo").
/// For "", returns ("", None).
fn split_pointer(path: &str) -> (&str, Option<&str>) {
    if path.is_empty() {
        return ("", None);
    }
    match path.rfind('/') {
        None => ("", Some(path)),
        Some(0) => ("", Some(&path[1..])),
        Some(pos) => (&path[..pos], Some(&path[pos + 1..])),
    }
}

/// Get an immutable reference via pointer.
fn get_pointer<'a>(root: &'a Value, pointer: &str) -> Option<&'a Value> {
    if pointer.is_empty() {
        return Some(root);
    }
    if !pointer.starts_with('/') {
        return None;
    }
    let segments: Vec<&str> = pointer.split('/').collect();
    let mut current = root;
    for segment in &segments[1..] {
        let decoded = decode_pointer_segment(segment);
        match current {
            Value::Object(map) => current = map.get(&decoded)?,
            Value::Array(arr) => {
                let index = parse_array_index(segment)?;
                current = arr.get(index)?;
            }
            _ => return None,
        }
    }
    Some(current)
}

// ---- Core operation helpers ----

/// For operations that modify a child of a container (add/remove to array or object),
/// we get a mutable ref to the parent container via traverse_mut(parent_path),
/// then apply the change using the child key.
fn modify_child<F>(root: &mut Value, path: &str, f: F) -> Result<(), PatchError>
where
    F: FnOnce(&mut Value, &str) -> Result<(), PatchError>,
{
    if path.is_empty() {
        return Err(PatchError {
            kind: PatchErrorKind::PathNotFound,
        });
    }
    let (parent_path, child) = split_pointer(path);
    let child = child.ok_or(PatchError {
        kind: PatchErrorKind::PathNotFound,
    })?;
    let decoded_child = decode_pointer_segment(child);
    let parent = traverse_mut(root, parent_path).ok_or(PatchError {
        kind: PatchErrorKind::PathNotFound,
    })?;
    f(parent, &decoded_child)
}

// ---- Patch operations ----

fn apply_add(
    root: &mut Value,
    path: &str,
    obj: &BTreeMap<String, Value>,
    _case_sensitive: bool,
) -> Result<(), PatchError> {
    let value = get_value_field(obj, "value").ok_or(PatchError {
        kind: PatchErrorKind::MissingValue,
    })?;
    let cloned = value.clone();

    if path.is_empty() {
        *root = cloned;
        return Ok(());
    }

    // Handle "-" for array append
    let (parent_path, child) = split_pointer(path);
    let child = child.ok_or(PatchError {
        kind: PatchErrorKind::PathNotFound,
    })?;

    if child == "-" {
        let parent = traverse_mut(root, parent_path).ok_or(PatchError {
            kind: PatchErrorKind::PathNotFound,
        })?;
        match parent {
            Value::Array(ref mut arr) => {
                arr.push(cloned);
                Ok(())
            }
            _ => Err(PatchError {
                kind: PatchErrorKind::NotAnArray,
            }),
        }
    } else {
        let decoded_child = decode_pointer_segment(child);
        let parent = traverse_mut(root, parent_path).ok_or(PatchError {
            kind: PatchErrorKind::PathNotFound,
        })?;
        match parent {
            Value::Array(ref mut arr) => {
                let index: usize = decoded_child.parse().map_err(|_| PatchError {
                    kind: PatchErrorKind::IndexOutOfBounds,
                })?;
                if index <= arr.len() {
                    arr.insert(index, cloned);
                    Ok(())
                } else {
                    Err(PatchError {
                        kind: PatchErrorKind::IndexOutOfBounds,
                    })
                }
            }
            Value::Object(ref mut map) => {
                map.insert(decoded_child, cloned);
                Ok(())
            }
            _ => Err(PatchError {
                kind: PatchErrorKind::PathNotFound,
            }),
        }
    }
}

fn apply_remove(root: &mut Value, path: &str, _case_sensitive: bool) -> Result<(), PatchError> {
    if path.is_empty() {
        *root = Value::Null;
        return Ok(());
    }

    modify_child(root, path, |parent, child| match parent {
        Value::Array(ref mut arr) => {
            let index: usize = child.parse().map_err(|_| PatchError {
                kind: PatchErrorKind::IndexOutOfBounds,
            })?;
            if index < arr.len() {
                arr.remove(index);
                Ok(())
            } else {
                Err(PatchError {
                    kind: PatchErrorKind::IndexOutOfBounds,
                })
            }
        }
        Value::Object(ref mut map) => {
            map.remove(&*child).ok_or(PatchError {
                kind: PatchErrorKind::PathNotFound,
            })?;
            Ok(())
        }
        _ => Err(PatchError {
            kind: PatchErrorKind::PathNotFound,
        }),
    })
}

fn apply_replace(
    root: &mut Value,
    path: &str,
    obj: &BTreeMap<String, Value>,
    _case_sensitive: bool,
) -> Result<(), PatchError> {
    let value = get_value_field(obj, "value").ok_or(PatchError {
        kind: PatchErrorKind::MissingValue,
    })?;
    let cloned = value.clone();

    if path.is_empty() {
        *root = cloned;
        return Ok(());
    }

    // For replace, we can just traverse to the target and replace it directly
    let target = traverse_mut(root, path).ok_or(PatchError {
        kind: PatchErrorKind::PathNotFound,
    })?;
    *target = cloned;
    Ok(())
}

fn apply_move(
    root: &mut Value,
    path: &str,
    obj: &BTreeMap<String, Value>,
    _case_sensitive: bool,
) -> Result<(), PatchError> {
    let from = get_string_field(obj, "from").ok_or(PatchError {
        kind: PatchErrorKind::InvalidPatch,
    })?;
    let value = detach_path(root, from).ok_or(PatchError {
        kind: PatchErrorKind::FromNotFound,
    })?;

    if path.is_empty() {
        *root = value;
        return Ok(());
    }

    let (parent_path, child) = split_pointer(path);
    let child = child.ok_or(PatchError {
        kind: PatchErrorKind::PathNotFound,
    })?;
    let decoded_child = decode_pointer_segment(child);

    if child == "-" {
        let parent = traverse_mut(root, parent_path).ok_or(PatchError {
            kind: PatchErrorKind::PathNotFound,
        })?;
        match parent {
            Value::Array(ref mut arr) => {
                arr.push(value);
                Ok(())
            }
            _ => Err(PatchError {
                kind: PatchErrorKind::NotAnArray,
            }),
        }
    } else {
        let parent = traverse_mut(root, parent_path).ok_or(PatchError {
            kind: PatchErrorKind::PathNotFound,
        })?;
        match parent {
            Value::Array(ref mut arr) => {
                let index: usize = decoded_child.parse().map_err(|_| PatchError {
                    kind: PatchErrorKind::IndexOutOfBounds,
                })?;
                if index <= arr.len() {
                    arr.insert(index, value);
                    Ok(())
                } else {
                    Err(PatchError {
                        kind: PatchErrorKind::IndexOutOfBounds,
                    })
                }
            }
            Value::Object(ref mut map) => {
                map.insert(decoded_child, value);
                Ok(())
            }
            _ => Err(PatchError {
                kind: PatchErrorKind::PathNotFound,
            }),
        }
    }
}

fn apply_copy(
    root: &mut Value,
    path: &str,
    obj: &BTreeMap<String, Value>,
    _case_sensitive: bool,
) -> Result<(), PatchError> {
    let from = get_string_field(obj, "from").ok_or(PatchError {
        kind: PatchErrorKind::InvalidPatch,
    })?;
    let source = get_pointer(root, from).ok_or(PatchError {
        kind: PatchErrorKind::FromNotFound,
    })?;
    let cloned = source.clone();

    if path.is_empty() {
        *root = cloned;
        return Ok(());
    }

    let (parent_path, child) = split_pointer(path);
    let child = child.ok_or(PatchError {
        kind: PatchErrorKind::PathNotFound,
    })?;
    let decoded_child = decode_pointer_segment(child);

    if child == "-" {
        let parent = traverse_mut(root, parent_path).ok_or(PatchError {
            kind: PatchErrorKind::PathNotFound,
        })?;
        match parent {
            Value::Array(ref mut arr) => {
                arr.push(cloned);
                Ok(())
            }
            _ => Err(PatchError {
                kind: PatchErrorKind::NotAnArray,
            }),
        }
    } else {
        let parent = traverse_mut(root, parent_path).ok_or(PatchError {
            kind: PatchErrorKind::PathNotFound,
        })?;
        match parent {
            Value::Array(ref mut arr) => {
                let index: usize = decoded_child.parse().map_err(|_| PatchError {
                    kind: PatchErrorKind::IndexOutOfBounds,
                })?;
                if index <= arr.len() {
                    arr.insert(index, cloned);
                    Ok(())
                } else {
                    Err(PatchError {
                        kind: PatchErrorKind::IndexOutOfBounds,
                    })
                }
            }
            Value::Object(ref mut map) => {
                map.insert(decoded_child, cloned);
                Ok(())
            }
            _ => Err(PatchError {
                kind: PatchErrorKind::PathNotFound,
            }),
        }
    }
}

fn apply_test(
    root: &mut Value,
    path: &str,
    obj: &BTreeMap<String, Value>,
    case_sensitive: bool,
) -> Result<(), PatchError> {
    let value = get_value_field(obj, "value").ok_or(PatchError {
        kind: PatchErrorKind::MissingValue,
    })?;
    let current = get_pointer(root, path).ok_or(PatchError {
        kind: PatchErrorKind::PathNotFound,
    })?;
    if !crate::compare(current, value, case_sensitive) {
        return Err(PatchError {
            kind: PatchErrorKind::TestFailed,
        });
    }
    Ok(())
}

/// Detach (remove and return) a value at a given JSON Pointer path.
fn detach_path(root: &mut Value, path: &str) -> Option<Value> {
    if path.is_empty() {
        let old = core::mem::replace(root, Value::Null);
        return Some(old);
    }
    let (parent_path, child) = split_pointer(path);
    let child = child?;
    let decoded_child = decode_pointer_segment(child);
    let parent = traverse_mut(root, parent_path)?;
    match parent {
        Value::Array(ref mut arr) => {
            let index: usize = decoded_child.parse().ok()?;
            if index < arr.len() {
                Some(arr.remove(index))
            } else {
                None
            }
        }
        Value::Object(ref mut map) => map.remove(&decoded_child),
        _ => None,
    }
}

// ---- Generate patches ----

fn generate_patches_inner(from: &Value, to: &Value, case_sensitive: bool) -> Value {
    let mut patches = Vec::new();
    if core::mem::discriminant(from) != core::mem::discriminant(to)
        || from.is_null()
        || to.is_null()
    {
        if !crate::compare(from, to, case_sensitive) {
            patches.push(create_patch_entry("replace", "", to));
        }
        return Value::Array(patches);
    }
    match (from, to) {
        (Value::Array(from_arr), Value::Array(to_arr)) => {
            generate_array_patches(&mut patches, from_arr, to_arr, case_sensitive);
        }
        (Value::Object(from_map), Value::Object(to_map)) => {
            generate_object_patches(&mut patches, from_map, to_map, case_sensitive);
        }
        (a, b) => {
            if !crate::compare(a, b, case_sensitive) {
                patches.push(create_patch_entry("replace", "", to));
            }
        }
    }
    Value::Array(patches)
}

fn create_patch_entry(op: &str, path: &str, value: &Value) -> Value {
    let mut obj = BTreeMap::new();
    obj.insert("op".into(), Value::String(op.into()));
    obj.insert("path".into(), Value::String(path.into()));
    obj.insert("value".into(), value.clone());
    Value::Object(obj)
}

fn generate_array_patches(
    patches: &mut Vec<Value>,
    from: &[Value],
    to: &[Value],
    case_sensitive: bool,
) {
    let min_len = from.len().min(to.len());
    for i in 0..min_len {
        let path = alloc::format!("/{}", i);
        let sub_patches = generate_patches_inner(&from[i], &to[i], case_sensitive);
        if let Value::Array(arr) = sub_patches {
            for sub_patch in arr {
                if let Value::Object(ref m) = sub_patch {
                    let sub_path = get_string_field(m, "path").unwrap_or("");
                    let sub_op = get_string_field(m, "op").unwrap_or("replace");
                    let sub_val = get_value_field(m, "value");
                    let full_path = if sub_path.is_empty() {
                        path.clone()
                    } else {
                        alloc::format!("{}{}", path, sub_path)
                    };
                    let mut new_patch = BTreeMap::new();
                    new_patch.insert("op".into(), Value::String(sub_op.into()));
                    new_patch.insert("path".into(), Value::String(full_path));
                    if let Some(v) = sub_val {
                        new_patch.insert("value".into(), v.clone());
                    }
                    patches.push(Value::Object(new_patch));
                }
            }
        }
    }
    for _ in min_len..from.len() {
        patches.push(create_patch_entry(
            "remove",
            &alloc::format!("/{}", min_len),
            &Value::Null,
        ));
    }
    for i in min_len..to.len() {
        patches.push(create_patch_entry("add", "/-", &to[i]));
    }
}

fn generate_object_patches(
    patches: &mut Vec<Value>,
    from: &BTreeMap<String, Value>,
    to: &BTreeMap<String, Value>,
    _case_sensitive: bool,
) {
    let mut all_keys: Vec<&str> = from.keys().map(|k| k.as_str()).collect();
    for key in to.keys() {
        if !from.contains_key(key) {
            all_keys.push(key);
        }
    }
    all_keys.sort();

    for key in all_keys {
        match (from.get(key), to.get(key)) {
            (Some(from_val), Some(to_val)) => {
                if !crate::compare(from_val, to_val, _case_sensitive) {
                    let sub_patches = generate_patches_inner(from_val, to_val, _case_sensitive);
                    if let Value::Array(arr) = sub_patches {
                        if arr.is_empty() {
                            continue;
                        }
                        for sub_patch in arr {
                            if let Value::Object(ref m) = sub_patch {
                                let sub_path = get_string_field(m, "path").unwrap_or("");
                                let sub_op = get_string_field(m, "op").unwrap_or("replace");
                                let sub_val = get_value_field(m, "value");
                                let full_path = if sub_path.is_empty() {
                                    alloc::format!("/{}", encode_pointer_segment(key))
                                } else {
                                    alloc::format!("/{}{}", encode_pointer_segment(key), sub_path)
                                };
                                let mut new_patch = BTreeMap::new();
                                new_patch.insert("op".into(), Value::String(sub_op.into()));
                                new_patch.insert("path".into(), Value::String(full_path));
                                if let Some(v) = sub_val {
                                    new_patch.insert("value".into(), v.clone());
                                }
                                patches.push(Value::Object(new_patch));
                            }
                        }
                    }
                }
            }
            (Some(_), None) => {
                patches.push(create_patch_entry(
                    "remove",
                    &alloc::format!("/{}", encode_pointer_segment(key)),
                    &Value::Null,
                ));
            }
            (None, Some(to_val)) => {
                patches.push(create_patch_entry(
                    "add",
                    &alloc::format!("/{}", encode_pointer_segment(key)),
                    to_val,
                ));
            }
            (None, None) => unreachable!(),
        }
    }
}

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
