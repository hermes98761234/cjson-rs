# cjson-rs: Idiomatic Rust Rewrite of cJSON

**Date:** 2026-07-09
**License:** MIT (same as upstream)
**Upstream:** https://github.com/DaveGamble/cJSON

## Overview

Rewrite `cJSON` — the ultralightweight ANSI C JSON library — as an idiomatic Rust crate with no `unsafe` code, automatic memory management via ownership, serde compatibility, and a `utils` module for JSON Pointer (RFC 6901), JSON Patch (RFC 6902), and JSON Merge Patch (RFC 7396).

## Decisions Made

All design decisions were made with the recommended option, bypassing interactive questions. Rejected alternatives are recorded under each section.

---

## 1. Data Model: `Value` Enum

**Chosen:** Enum-based tree type (like `serde_json::Value`).

```rust
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Number(Number),
    String(String),
    Array(Vec<Value>),
    Object(BTreeMap<String, Value>),
}
```

**Rejected:**
- **Doubly-linked cJSON struct mirror** — self-referential `next`/`prev`/`child` pointers require `unsafe` in Rust; defeats the point.
- **Arena-allocated tree** — faster but adds complexity; can be an optimization layer later.
- **Zero-copy/Cow parser** — more efficient for large inputs but significantly more complex; defer to v2.

**`Number` type:** Wraps `f64` with constructor `from_f64` (returns `Result` on NaN/Inf) and `from_int(i64)`. Serializes without trailing `.0` for integer values. Provides `as_f64()`, `as_i64()`, `as_u64()`, `is_integer()` methods.

**Why BTreeMap for Object:** Deterministic key ordering (matches cJSON behavior where `SortObject` is supported). Users who want insertion-order can use `serde_json` crate instead.

---

## 2. Parsing

**Chosen:** Recursive descent parser, public `parse` and `parse_with_options` functions.

```rust
pub fn parse(input: &str) -> Result<Value, Error>;
pub fn parse_with_options(input: &str, opts: ParseOptions) -> Result<Value, Error>;
pub fn from_reader<R: Read>(input: R) -> Result<Value, Error>;
```

**ParseOptions:**
- `max_nesting: usize` — default 1000, matches cJSON's `CJSON_NESTING_LIMIT`
- `require_null_terminated: bool` — matches cJSON's `ParseWithOpts` behavior

**Error type:**

```rust
pub struct Error {
    pub kind: ErrorKind,
    pub position: usize,
    pub line: usize,
    pub column: usize,
}
pub enum ErrorKind {
    UnexpectedToken { expected: String, found: char },
    UnexpectedEndOfInput,
    InvalidNumber,
    InvalidStringEscape,
    InvalidUnicodeSurrogate,
    ExceededNestingLimit,
    TrailingComma,
    TrailingGarbage,
    InvalidUtf8,
}
```

**Error behavior:** First parse error halts and returns `Err` with position info. No `GetErrorPtr` global (the position field replaces it).

**UTF-8 handling:** All strings are validated as UTF-8 during parsing. Invalid UTF-8 sequences produce `ErrorKind::InvalidUtf8`.

**Large numbers:** Parsed as `f64` by default. The `Number` type stores the original `f64` value. Users who want big-integer precision should use `serde_json` with a custom visitor.

**cJSON.parse compatibility:** The parser accepts the same JSON dialect cJSON accepts: no trailing commas in objects/arrays, no comments, no single quotes, no NaN/Inf literals.

---

## 3. Printing

**Chosen:** `Display`-based printing with formatted and minified variants.

```rust
pub fn to_string(&Value) -> String;            // pretty-printed, 2-space indent
pub fn to_string_minified(&Value) -> String;   // no whitespace
pub fn to_writer<W: Write>(&Value, w: W) -> Result<()>;

// Configurable printing
pub fn to_string_with_options(&Value, opts: PrintOptions) -> String;
pub struct PrintOptions {
    pub indent: usize,       // default 2, use 0 for minified
    pub sort_keys: bool,     // recursive key sorting
}
```

**Rejected:**
- `cJSON_PrintPreallocated` — Rust manages allocation; no pre-allocated buffer API needed.
- `cJSON_PrintBuffered` — same reason; Rust handles buffering via `Write`.

**`Display` impl:** Prints formatted (pretty) JSON.

**cJSON printf compatibility:** Same format: `"key": value`, newlines between elements, 2-space indent, no trailing comma.

---

## 4. Construction API

**Chosen:** Convenience methods on `Value` and free functions matching cJSON's `cJSON_Create*` API.

```rust
// Value constructors
impl Value {
    pub fn null() -> Value;
    pub fn bool(b: bool) -> Value;
    pub fn number(n: impl Into<Number>) -> Value;
    pub fn string(s: impl Into<String>) -> Value;
    pub fn array() -> Value;   // creates Array(Vec::new())
    pub fn object() -> Value;  // creates Object(BTreeMap::new())
    pub fn raw(s: &str) -> Value; // creates a String containing raw JSON (parsed during print)
}

// Builder methods
impl Value {
    // Append to array
    pub fn push(&mut self, v: Value);

    // Insert into object
    pub fn insert(&mut self, key: impl Into<String>, v: Value);

    // Reference creation (analogous to cJSON's Create*Reference)
    pub fn from_ref(v: &Value) -> Value;  // Clone, not reference-counted
}

// Free functions (cJSON API compatibility layer)
pub fn create_null() -> Value;
pub fn create_true() -> Value;
pub fn create_false() -> Value;
pub fn create_bool(b: bool) -> Value;
pub fn create_number(n: f64) -> Value;
pub fn create_string(s: &str) -> Value;
pub fn create_array() -> Value;
pub fn create_object() -> Value;
pub fn create_int_array(items: &[i32]) -> Value;
pub fn create_float_array(items: &[f32]) -> Value;
pub fn create_double_array(items: &[f64]) -> Value;
pub fn create_string_array(items: &[&str]) -> Value;
```

**Rejected:** cJSON's `CreateStringReference` / `CreateObjectReference` / `CreateArrayReference` — these avoid copying in C for performance. Rust's `Clone` is explicit; if users want efficiency they should construct values directly rather than using reference semantics with lifetime tracking.

**`cJSON_AddItemToObjectCS`:** cJSON had a const-string optimization. In Rust, `String` owns its data; no equivalent needed.

---

## 5. Accessor API

```rust
impl Value {
    // Type checking
    pub fn is_null(&self) -> bool;
    pub fn is_bool(&self) -> bool;
    pub fn is_number(&self) -> bool;
    pub fn is_string(&self) -> bool;
    pub fn is_array(&self) -> bool;
    pub fn is_object(&self) -> bool;
    pub fn is_raw(&self) -> bool;

    // Value extraction
    pub fn as_bool(&self) -> Option<bool>;
    pub fn as_number(&self) -> Option<&Number>;
    pub fn as_f64(&self) -> Option<f64>;
    pub fn as_i64(&self) -> Option<i64>;
    pub fn as_str(&self) -> Option<&str>;

    // Array access
    pub fn get(&self, index: usize) -> Option<&Value>;
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Value>;
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
    pub fn as_array(&self) -> Option<&[Value]>;
    pub fn as_array_mut(&mut self) -> Option<&mut Vec<Value>>;

    // Object access
    pub fn get(&self, key: &str) -> Option<&Value>;
    pub fn get_mut(&mut self, key: &str) -> Option<&mut Value>;
    pub fn get_case_sensitive(&self, key: &str) -> Option<&Value>;
    pub fn has_key(&self, key: &str) -> bool;
    pub fn keys(&self) -> Option<impl Iterator<Item = &String>>;
    pub fn as_object(&self) -> Option<&BTreeMap<String, Value>>;
    pub fn as_object_mut(&mut self) -> Option<&mut BTreeMap<String, Value>>;

    // Mutation
    pub fn remove(&mut self, index: usize) -> Option<Value>;
    pub fn remove(&mut self, key: &str) -> Option<Value>;
    pub fn insert_in_array(&mut self, index: usize, value: Value) -> Result<()>;
    pub fn replace(&mut self, index: usize, value: Value) -> Option<Value>;
    pub fn replace(&mut self, key: &str, value: Value) -> Option<Value>;
}

// Iteration
impl<'a> IntoIterator for &'a Value { /* yields &Value over array elements */ }
impl<'a> IntoIterator for &'a mut Value { /* yields &mut Value over array elements */ }

// Object entry iteration
pub struct ObjectIter<'a> { /* yields (&'a str, &'a Value) */ }
```

**Case sensitivity:** cJSON's default API is case-insensitive for object keys. Our default is **case-sensitive** (Rust convention). Provide `get_case_sensitive` as an explicit alternative. The `utils` module provides case-sensitive variants matching cJSON's `*CaseSensitive` functions.

---

## 6. Utility Functions

```rust
// Duplication
impl Clone for Value;  // Already #[derive(Clone)]

// Compare
impl PartialEq for Value;  // Already #[derive(PartialEq)], case-sensitive by default
pub fn compare(a: &Value, b: &Value, case_sensitive: bool) -> bool;

// Minify
pub fn minify(input: &str) -> String;  // Removes whitespace from JSON string
```

---

## 7. Utils Module (Feature: `utils`)

All under `cjson::utils`:

### JSON Pointer (RFC 6901)

```rust
pub fn get_pointer<'a>(root: &'a Value, pointer: &str) -> Option<&'a Value>;
pub fn get_pointer_case_sensitive<'a>(root: &'a Value, pointer: &str) -> Option<&'a Value>;
```

### JSON Patch (RFC 6902)

```rust
pub fn apply_patches(root: &mut Value, patches: &Value) -> Result<()>;
pub fn apply_patches_case_sensitive(root: &mut Value, patches: &Value) -> Result<()>;
pub fn generate_patches(from: &Value, to: &Value) -> Value;
pub fn generate_patches_case_sensitive(from: &Value, to: &Value) -> Value;
pub fn add_patch_to_array(array: &mut Value, op: &str, path: &str, value: &Value);
```

### JSON Merge Patch (RFC 7396)

```rust
pub fn merge_patch(target: &mut Value, patch: &Value);
pub fn merge_patch_case_sensitive(target: &mut Value, patch: &Value);
pub fn generate_merge_patch(from: &Value, to: &Value) -> Value;
pub fn generate_merge_patch_case_sensitive(from: &Value, to: &Value) -> Value;
```

### Sorting

```rust
pub fn sort_object(value: &mut Value);
pub fn sort_object_case_sensitive(value: &mut Value);
```

### Find Pointer

```rust
pub fn find_pointer_from_object_to<'a>(root: &'a Value, target: &Value) -> Option<String>;
```

---

## 8. serde Feature

**Chosen:** Optional `serde` feature (default off) implementing `Serialize` and `Deserialize` for `Value`.

```rust
#[cfg(feature = "serde")]
impl Serialize for Value { ... }
#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for Value { ... }
```

**Why optional:** Users who only need JSON parsing don't need the serde dependency. Users who want serde interop enable it. This matches cJSON's philosophy of keeping things minimal with composable extras (just like cJSON_Utils is optional).

---

## 9. Feature Flags

```toml
[features]
default = ["std"]
std = []                    # Provides Read/Write-based APIs, errors with std::error::Error impl
utils = ["std"]             # JSON Pointer/Patch/MergePatch (requires std)
serde = ["dep:serde", "dep:serde_json"];  # Serialize/Deserialize for Value
```

**no_std support:** Core parsing/printing works without `std` (requires `alloc` for `BTreeMap`, `Vec`, `String`). The `std` feature adds `Error: std::error::Error`, `from_reader`, and `to_writer`.

---

## 10. Testing Strategy

**Unit tests:** Port every test from cJSON's test suite (~20 test files, ~7700 lines of C):
- `parse_value`, `parse_object`, `parse_array`, `parse_string`, `parse_number`, `parse_examples`, `parse_hex4`, `parse_with_opts`
- `print_value`, `print_object`, `print_array`, `print_string`, `print_number`
- `cjson_add`, `compare_tests`, `minify_tests`, `misc_tests`
- `readme_examples` (all examples from cJSON README)
- `old_utils_tests`, `misc_utils_tests`, `json_patch_tests` (utils module)

**Property-based tests (proptest):**
- Round-trip: `parse(to_string(v))` should produce an equivalent value
- Minify round-trip: `minify(to_string(v))` should be parseable and equivalent
- Arbitrary `Value` generation

**Fuzz targets:** `cargo-fuzz` targets for the parser with structured inputs.

---

## 11. Project Structure

```
cjson-rs/
├── Cargo.toml
├── LICENSE                     # MIT (same as cJSON)
├── README.md
├── build.rs                    # (optional, if codegen needed)
├── src/
│   ├── lib.rs                  # Public API re-exports
│   ├── value.rs                # Value enum, Number type
│   ├── parse.rs                # Recursive descent parser
│   ├── print.rs                # JSON printer
│   ├── error.rs                # Error type
│   ├── raw.rs                  # Raw JSON value type
│   ├── minify.rs               # Minify implementation
│   ├── serde_impl.rs           # #[cfg(feature = "serde")]
│   ├── utils/                  # #[cfg(feature = "utils")]
│   │   ├── mod.rs
│   │   ├── pointer.rs          # JSON Pointer
│   │   ├── patch.rs            # JSON Patch
│   │   ├── merge_patch.rs      # JSON Merge Patch
│   │   └── sort.rs             # Sorting
│   └── fuzz/                   # Fuzz targets
│       └── fuzz_targets/
│           └── parse.rs
├── tests/                      # Integration tests ported from cJSON
│   ├── parse_value.rs
│   ├── parse_object.rs
│   ├── parse_array.rs
│   ├── parse_string.rs
│   ├── parse_number.rs
│   ├── parse_examples.rs
│   ├── parse_with_opts.rs
│   ├── print_value.rs
│   ├── print_object.rs
│   ├── print_array.rs
│   ├── print_string.rs
│   ├── print_number.rs
│   ├── cjson_add.rs
│   ├── compare_tests.rs
│   ├── minify_tests.rs
│   ├── misc_tests.rs
│   ├── readme_examples.rs
│   ├── utils/
│   │   ├── old_utils_tests.rs
│   │   ├── misc_utils_tests.rs
│   │   └── json_patch_tests.rs
│   └── proptest/
│       └── roundtrip.rs
└── .github/
    └── workflows/
        ├── ci.yml              # fmt, lint, test, clippy
        └── release.yml         # tag-driven release
```

---

## 12. cJSON API Compatibility

We aim for **logical equivalence**, not a line-for-line port:

| C API | Rust Equivalent |
|-------|----------------|
| `cJSON_Parse` | `cjson::parse` |
| `cJSON_Print` | `cjson::to_string` |
| `cJSON_PrintUnformatted` | `cjson::to_string_minified` |
| `cJSON_Delete` | Drop automatically |
| `cJSON_CreateObject` | `Value::object()` |
| `cJSON_AddItemToArray` | `value.push(item)` |
| `cJSON_GetObjectItem` | `value.get("key")` |
| `cJSON_GetErrorPtr` | `Error.position` field |
| `cJSON_Duplicate` | `value.clone()` |
| `cJSON_Compare` | `cjson::compare` or `==` |
| `cJSON_Minify` | `cjson::minify` |

---

## 13. Out of Scope (v1)

- Streaming parser (SAX-style)
- Pretty-print with configurable indent depth (handled in v1 PrintOptions already)
- JSON Schema validation
- SAX/streaming parser
- DOM mutation events
- no_std support without alloc (currently requires alloc)