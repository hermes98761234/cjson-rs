use crate::Value;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt::Write;

/// Options for JSON printing.
pub struct PrintOptions {
    pub indent: usize,
    pub sort_keys: bool,
}

impl Default for PrintOptions {
    fn default() -> Self {
        PrintOptions {
            indent: 2,
            sort_keys: false,
        }
    }
}

/// Pretty-print a JSON value with 2-space indent.
pub fn to_string(v: &Value) -> String {
    to_string_with_options(v, PrintOptions::default())
}

/// Print a JSON value with no whitespace (minified).
pub fn to_string_minified(v: &Value) -> String {
    let mut out = String::new();
    print_value(v, &mut out, 0, 0, false);
    out
}

/// Print a JSON value with custom options.
pub fn to_string_with_options(v: &Value, opts: PrintOptions) -> String {
    let mut out = String::new();
    print_value(v, &mut out, 0, opts.indent, opts.sort_keys);
    out
}

/// Write a JSON value to a writer.
#[cfg(feature = "std")]
pub fn to_writer<W: std::io::Write>(v: &Value, mut w: W) -> Result<(), crate::Error> {
    let s = to_string(v);
    w.write_all(s.as_bytes())
        .map_err(|_| crate::Error::new(crate::ErrorKind::InvalidUtf8, 0, 1, 1))
}

fn print_value(v: &Value, out: &mut String, depth: usize, indent: usize, sort_keys: bool) {
    match v {
        Value::Null => out.push_str("null"),
        Value::Bool(true) => out.push_str("true"),
        Value::Bool(false) => out.push_str("false"),
        Value::Number(n) => {
            write!(out, "{n}").unwrap();
        }
        Value::String(s) => print_string(s, out),
        Value::Array(arr) => print_array(arr, out, depth, indent, sort_keys),
        Value::Object(map) => print_object(map, out, depth, indent, sort_keys),
        Value::Raw(s) => out.push_str(s),
    }
}

fn print_string(s: &str, out: &mut String) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            '\x08' => out.push_str("\\b"),
            '\x0C' => out.push_str("\\f"),
            c if (c as u32) < 0x20 => {
                write!(out, "\\u{:04x}", c as u32).unwrap();
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

fn print_array(arr: &[Value], out: &mut String, depth: usize, indent: usize, sort_keys: bool) {
    out.push('[');
    if arr.is_empty() {
        out.push(']');
        return;
    }
    if indent > 0 {
        write_newline_indent(out, depth + 1, indent);
    }
    for (i, item) in arr.iter().enumerate() {
        if i > 0 {
            out.push(',');
            if indent > 0 {
                write_newline_indent(out, depth + 1, indent);
            }
        }
        print_value(item, out, depth + 1, indent, sort_keys);
    }
    if indent > 0 {
        write_newline_indent(out, depth, indent);
    }
    out.push(']');
}

fn print_object(
    map: &BTreeMap<String, Value>,
    out: &mut String,
    depth: usize,
    indent: usize,
    sort_keys: bool,
) {
    out.push('{');
    if map.is_empty() {
        out.push('}');
        return;
    }
    let entries: Vec<(&String, &Value)> = if sort_keys {
        let mut pairs: Vec<_> = map.iter().collect();
        pairs.sort_by(|a, b| a.0.cmp(b.0));
        pairs
    } else {
        map.iter().collect()
    };
    for (i, (key, value)) in entries.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        if indent > 0 {
            write_newline_indent(out, depth + 1, indent);
        }
        print_string(key, out);
        if indent > 0 {
            out.push_str(": ");
        } else {
            out.push(':');
        }
        print_value(value, out, depth + 1, indent, sort_keys);
    }
    if indent > 0 {
        write_newline_indent(out, depth, indent);
    }
    out.push('}');
}

fn write_newline_indent(out: &mut String, depth: usize, indent: usize) {
    out.push('\n');
    let spaces = depth * indent;
    for _ in 0..spaces {
        out.push(' ');
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Number;
    use alloc::vec;

    #[test]
    fn test_null() {
        assert_eq!(to_string_minified(&Value::Null), "null");
        assert_eq!(to_string(&Value::Null), "null");
    }

    #[test]
    fn test_bool() {
        assert_eq!(to_string_minified(&Value::Bool(true)), "true");
        assert_eq!(to_string_minified(&Value::Bool(false)), "false");
        assert_eq!(to_string(&Value::Bool(true)), "true");
    }

    #[test]
    fn test_number_integer() {
        let v = Value::Number(Number::from_i64(42));
        assert_eq!(to_string_minified(&v), "42");
    }

    #[test]
    fn test_number_negative() {
        let v = Value::Number(Number::from_i64(-17));
        assert_eq!(to_string_minified(&v), "-17");
    }

    #[test]
    fn test_number_float() {
        #[allow(clippy::approx_constant)]
        let v = Value::Number(3.14.into());
        let s = to_string_minified(&v);
        assert!(s.starts_with("3.14"), "got {s}");
    }

    #[test]
    fn test_string_empty() {
        assert_eq!(to_string_minified(&Value::String("".into())), "\"\"");
    }

    #[test]
    fn test_string_hello() {
        assert_eq!(
            to_string_minified(&Value::String("hello".into())),
            "\"hello\""
        );
    }

    #[test]
    fn test_string_escapes() {
        let s = Value::String("\"\\/\x08\x0C\n\r\t".into());
        assert_eq!(to_string_minified(&s), r#""\"\\/\b\f\n\r\t""#);
    }

    #[test]
    fn test_string_control_chars() {
        let s = Value::String("\x01\x02\x1f".into());
        assert_eq!(to_string_minified(&s), r#""\u0001\u0002\u001f""#);
    }

    #[test]
    fn test_string_utf8() {
        let s = Value::String("ü猫慕".into());
        assert_eq!(to_string_minified(&s), "\"ü猫慕\"");
    }

    #[test]
    fn test_array_empty() {
        assert_eq!(to_string_minified(&Value::Array(vec![])), "[]");
    }

    #[test]
    fn test_array_one_element() {
        let v = Value::Array(vec![Value::Number(1.into())]);
        assert_eq!(to_string_minified(&v), "[1]");
        assert_eq!(to_string(&v), "[\n  1\n]");
    }

    #[test]
    fn test_array_numbers() {
        let v = Value::Array(vec![
            Value::Number(Number::from_i64(1)),
            Value::Number(Number::from_i64(2)),
            Value::Number(Number::from_i64(3)),
        ]);
        assert_eq!(to_string_minified(&v), "[1,2,3]");
        assert_eq!(to_string(&v), "[\n  1,\n  2,\n  3\n]");
    }

    #[test]
    fn test_array_nested() {
        let v = Value::Array(vec![Value::Array(vec![])]);
        assert_eq!(to_string_minified(&v), "[[]]");
        assert_eq!(to_string(&v), "[\n  []\n]");
    }

    #[test]
    fn test_array_mixed() {
        let v = Value::Array(vec![
            Value::Number(1.into()),
            Value::Null,
            Value::Bool(true),
            Value::Bool(false),
            Value::Array(vec![]),
            Value::String("hello".into()),
            Value::Object(BTreeMap::new()),
        ]);
        assert_eq!(
            to_string_minified(&v),
            r#"[1,null,true,false,[],"hello",{}]"#
        );
    }

    #[test]
    fn test_object_empty() {
        assert_eq!(to_string_minified(&Value::Object(BTreeMap::new())), "{}");
    }

    #[test]
    fn test_object_one_element() {
        let mut map = BTreeMap::new();
        map.insert("one".into(), Value::Number(1.into()));
        let v = Value::Object(map);
        assert_eq!(to_string_minified(&v), r#"{"one":1}"#);
        assert_eq!(to_string(&v), "{\n  \"one\": 1\n}");
    }

    #[test]
    fn test_object_multiple_elements() {
        let mut map = BTreeMap::new();
        map.insert("one".into(), Value::Number(1.into()));
        map.insert("two".into(), Value::Number(2.into()));
        map.insert("three".into(), Value::Number(3.into()));
        let v = Value::Object(map);
        // BTreeMap is already sorted by key
        assert_eq!(to_string_minified(&v), r#"{"one":1,"three":3,"two":2}"#);
        assert_eq!(
            to_string(&v),
            "{\n  \"one\": 1,\n  \"three\": 3,\n  \"two\": 2\n}"
        );
    }

    #[test]
    fn test_object_nested() {
        let mut inner = BTreeMap::new();
        inner.insert("x".into(), Value::Number(2.into()));
        let mut outer = BTreeMap::new();
        outer.insert(
            "arr".into(),
            Value::Array(vec![Value::Number(1.into()), Value::Object(inner)]),
        );
        let v = Value::Object(outer);
        assert_eq!(to_string_minified(&v), r#"{"arr":[1,{"x":2}]}"#);
    }

    #[test]
    fn test_roundtrip_simple() {
        let cases = vec![
            "null",
            "true",
            "false",
            "42",
            "-17",
            "3.14",
            r#""hello""#,
            "[]",
            "{}",
        ];
        for input in cases {
            let parsed = crate::parse(input).unwrap();
            let printed = to_string_minified(&parsed);
            assert_eq!(printed, input, "roundtrip failed for {input}");
        }
    }

    #[test]
    fn test_roundtrip_complex() {
        let input = r#"{"array":[1,2,3],"bool":true,"null":null,"nested":{"a":1},"number":-3.14,"string":"hello"}"#;
        let parsed = crate::parse(input).unwrap();
        let printed = to_string_minified(&parsed);
        // parse back again
        let reparsed = crate::parse(&printed).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    fn test_sort_keys() {
        let mut map = BTreeMap::new();
        map.insert("z".into(), Value::Number(1.into()));
        map.insert("a".into(), Value::Number(2.into()));
        map.insert("m".into(), Value::Number(3.into()));
        let v = Value::Object(map);
        // Without sort_keys, BTreeMap is already sorted by key
        let opts = PrintOptions {
            indent: 2,
            sort_keys: true,
        };
        let s = to_string_with_options(&v, opts);
        assert_eq!(s, "{\n  \"a\": 2,\n  \"m\": 3,\n  \"z\": 1\n}");
    }

    #[test]
    fn test_indent0_is_minified() {
        let mut map = BTreeMap::new();
        map.insert("a".into(), Value::Number(1.into()));
        let v = Value::Object(map);
        let opts = PrintOptions {
            indent: 0,
            sort_keys: false,
        };
        assert_eq!(to_string_with_options(&v, opts), r#"{"a":1}"#);
    }

    #[test]
    fn test_object_with_strings() {
        let mut map = BTreeMap::new();
        map.insert("hello".into(), Value::String("world!".into()));
        let v = Value::Object(map);
        assert_eq!(to_string_minified(&v), r#"{"hello":"world!"}"#);
        assert_eq!(to_string(&v), "{\n  \"hello\": \"world!\"\n}");
    }

    #[test]
    fn test_object_with_array() {
        let mut map = BTreeMap::new();
        map.insert("array".into(), Value::Array(vec![]));
        let v = Value::Object(map);
        assert_eq!(to_string_minified(&v), r#"{"array":[]}"#);
        assert_eq!(to_string(&v), "{\n  \"array\": []\n}");
    }

    #[test]
    fn test_object_with_null() {
        let mut map = BTreeMap::new();
        map.insert("null".into(), Value::Null);
        let v = Value::Object(map);
        assert_eq!(to_string_minified(&v), r#"{"null":null}"#);
        assert_eq!(to_string(&v), "{\n  \"null\": null\n}");
    }
}
