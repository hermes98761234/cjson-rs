use crate::{Number, Value};
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;
use serde::de::{self, Deserialize, Deserializer, MapAccess, SeqAccess, Visitor};
use serde::ser::{Serialize, SerializeMap, SerializeSeq, Serializer};

// ---------------------------------------------------------------------------
// Serialize
// ---------------------------------------------------------------------------

impl Serialize for Number {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_f64(self.0)
    }
}

impl Serialize for Value {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Value::Null => serializer.serialize_unit(),
            Value::Bool(b) => serializer.serialize_bool(*b),
            Value::Number(n) => n.serialize(serializer),
            Value::String(s) => serializer.serialize_str(s),
            // Raw values are treated as opaque JSON strings
            Value::Raw(r) => serializer.serialize_str(r),
            Value::Array(v) => {
                let mut seq = serializer.serialize_seq(Some(v.len()))?;
                for e in v {
                    seq.serialize_element(e)?;
                }
                seq.end()
            }
            Value::Object(m) => {
                let mut map = serializer.serialize_map(Some(m.len()))?;
                for (k, v) in m {
                    map.serialize_entry(k, v)?;
                }
                map.end()
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Deserialize
// ---------------------------------------------------------------------------

struct ValueVisitor;

impl<'de> Visitor<'de> for ValueVisitor {
    type Value = Value;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a valid JSON value")
    }

    fn visit_bool<E: de::Error>(self, v: bool) -> Result<Value, E> {
        Ok(Value::Bool(v))
    }

    fn visit_i64<E: de::Error>(self, v: i64) -> Result<Value, E> {
        Ok(Value::Number(Number::from_i64(v)))
    }

    fn visit_u64<E: de::Error>(self, v: u64) -> Result<Value, E> {
        Ok(Value::Number(Number(v as f64)))
    }

    fn visit_f64<E: de::Error>(self, v: f64) -> Result<Value, E> {
        Number::from_f64(v)
            .map(Value::Number)
            .ok_or_else(|| de::Error::custom("infinity and NaN not allowed"))
    }

    fn visit_str<E: de::Error>(self, v: &str) -> Result<Value, E> {
        Ok(Value::String(v.to_owned()))
    }

    fn visit_string<E: de::Error>(self, v: String) -> Result<Value, E> {
        Ok(Value::String(v))
    }

    fn visit_unit<E: de::Error>(self) -> Result<Value, E> {
        Ok(Value::Null)
    }

    fn visit_none<E: de::Error>(self) -> Result<Value, E> {
        Ok(Value::Null)
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Value, A::Error> {
        let mut vec = if let Some(len) = seq.size_hint() {
            Vec::with_capacity(len)
        } else {
            Vec::new()
        };
        while let Some(elem) = seq.next_element::<Value>()? {
            vec.push(elem);
        }
        Ok(Value::Array(vec))
    }

    fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Value, M::Error> {
        let mut obj = BTreeMap::new();
        while let Some((key, value)) = map.next_entry::<String, Value>()? {
            obj.insert(key, value);
        }
        Ok(Value::Object(obj))
    }
}

impl<'de> Deserialize<'de> for Value {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(ValueVisitor)
    }
}

impl<'de> Deserialize<'de> for Number {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        f64::deserialize(deserializer).map(Number)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::parse;
    use crate::print::to_string;
    use alloc::string::String;

    #[test]
    fn serialize_null() {
        let v = Value::Null;
        let json = serde_json::to_string(&v).unwrap();
        assert_eq!(json, "null");
    }

    #[test]
    fn serialize_bool() {
        let v = Value::Bool(true);
        assert_eq!(serde_json::to_string(&v).unwrap(), "true");
        let v = Value::Bool(false);
        assert_eq!(serde_json::to_string(&v).unwrap(), "false");
    }

    #[test]
    fn serialize_number() {
        let v = Value::Number(Number(42.0));
        assert_eq!(serde_json::to_string(&v).unwrap(), "42.0");
        let v = Value::Number(Number(-3.14));
        assert_eq!(serde_json::to_string(&v).unwrap(), "-3.14");
    }

    #[test]
    fn serialize_string() {
        let v = Value::String("hello".into());
        assert_eq!(serde_json::to_string(&v).unwrap(), r#""hello""#);
    }

    #[test]
    fn serialize_array() {
        let v = Value::Array(vec![
            Value::Bool(true),
            Value::Number(Number(1.0)),
            Value::Null,
        ]);
        assert_eq!(serde_json::to_string(&v).unwrap(), "[true,1.0,null]");
    }

    #[test]
    fn serialize_object() {
        let mut obj = BTreeMap::new();
        obj.insert("a".into(), Value::Number(Number(1.0)));
        obj.insert("b".into(), Value::String("two".into()));
        let v = Value::Object(obj);
        let json = serde_json::to_string(&v).unwrap();
        // BTreeMap gives deterministic ordering: a, b
        assert_eq!(json, r#"{"a":1.0,"b":"two"}"#);
    }

    #[test]
    fn serialize_nested() {
        let mut inner = BTreeMap::new();
        inner.insert(
            "x".into(),
            Value::Array(vec![Value::Null, Value::Bool(false)]),
        );
        let v = Value::Object(inner);
        assert_eq!(serde_json::to_string(&v).unwrap(), r#"{"x":[null,false]}"#);
    }

    #[test]
    fn deserialize_null() {
        let v: Value = serde_json::from_str("null").unwrap();
        assert_eq!(v, Value::Null);
    }

    #[test]
    fn deserialize_bool() {
        let v: Value = serde_json::from_str("true").unwrap();
        assert_eq!(v, Value::Bool(true));
        let v: Value = serde_json::from_str("false").unwrap();
        assert_eq!(v, Value::Bool(false));
    }

    #[test]
    fn deserialize_number() {
        let v: Value = serde_json::from_str("42").unwrap();
        assert_eq!(v, Value::Number(Number(42.0)));
        let v: Value = serde_json::from_str("-3.14").unwrap();
        assert_eq!(v, Value::Number(Number(-3.14)));
    }

    #[test]
    fn deserialize_string() {
        let v: Value = serde_json::from_str(r#""hello""#).unwrap();
        assert_eq!(v, Value::String("hello".into()));
    }

    #[test]
    fn deserialize_array() {
        let v: Value = serde_json::from_str("[true, 1, null]").unwrap();
        assert_eq!(
            v,
            Value::Array(vec![
                Value::Bool(true),
                Value::Number(Number(1.0)),
                Value::Null,
            ])
        );
    }

    #[test]
    fn deserialize_object() {
        let v: Value = serde_json::from_str(r#"{"a": 1, "b": "two"}"#).unwrap();
        let mut expected = BTreeMap::new();
        expected.insert("a".into(), Value::Number(Number(1.0)));
        expected.insert("b".into(), Value::String("two".into()));
        assert_eq!(v, Value::Object(expected));
    }

    #[test]
    fn deserialize_nested() {
        let v: Value = serde_json::from_str(r#"{"x": [null, false]}"#).unwrap();
        let mut expected = BTreeMap::new();
        expected.insert(
            "x".into(),
            Value::Array(vec![Value::Null, Value::Bool(false)]),
        );
        assert_eq!(v, Value::Object(expected));
    }

    #[test]
    fn deserialize_empty_array() {
        let v: Value = serde_json::from_str("[]").unwrap();
        assert_eq!(v, Value::Array(vec![]));
    }

    #[test]
    fn deserialize_empty_object() {
        let v: Value = serde_json::from_str("{}").unwrap();
        assert_eq!(v, Value::Object(BTreeMap::new()));
    }

    #[test]
    fn deserialize_complex() {
        let json = r#"{
            "name": "test",
            "count": 42,
            "active": true,
            "tags": ["a", "b"],
            "meta": {"key": "val"}
        }"#;
        let v: Value = serde_json::from_str(json).unwrap();
        // Round-trip through our own parser/printer
        let expected = parse(json).unwrap();
        assert_eq!(v, expected);
    }

    #[test]
    fn round_trip_parse_then_serde() {
        let json = r#"{"name":"hello","arr":[1,2,null],"flag":false}"#;
        let parsed = parse(json).unwrap();
        let serde_serialized = serde_json::to_string(&parsed).unwrap();
        let serde_deserialized: Value = serde_json::from_str(&serde_serialized).unwrap();
        assert_eq!(parsed, serde_deserialized);
    }

    #[test]
    fn round_trip_serde_through_cjson() {
        let v: Value = serde_json::from_str(r#"{"a":[1,2,3],"b":{"c":"d"}}"#).unwrap();
        let printed = to_string(&v);
        let reparsed = parse(&printed).unwrap();
        assert_eq!(v, reparsed);
    }
}
