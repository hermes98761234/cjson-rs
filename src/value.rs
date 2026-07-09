use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use core::fmt;

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Number(Number),
    String(String),
    Array(Vec<Value>),
    Object(BTreeMap<String, Value>),
    Raw(String),
}

impl Value {
    pub fn null() -> Self {
        Value::Null
    }
    pub fn bool(b: bool) -> Self {
        Value::Bool(b)
    }
    pub fn number(n: impl Into<Number>) -> Self {
        Value::Number(n.into())
    }
    pub fn string(s: impl Into<String>) -> Self {
        Value::String(s.into())
    }
    pub fn array() -> Self {
        Value::Array(Vec::new())
    }
    pub fn object() -> Self {
        Value::Object(BTreeMap::new())
    }
    pub fn raw(s: impl Into<String>) -> Self {
        Value::Raw(s.into())
    }

    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }
    pub fn is_bool(&self) -> bool {
        matches!(self, Value::Bool(_))
    }
    pub fn is_true(&self) -> bool {
        matches!(self, Value::Bool(true))
    }
    pub fn is_false(&self) -> bool {
        matches!(self, Value::Bool(false))
    }
    pub fn is_number(&self) -> bool {
        matches!(self, Value::Number(_))
    }
    pub fn is_string(&self) -> bool {
        matches!(self, Value::String(_))
    }
    pub fn is_array(&self) -> bool {
        matches!(self, Value::Array(_))
    }
    pub fn is_object(&self) -> bool {
        matches!(self, Value::Object(_))
    }
    pub fn is_raw(&self) -> bool {
        matches!(self, Value::Raw(_))
    }

    pub fn as_bool(&self) -> Option<bool> {
        if let Value::Bool(b) = self {
            Some(*b)
        } else {
            None
        }
    }
    pub fn as_f64(&self) -> Option<f64> {
        if let Value::Number(n) = self {
            Some(n.0)
        } else {
            None
        }
    }
    pub fn as_i64(&self) -> Option<i64> {
        if let Value::Number(n) = self {
            n.as_i64()
        } else {
            None
        }
    }
    pub fn as_str(&self) -> Option<&str> {
        if let Value::String(s) = self {
            Some(s)
        } else {
            None
        }
    }
    pub fn as_array(&self) -> Option<&[Value]> {
        if let Value::Array(v) = self {
            Some(v)
        } else {
            None
        }
    }
    pub fn as_array_mut(&mut self) -> Option<&mut Vec<Value>> {
        if let Value::Array(v) = self {
            Some(v)
        } else {
            None
        }
    }
    pub fn as_object(&self) -> Option<&BTreeMap<String, Value>> {
        if let Value::Object(m) = self {
            Some(m)
        } else {
            None
        }
    }
    pub fn as_object_mut(&mut self) -> Option<&mut BTreeMap<String, Value>> {
        if let Value::Object(m) = self {
            Some(m)
        } else {
            None
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Value::Array(v) => v.len(),
            Value::Object(m) => m.len(),
            _ => 0,
        }
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn get(&self, key: &str) -> Option<&Value> {
        if let Value::Object(m) = self {
            m.get(key)
        } else {
            None
        }
    }
    pub fn get_mut(&mut self, key: &str) -> Option<&mut Value> {
        if let Value::Object(m) = self {
            m.get_mut(key)
        } else {
            None
        }
    }
    pub fn get_index(&self, i: usize) -> Option<&Value> {
        if let Value::Array(v) = self {
            v.get(i)
        } else {
            None
        }
    }
    pub fn get_index_mut(&mut self, i: usize) -> Option<&mut Value> {
        if let Value::Array(v) = self {
            v.get_mut(i)
        } else {
            None
        }
    }
    pub fn push(&mut self, v: Value) {
        if let Value::Array(ref mut a) = self {
            a.push(v)
        }
    }
    pub fn insert(&mut self, key: impl Into<String>, v: Value) {
        if let Value::Object(ref mut m) = self {
            m.insert(key.into(), v);
        }
    }
    pub fn has_key(&self, key: &str) -> bool {
        matches!(self, Value::Object(m) if m.contains_key(key))
    }

    /// Remove an element from an array at the given index. Returns the removed value, or None if out of bounds or wrong type.
    pub fn remove_index(&mut self, index: usize) -> Option<Value> {
        if let Value::Array(ref mut v) = self {
            if index < v.len() {
                Some(v.remove(index))
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Remove a key from an object. Returns the removed value, or None if key not found or wrong type.
    pub fn remove_key(&mut self, key: &str) -> Option<Value> {
        if let Value::Object(ref mut m) = self {
            m.remove(key)
        } else {
            None
        }
    }

    /// Insert a value into an array at the given index. Panics if index > len.
    pub fn insert_in_array(&mut self, index: usize, value: Value) {
        if let Value::Array(ref mut v) = self {
            if index <= v.len() {
                v.insert(index, value);
            }
        }
    }

    /// Replace an element in an array at the given index. Returns the old value, or None if out of bounds or wrong type.
    pub fn replace_index(&mut self, index: usize, value: Value) -> Option<Value> {
        if let Value::Array(ref mut v) = self {
            if index < v.len() {
                let old = core::mem::replace(&mut v[index], value);
                Some(old)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Replace a value in an object by key. Returns the old value, or None if key not found or wrong type.
    pub fn replace_key(&mut self, key: &str, value: Value) -> Option<Value> {
        if let Value::Object(ref mut m) = self {
            m.insert(key.to_string(), value)
        } else {
            None
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Number(pub f64);

impl Number {
    pub fn from_f64(n: f64) -> Option<Self> {
        if n.is_finite() {
            Some(Number(n))
        } else {
            None
        }
    }
    pub fn from_i64(n: i64) -> Self {
        Number(n as f64)
    }
    pub fn as_f64(&self) -> f64 {
        self.0
    }
    pub fn as_i64(&self) -> Option<i64> {
        if self.0 >= i64::MIN as f64
            && self.0 <= i64::MAX as f64
            && (self.0 as i64) as f64 == self.0
        {
            Some(self.0 as i64)
        } else {
            None
        }
    }
    pub fn is_integer(&self) -> bool {
        self.0 >= i64::MIN as f64 && self.0 <= i64::MAX as f64 && (self.0 as i64) as f64 == self.0
    }
}
impl From<f64> for Number {
    fn from(n: f64) -> Self {
        Number(n)
    }
}
impl From<i64> for Number {
    fn from(n: i64) -> Self {
        Number(n as f64)
    }
}
impl fmt::Display for Number {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_integer() && self.0.abs() <= i64::MAX as f64 {
            write!(f, "{}", self.0 as i64)
        } else {
            write!(f, "{}", self.0)
        }
    }
}
