# cjson-rs

[![CI](https://github.com/hermes98761234/cjson-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/hermes98761234/cjson-rs/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/cjson.svg)](https://crates.io/crates/cjson)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Idiomatic Rust rewrite of [cJSON](https://github.com/DaveGamble/cJSON), the
ultralightweight C JSON parser. Same philosophy — simple, fast, minimal
dependencies — but written for Rust's type system and ecosystem.

- **Zero `unsafe`.** Every line is safe Rust.
- **UTF-8 validated.** All strings are checked on parse.
- **Nesting limit.** Configurable (default 1000), prevents runaway recursion.
- **`no_std` compatible.** Core types work without the standard library (`alloc` only).
- **Feature-gated extras.** Enable `serde` for Serialize/Deserialize, or `utils` for
  JSON Pointer, Patch, and MergePatch.

## Quick Start

```toml
[dependencies]
cjson = "0.1"
```

To enable optional features:

```toml
[dependencies]
cjson = { version = "0.1", features = ["serde", "utils"] }
```

## Parsing

```rust
use cjson::{parse, Value};

let value = parse(r#"{"name": "cjson-rs", "version": 1}"#)?;

// That's it. `value` is a `cjson::Value` enum.
```

For fine-grained control:

```rust
use cjson::{parse_with_options, ParseOptions};

let opts = ParseOptions {
    max_nesting: 500,
    require_null_terminated: false, // allow trailing whitespace without error
};
let value = parse_with_options(r#"{"loose": true}  "#, opts)?;
```

## Printing

```rust
use cjson::{parse, to_string, to_string_minified};

let value = parse(r#"{"hello": "world", "nested": [1, 2, 3]}"#)?;

// Pretty-print with 2-space indent
println!("{}", to_string(&value));

// Minified (no whitespace)
println!("{}", to_string_minified(&value));
```

Custom options:

```rust
use cjson::{to_string_with_options, PrintOptions};

let pretty = to_string_with_options(&value, PrintOptions {
    indent: 4,
    sort_keys: true,
});
```

## Building JSON

Construct values programmatically:

```rust
use cjson::{Value, Number};

// Objects
let mut obj = Value::object();
obj.insert("name", Value::string("cjson-rs"));
obj.insert("stars", Value::number(42));
obj.insert("active", Value::bool(true));
obj.insert("data", Value::null());

// Arrays
let mut arr = Value::array();
arr.push(Value::number(1));
arr.push(Value::number(2));
arr.push(Value::number(3));

// Nested
let mut root = Value::object();
root.insert("items", arr);
root.insert("metadata", obj);

let json = to_string(&root);
```

## Accessing Data

```rust
let value = parse(r#"{
    "name": "cjson-rs",
    "count": 42,
    "tags": ["json", "rust"],
    "meta": { "version": 1 }
}"#)?;

// Object access
assert_eq!(value.get("name").and_then(|v| v.as_str()), Some("cjson-rs"));
assert_eq!(value.get("count").and_then(|v| v.as_i64()), Some(42));
assert!(value.has_key("meta"));

// Array access
let tags = value.get("tags").and_then(|v| v.as_array()).unwrap();
assert_eq!(tags[0].as_str(), Some("json"));
assert_eq!(value.get_index(1), None); // only works on arrays

// Type checks
assert!(value.get("name").unwrap().is_string());
assert!(value.get("count").unwrap().is_number());

// Mutation
let mut arr = Value::array();
arr.push(Value::string("a"));
arr.push(Value::string("b"));
arr.push(Value::string("c"));

arr.remove_index(1);                            // removes "b"
arr.insert_in_array(1, Value::string("x"));     // [ "a", "x", "c" ]
arr.replace_index(0, Value::string("first"));   // [ "first", "x", "c" ]
```

### Accessor methods

| Method | Returns | Works on |
|---|---|---|
| `value.as_bool()` | `Option<bool>` | Bool |
| `value.as_str()` | `Option<&str>` | String |
| `value.as_f64()` | `Option<f64>` | Number |
| `value.as_i64()` | `Option<i64>` | Number (if integer) |
| `value.as_array()` / `as_array_mut()` | `Option<&[Value]>` / `Option<&mut Vec<Value>>` | Array |
| `value.as_object()` / `as_object_mut()` | `Option<&BTreeMap>` / `Option<&mut BTreeMap>` | Object |
| `value.get(key)` / `value.get_mut(key)` | `Option<&Value>` / `Option<&mut Value>` | Object |
| `value.get_index(i)` / `value.get_index_mut(i)` | `Option<&Value>` / `Option<&mut Value>` | Array |

## Comparing Values

```rust
use cjson::{parse, compare};

let a = parse(r#"{"Key": 1, "value": 2}"#)?;
let b = parse(r#"{"key": 1, "Value": 2}"#)?;

// Case-sensitive — keys must match exactly
assert!(!compare(&a, &b, true));

// Case-insensitive — "Key" == "key"
assert!(compare(&a, &b, false));
```

## Features

### `serde` (optional)

Enable `Serialize` and `Deserialize` for `Value` and `Number` — useful for
interop with generic serialization frameworks:

```rust
// Serialize a cjson::Value via serde
let value = parse(r#"{"msg": "hello"}"#)?;
let json_str = serde_json::to_string(&value)?;

// Deserialize into a cjson::Value
let value: cjson::Value = serde_json::from_str(r#"{"x": 1}"#)?;
```

### `utils` (optional)

JSON Pointer ([RFC 6901](https://datatracker.ietf.org/doc/html/rfc6901)),
JSON Patch ([RFC 6902](https://datatracker.ietf.org/doc/html/rfc6902)), and
MergePatch ([RFC 7396](https://datatracker.ietf.org/doc/html/rfc7396)).

```rust
use cjson::utils::*;

let doc = parse(r#"{"foo": ["bar", "baz"], "x": 1}"#)?;

// JSON Pointer
let found = get_pointer(&doc, "/foo/0");  // Some(&Value::String("bar"))

// JSON Patch
let patches = generate_patches(&doc, &new_doc);
apply_patches(&mut doc, &patches);

// MergePatch
merge_patch(&mut doc, &parse(r#"{"x": null}"#)?);  // removes x

// Sort object keys
let mut sorted = doc.clone();
sort_object(&mut sorted);
```

## `no_std`

The crate works in `no_std` environments with `alloc`:

```toml
[dependencies]
cjson = { version = "0.1", default-features = false, features = ["utils"] }
```

Add `features = ["std"]` explicitly to enable I/O helpers (`from_reader`,
`to_writer`).

## Comparison with cJSON (C)

| Feature | cjson-rs (Rust) | cJSON (C) |
|---|---|---|
| Safety | Zero `unsafe` — compiler guarantees | Manual memory management |
| Error handling | `Result<Value, Error>` with line/col info | Check `cJSON_IsError` |
| Strings | UTF-8 validated on parse | Raw C strings |
| Nesting limit | Configurable, default 1000 | Configurable, default 1000 |
| Number type | `f64` with `as_i64()` helper | `double` |
| Printing | `to_string()` / `to_string_minified()` with options | `cJSON_Print` / `cJSON_PrintUnformatted` |
| Object keys | `BTreeMap<String, Value>` — deterministic order | Linked list — insertion order |
| Mutation | `insert`, `push`, `remove_index`, `remove_key`, `replace_index`, `replace_key` | `AddItemToObject`, `AddItemToArray`, `DeleteItemFromObject` |
| Comparison | `compare()` with case-sensitive/insensitive modes | `cJSON_Compare` |
| `no_std` | Optional (drop `std` feature) | Always freestanding |
| serde | Optional `Serialize`/`Deserialize` | N/A |
| JSON Pointer/Patch | Optional `utils` feature | Separate `cJSON_Utils` |
| Unsafe | Zero (`#![forbid(unsafe_code)]`) | ~800 lines of pointer arithmetic |

## MSRV

Minimum supported Rust version: **1.65.0**.

## License

MIT. See [LICENSE](LICENSE).

## Contributing

1. All commits must end with `git push origin main`.
2. All `cargo test` must pass before commit.
3. No `unsafe` code — the crate forbids it.
4. Code is formatted with `cargo fmt` and linted with `cargo clippy`.

Tests include property-based tests via `proptest` and a suite ported from the
upstream cJSON test suite. Run them with:

```sh
cargo test
cargo test --features serde
cargo test --all-features
cargo clippy --all-features
```
