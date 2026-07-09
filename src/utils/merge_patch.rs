use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use crate::Value;

/// Apply a JSON Merge Patch (RFC 7396) to a target value.
///
/// The patch is merged recursively into the target. If the patch is an object,
/// keys with `null` values are removed from the target, keys with other values
/// are merged recursively. If the patch is not an object, it replaces the target.
///
/// This function modifies `target` in place and returns it.
pub fn merge_patch(target: &mut Value, patch: &Value) {
    merge_patch_inner(target, patch, false);
}

/// Case-sensitive variant of `merge_patch`.
pub fn merge_patch_case_sensitive(target: &mut Value, patch: &Value) {
    merge_patch_inner(target, patch, true);
}

/// Generate a JSON Merge Patch (RFC 7396) that would transform `from` into `to`.
///
/// Returns the patch as a `Value`. For scalar or array differences, the patch
/// is simply the `to` value. For object differences, the patch contains only
/// the changed keys.
pub fn generate_merge_patch(from: &Value, to: &Value) -> Value {
    generate_merge_patch_inner(from, to, false)
}

/// Case-sensitive variant of `generate_merge_patch`.
pub fn generate_merge_patch_case_sensitive(from: &Value, to: &Value) -> Value {
    generate_merge_patch_inner(from, to, true)
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn merge_patch_inner(target: &mut Value, patch: &Value, _case_sensitive: bool) {
    match patch {
        Value::Object(patch_map) => {
            // Ensure target is an object too
            if !target.is_object() {
                *target = Value::Object(BTreeMap::new());
            }

            if let Value::Object(ref mut target_map) = target {
                for (key, patch_val) in patch_map.iter() {
                    match patch_val {
                        Value::Null => {
                            // Remove key
                            target_map.remove(key);
                        }
                        _ => {
                            // Recursively merge
                            let entry = target_map.entry(key.clone()).or_insert(Value::Null);
                            merge_patch_inner(entry, patch_val, _case_sensitive);
                        }
                    }
                }
            }
        }
        _ => {
            // Non-object patch replaces the target
            *target = patch.clone();
        }
    }
}

fn generate_merge_patch_inner(from: &Value, to: &Value, _case_sensitive: bool) -> Value {
    if to.is_null() {
        return Value::Null;
    }

    // If either is not an object, the patch is just a copy of `to`
    if !from.is_object() || !to.is_object() {
        return to.clone();
    }

    let from_map = from.as_object().unwrap();
    let to_map = to.as_object().unwrap();

    let mut patch_map = BTreeMap::new();

    // Collect all keys
    let mut all_keys: Vec<&str> = from_map.keys().map(|k| k.as_str()).collect();
    for key in to_map.keys() {
        if !from_map.contains_key(key) {
            all_keys.push(key);
        }
    }
    // Sort for deterministic output
    all_keys.sort();
    all_keys.dedup();

    for key in all_keys {
        let from_val = from_map.get(key);
        let to_val = to_map.get(key);

        match (from_val, to_val) {
            (Some(fv), Some(tv)) => {
                if fv != tv {
                    // Different values - generate sub-patch
                    let sub = generate_merge_patch_inner(fv, tv, _case_sensitive);
                    if sub.is_null() {
                        patch_map.insert(key.to_string(), Value::Null);
                    } else if !is_empty_object(&sub) {
                        patch_map.insert(key.to_string(), sub);
                    } else {
                        patch_map.insert(key.to_string(), tv.clone());
                    }
                }
                // Same value - omitted from patch
            }
            (Some(_), None) => {
                // Key in 'from' but not in 'to' -> remove (null)
                patch_map.insert(key.to_string(), Value::Null);
            }
            (None, Some(tv)) => {
                // Key in 'to' but not in 'from' -> add
                patch_map.insert(key.to_string(), tv.clone());
            }
            (None, None) => unreachable!(),
        }
    }

    if patch_map.is_empty() {
        // No changes
        Value::Object(patch_map)
    } else {
        Value::Object(patch_map)
    }
}

fn is_empty_object(v: &Value) -> bool {
    matches!(v, Value::Object(m) if m.is_empty())
}
