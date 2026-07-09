# cjson-rs Research Notes

## Upstream Audit (cJSON by DaveGamble)

**Cloned from:** https://github.com/DaveGamble/cJSON.git
**Reference location:** /tmp/cJSON (git clone --depth 1)

### API Surface

| Header | Public Functions |
|--------|-----------------|
| `cJSON.h` | **83** |
| `cJSON_Utils.h` | **14** |
| **Total** | **97** |

### Test Suite

22 test files in `/tmp/cJSON/tests/`:

| Test File | Purpose |
|-----------|---------|
| `parse_value.c` | All value type parsing |
| `parse_string.c` | String parsing with escapes |
| `parse_number.c` | Number parsing |
| `parse_object.c` | Object parsing |
| `parse_array.c` | Array parsing |
| `parse_examples.c` | Real-world examples |
| `parse_hex4.c` | Unicode hex escapes |
| `parse_with_opts.c` | Parse options |
| `print_value.c` | Value printing |
| `print_string.c` | String printing |
| `print_number.c` | Number printing |
| `print_object.c` | Object printing |
| `print_array.c` | Array printing |
| `cjson_add.c` | AddItemTo* functions |
| `compare_tests.c` | Comparison |
| `minify_tests.c` | Minify |
| `misc_tests.c` | Miscellaneous |
| `readme_examples.c` | README examples |
| `old_utils_tests.c` | Legacy utils |
| `misc_utils_tests.c` | Utils miscellaneous |
| `json_patch_tests.c` | JSON Patch |
| `unity_setup.c` | Test framework helper |

### Key Design Decisions (from spec)

- **Data model:** Enum-based `Value` tree (Null/Bool/Number/String/Array/Object/Raw)
- **Number type:** `f64` wrapper with `from_f64` (rejects NaN/Inf) and `from_i64`
- **Parser:** Recursive descent, default nesting limit 1000
- **Printer:** `Display`-based with formatted (2-space) and minified variants
- **Object keys:** `BTreeMap` for deterministic ordering (case-sensitive by default)
- **Safety:** Zero `unsafe` code
- **UTF-8:** All strings validated during parse
- **Features:** `std` (default), `utils` (JSON Pointer/Patch/MergePatch), `serde` (optional)
- **no_std:** Core parsing/printing works with `alloc` only

### Implementation Plan

The full plan covers 9 tasks, each with detailed code and test stubs ready to follow.

## License

MIT (same as upstream cJSON)
