# cjson-rs Implementation Plan

> **For agentic workers:** Follow each task sequentially. Each task is self-contained. Work dir: `/home/user/projects/cjson-rs`

**Goal:** Rewrite cJSON as an idiomatic Rust JSON library with parser, printer, serde, and RFC JSON Pointer/Patch/MergePatch.

**Approach:** Enum-based `Value` tree (Null/Bool/Number/String/Array/Object/Raw), recursive descent parser, pretty/minified printer. Zero unsafe code. no_std+alloc compatible. MIT license.

**Tech Stack:** Rust 1.96+, cargo, serde (optional dep), proptest (dev-dep)

## Global Constraints

- Never use `unsafe` in any file
- Validate all strings as UTF-8 during parse
- Nesting limit default 1000
- All commits must end with `git push origin main`
- All tests must pass before commit

---

### Task 1: Research and Audit

**Files:** None (read-only)

- [ ] **Step 1: Clone upstream for reference**

```bash
cd /tmp && rm -rf cJSON && git clone --depth 1 https://github.com/DaveGamble/cJSON.git && cd /tmp/cJSON && ls
```

Expected: cJSON.c, cJSON.h, cJSON_Utils.c, cJSON_Utils.h, tests/ directory with ~20 .c files.

- [ ] **Step 2: Count API surface**

```bash
grep -c 'CJSON_PUBLIC' /tmp/cJSON/cJSON.h
grep -c 'CJSON_PUBLIC' /tmp/cJSON/cJSON_Utils.h
ls /tmp/cJSON/tests/*.c | wc -l
```

Expected: ~50 public functions in cJSON.h, ~20 in cJSON_Utils.h, ~20 test files.

- [ ] **Step 3: Read the design spec**

```bash
cat /home/user/projects/cjson-rs/2026-07-09-cjson-rs-design.md
```

- [ ] **Step 4: Commit (even if only notes)**

```bash
cd /home/user/projects/cjson-rs && touch .gitkeep && git add .gitkeep && git commit -m "chore: begin cjson-rs project" && echo "done"
```

---

### Task 2: Scaffold Project and Create GitHub Repo

**Files:**
- Create: `Cargo.toml`
- Create: `src/lib.rs`
- Create: `src/value.rs`
- Create: `src/error.rs`
- Create: `src/raw.rs`
- Create: `src/minify.rs` (stub)
- Create: `src/parse.rs` (stub)
- Create: `src/print.rs` (stub)
- Create: `LICENSE`
- Create: `.gitignore`

- [ ] **Step 1: Create Cargo.toml**

```toml
[package]
name = "cjson"
version = "0.1.0"
edition = "2021"
description = "Idiomatic Rust rewrite of cJSON — ultralightweight JSON parser"
license = "MIT"
repository = "https://github.com/hermes98761234/cjson-rs"

[features]
default = ["std"]
std = []
utils = ["std"]
serde = ["dep:serde"]

[dependencies]
serde = { version = "1", optional = true, features = ["derive"] }

[dev-dependencies]
proptest = "1"
serde_json = "1"

[lib]
name = "cjson"
path = "src/lib.rs"
```

- [ ] **Step 2: Create src/lib.rs**

Write this exact content to `src/lib.rs`:

```rust
#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]

extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

mod value;
mod error;
mod parse;
mod print;
mod raw;
mod minify;

#[cfg(feature = "utils")]
pub mod utils;
#[cfg(feature = "serde")]
mod serde_impl;

pub use value::{Value, Number};
pub use error::{Error, ErrorKind};
pub use parse::{parse, parse_with_options, from_reader, ParseOptions};
pub use print::{to_string, to_string_minified, to_writer, to_string_with_options, PrintOptions};
pub use raw::RawValue;
pub use minify::minify;

pub const fn version() -> &'static str { "0.1.0" }
```

- [ ] **Step 3: Create src/error.rs**

Write this exact content:

```rust
use core::fmt;

#[derive(Clone, Debug, PartialEq)]
pub struct Error {
    pub kind: ErrorKind,
    pub position: usize,
    pub line: usize,
    pub column: usize,
}

impl Error {
    pub fn new(kind: ErrorKind, position: usize, line: usize, column: usize) -> Self {
        Error { kind, position, line, column }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} at pos {} (line {}, col {})", self.kind, self.position, self.line, self.column)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

#[derive(Clone, Debug, PartialEq)]
pub enum ErrorKind {
    UnexpectedToken { expected: &'static str, found: char },
    UnexpectedEndOfInput,
    InvalidNumber,
    InvalidStringEscape,
    InvalidUnicodeSurrogate,
    ExceededNestingLimit,
    TrailingComma,
    TrailingGarbage,
    InvalidUtf8,
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorKind::UnexpectedToken { expected, found } => write!(f, "expected {expected}, found '{found}'"),
            ErrorKind::UnexpectedEndOfInput => write!(f, "unexpected end of input"),
            ErrorKind::InvalidNumber => write!(f, "invalid number"),
            ErrorKind::InvalidStringEscape => write!(f, "invalid string escape"),
            ErrorKind::InvalidUnicodeSurrogate => write!(f, "invalid unicode surrogate"),
            ErrorKind::ExceededNestingLimit => write!(f, "exceeded nesting limit"),
            ErrorKind::TrailingComma => write!(f, "trailing comma"),
            ErrorKind::TrailingGarbage => write!(f, "trailing garbage"),
            ErrorKind::InvalidUtf8 => write!(f, "invalid UTF-8"),
        }
    }
}
```

- [ ] **Step 4: Create src/value.rs**

Write this exact content:

```rust
use alloc::collections::BTreeMap;
use alloc::string::String;
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
    pub fn null() -> Self { Value::Null }
    pub fn bool(b: bool) -> Self { Value::Bool(b) }
    pub fn number(n: impl Into<Number>) -> Self { Value::Number(n.into()) }
    pub fn string(s: impl Into<String>) -> Self { Value::String(s.into()) }
    pub fn array() -> Self { Value::Array(Vec::new()) }
    pub fn object() -> Self { Value::Object(BTreeMap::new()) }
    pub fn raw(s: impl Into<String>) -> Self { Value::Raw(s.into()) }

    pub fn is_null(&self) -> bool { matches!(self, Value::Null) }
    pub fn is_bool(&self) -> bool { matches!(self, Value::Bool(_)) }
    pub fn is_number(&self) -> bool { matches!(self, Value::Number(_)) }
    pub fn is_string(&self) -> bool { matches!(self, Value::String(_)) }
    pub fn is_array(&self) -> bool { matches!(self, Value::Array(_)) }
    pub fn is_object(&self) -> bool { matches!(self, Value::Object(_)) }
    pub fn is_raw(&self) -> bool { matches!(self, Value::Raw(_)) }

    pub fn as_bool(&self) -> Option<bool> { if let Value::Bool(b)=self{Some(*b)}else{None} }
    pub fn as_f64(&self) -> Option<f64> { if let Value::Number(n)=self{Some(n.0)}else{None} }
    pub fn as_i64(&self) -> Option<i64> { if let Value::Number(n)=self{n.as_i64()}else{None} }
    pub fn as_str(&self) -> Option<&str> { if let Value::String(s)=self{Some(s)}else{None} }
    pub fn as_array(&self) -> Option<&[Value]> { if let Value::Array(v)=self{Some(v)}else{None} }
    pub fn as_array_mut(&mut self) -> Option<&mut Vec<Value>> { if let Value::Array(v)=self{Some(v)}else{None} }
    pub fn as_object(&self) -> Option<&BTreeMap<String,Value>> { if let Value::Object(m)=self{Some(m)}else{None} }
    pub fn as_object_mut(&mut self) -> Option<&mut BTreeMap<String,Value>> { if let Value::Object(m)=self{Some(m)}else{None} }

    pub fn len(&self) -> usize { match self { Value::Array(v)=>v.len(), Value::Object(m)=>m.len(), _=>0 } }
    pub fn is_empty(&self) -> bool { self.len()==0 }
    pub fn get(&self, key: &str) -> Option<&Value> { if let Value::Object(m)=self{m.get(key)}else{None} }
    pub fn get_mut(&mut self, key: &str) -> Option<&mut Value> { if let Value::Object(m)=self{m.get_mut(key)}else{None} }
    pub fn get_index(&self, i: usize) -> Option<&Value> { if let Value::Array(v)=self{v.get(i)}else{None} }
    pub fn get_index_mut(&mut self, i: usize) -> Option<&mut Value> { if let Value::Array(v)=self{v.get_mut(i)}else{None} }
    pub fn push(&mut self, v: Value) { if let Value::Array(ref mut a)=self{a.push(v)} }
    pub fn insert(&mut self, key: impl Into<String>, v: Value) { if let Value::Object(ref mut m)=self{m.insert(key.into(),v);} }
    pub fn has_key(&self, key: &str) -> bool { matches!(self, Value::Object(m) if m.contains_key(key)) }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Number(pub f64);

impl Number {
    pub fn from_f64(n: f64) -> Option<Self> { if n.is_finite(){Some(Number(n))}else{None} }
    pub fn from_i64(n: i64) -> Self { Number(n as f64) }
    pub fn as_f64(&self) -> f64 { self.0 }
    pub fn as_i64(&self) -> Option<i64> {
        if self.0.fract()==0.0 && self.0>=i64::MIN as f64 && self.0<=i64::MAX as f64 { Some(self.0 as i64) } else { None }
    }
    pub fn is_integer(&self) -> bool { self.0.fract()==0.0 }
}
impl From<f64> for Number { fn from(n: f64) -> Self { Number(n) } }
impl From<i64> for Number { fn from(n: i64) -> Self { Number(n as f64) } }
impl fmt::Display for Number {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_integer() && self.0.abs()<=i64::MAX as f64 { write!(f,"{}",self.0 as i64) } else { write!(f,"{}",self.0) }
    }
}
```

- [ ] **Step 5: Create stubs for parse.rs, print.rs, raw.rs, minify.rs**

`src/raw.rs`:
```rust
use alloc::string::String;
#[derive(Clone, Debug, PartialEq)]
pub struct RawValue(pub String);
impl RawValue { pub fn new(s: impl Into<String>) -> Self { RawValue(s.into()) } }
```

`src/parse.rs` stub:
```rust
pub struct ParseOptions {
    pub max_nesting: usize,
    pub require_null_terminated: bool,
}
impl Default for ParseOptions {
    fn default() -> Self { ParseOptions { max_nesting: 1000, require_null_terminated: true } }
}
pub fn parse(input: &str) -> Result<crate::Value, crate::Error> { todo!() }
pub fn parse_with_options(input: &str, _opts: ParseOptions) -> Result<crate::Value, crate::Error> { todo!() }
#[cfg(feature="std")]
pub fn from_reader<R: std::io::Read>(_r: R) -> Result<crate::Value, crate::Error> { todo!() }
```

`src/print.rs` stub:
```rust
use crate::Value;
pub struct PrintOptions { pub indent: usize, pub sort_keys: bool }
impl Default for PrintOptions { fn default() -> Self { PrintOptions { indent: 2, sort_keys: false } } }
pub fn to_string(v: &Value) -> String { todo!() }
pub fn to_string_minified(v: &Value) -> String { todo!() }
pub fn to_string_with_options(v: &Value, _opts: PrintOptions) -> String { todo!() }
#[cfg(feature="std")]
pub fn to_writer<W: std::io::Write>(v: &Value, _w: W) -> Result<(), crate::Error> { todo!() }
```

`src/minify.rs`:
```rust
pub fn minify(input: &str) -> String {
    // Remove whitespace outside strings
    let mut out = alloc::string::String::with_capacity(input.len());
    let mut in_string = false;
    let mut escape = false;
    for c in input.chars() {
        if in_string {
            out.push(c);
            if escape { escape = false; }
            else if c == '\\' { escape = true; }
            else if c == '"' { in_string = false; }
        } else {
            if c == '"' { in_string = true; out.push(c); }
            else if !c.is_ascii_whitespace() { out.push(c); }
        }
    }
    out
}
```

- [ ] **Step 6: Create .gitignore**

```
target/
*.swp
*.swo
*.orig
*.log
```

- [ ] **Step 7: Create LICENSE**

```
MIT License

Copyright (c) 2026 cjson-rs contributors

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

- [ ] **Step 8: Verify build**

```bash
cd /home/user/projects/cjson-rs && cargo check 2>&1
```

Expected: warnings about `todo!()` but no errors.

- [ ] **Step 9: Create GitHub repo and push**

```bash
cd /home/user/projects/cjson-rs && git branch -M main
gh repo create hermes98761234/cjson-rs --public --description 'Idiomatic Rust rewrite of cJSON — ultralightweight JSON parser' --source . --remote origin --push 2>&1
```

Expected: GitHub URL printed.

- [ ] **Step 10: Commit scaffold**

```bash
cd /home/user/projects/cjson-rs && git add -A && git commit -m "feat: scaffold cjson-rs project" && git push origin main

---

### Task 3: Implement Parser (parse_value, parse_string, parse_number)

**Files:**
- Modify: `src/parse.rs` (full implementation)
- Create: `tests/parse_value.rs`
- Create: `tests/parse_string.rs`
- Create: `tests/parse_number.rs`

- [ ] **Step 1: Read cJSON parse_value and parse_string source for reference**

```bash
head -200 /tmp/cJSON/cJSON.c
cat /tmp/cJSON/tests/parse_value.c
cat /tmp/cJSON/tests/parse_string.c
cat /tmp/cJSON/tests/parse_number.c
```

- [ ] **Step 2: Implement src/parse.rs with recursive descent parser**

Replace the stub with a complete parser. Here's the implementation:

```rust
use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use crate::{Value, Number, Error, ErrorKind};

pub struct ParseOptions {
    pub max_nesting: usize,
    pub require_null_terminated: bool,
}
impl Default for ParseOptions {
    fn default() -> Self { ParseOptions { max_nesting: 1000, require_null_terminated: true } }
}

pub fn parse(input: &str) -> Result<Value, Error> {
    parse_with_options(input, ParseOptions::default())
}

pub fn parse_with_options(input: &str, opts: ParseOptions) -> Result<Value, Error> {
    let mut parser = Parser { input, pos: 0, line: 1, col: 1, max_nesting: opts.max_nesting };
    let value = parser.parse_value(0)?;
    if opts.require_null_terminated {
        parser.skip_whitespace();
        if parser.pos < input.len() {
            return Err(parser.err(ErrorKind::TrailingGarbage));
        }
    }
    Ok(value)
}

#[cfg(feature="std")]
pub fn from_reader<R: std::io::Read>(mut r: R) -> Result<Value, Error> {
    let mut s = String::new();
    r.read_to_string(&mut s).map_err(|e| Error::new(ErrorKind::InvalidUtf8, 0, 1, 1))?;
    parse(&s)
}

struct Parser<'a> {
    input: &'a str,
    pos: usize,
    line: usize,
    col: usize,
    max_nesting: usize,
}

impl<'a> Parser<'a> {
    fn peek(&self) -> Option<char> { self.input[self.pos..].chars().next() }
    
    fn advance(&mut self) {
        if let Some(c) = self.peek() {
            self.pos += c.len_utf8();
            if c == '\n' { self.line += 1; self.col = 1; } else { self.col += 1; }
        }
    }
    
    fn err(&self, kind: ErrorKind) -> Error {
        Error::new(kind, self.pos, self.line, self.col)
    }
    
    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_ascii_whitespace() { self.advance(); } else { break; }
        }
    }
    
    fn expect_char(&mut self, expected: char) -> Result<(), Error> {
        match self.peek() {
            Some(c) if c == expected => { self.advance(); Ok(()) }
            Some(c) => Err(self.err(ErrorKind::UnexpectedToken { expected: expected.to_string().leak(), found: c })),
            None => Err(self.err(ErrorKind::UnexpectedEndOfInput)),
        }
    }
    
    fn parse_value(&mut self, depth: usize) -> Result<Value, Error> {
        if depth > self.max_nesting {
            return Err(self.err(ErrorKind::ExceededNestingLimit));
        }
        self.skip_whitespace();
        match self.peek() {
            Some('n') => self.parse_literal("null", Value::Null),
            Some('t') => self.parse_literal("true", Value::Bool(true)),
            Some('f') => self.parse_literal("false", Value::Bool(false)),
            Some('"') => self.parse_string(),
            Some('[') => self.parse_array(depth),
            Some('{') => self.parse_object(depth),
            Some(c) if c == '-' || c.is_ascii_digit() => self.parse_number(),
            Some(c) => Err(self.err(ErrorKind::UnexpectedToken { expected: "a value", found: c })),
            None => Err(self.err(ErrorKind::UnexpectedEndOfInput)),
        }
    }
    
    fn parse_literal(&mut self, literal: &str, value: Value) -> Result<Value, Error> {
        for expected_char in literal.chars() {
            if self.peek() == Some(expected_char) { self.advance(); }
            else { return Err(self.err(ErrorKind::UnexpectedToken { expected: literal, found: self.peek().unwrap_or('\0') })); }
        }
        Ok(value)
    }
    
    fn parse_string(&mut self) -> Result<Value, Error> {
        self.expect_char('"')?;
        let mut s = String::new();
        loop {
            match self.peek() {
                None => return Err(self.err(ErrorKind::UnexpectedEndOfInput)),
                Some('"') => { self.advance(); return Ok(Value::String(s)); }
                Some('\\') => {
                    self.advance();
                    match self.peek() {
                        Some('"') => { s.push('"'); self.advance(); }
                        Some('\\') => { s.push('\\'); self.advance(); }
                        Some('/') => { s.push('/'); self.advance(); }
                        Some('b') => { s.push('\x08'); self.advance(); }
                        Some('f') => { s.push('\x0C'); self.advance(); }
                        Some('n') => { s.push('\n'); self.advance(); }
                        Some('r') => { s.push('\r'); self.advance(); }
                        Some('t') => { s.push('\t'); self.advance(); }
                        Some('u') => { self.advance(); s.push(self.parse_unicode_escape()?); }
                        Some(c) => return Err(self.err(ErrorKind::InvalidStringEscape)),
                        None => return Err(self.err(ErrorKind::UnexpectedEndOfInput)),
                    }
                }
                Some(c) => { s.push(c); self.advance(); }
            }
        }
    }
    
    fn parse_unicode_escape(&mut self) -> Result<char, Error> {
        let hex: String = (0..4).filter_map(|_| {
            self.peek().map(|c| { self.advance(); c })
        }).collect();
        if hex.len() != 4 { return Err(self.err(ErrorKind::InvalidUnicodeSurrogate)); }
        let code = u32::from_str_radix(&hex, 16).map_err(|_| self.err(ErrorKind::InvalidUnicodeSurrogate))?;
        // Handle surrogate pairs
        if (0xD800..=0xDBFF).contains(&code) {
            if self.peek() == Some('\\') { self.advance(); }
            if self.peek() == Some('u') { self.advance(); } else { return Err(self.err(ErrorKind::InvalidUnicodeSurrogate)); }
            let hex2: String = (0..4).filter_map(|_| self.peek().map(|c|{self.advance();c})).collect();
            let low = u32::from_str_radix(&hex2, 16).map_err(|_| self.err(ErrorKind::InvalidUnicodeSurrogate))?;
            if !(0xDC00..=0xDFFF).contains(&low) { return Err(self.err(ErrorKind::InvalidUnicodeSurrogate)); }
            let cp = ((code - 0xD800) << 10) + (low - 0xDC00) + 0x10000;
            char::from_u32(cp).ok_or_else(|| self.err(ErrorKind::InvalidUnicodeSurrogate))
        } else {
            char::from_u32(code).ok_or_else(|| self.err(ErrorKind::InvalidUnicodeSurrogate))
        }
    }
    
    fn parse_number(&mut self) -> Result<Value, Error> {
        let start = self.pos;
        // Consume minus
        if self.peek() == Some('-') { self.advance(); }
        // Integer part
        match self.peek() {
            Some('0') => { self.advance(); }
            Some(d) if d.is_ascii_digit() => { self.advance(); while self.peek().map_or(false, |c|c.is_ascii_digit()){self.advance();} }
            _ => return Err(self.err(ErrorKind::InvalidNumber)),
        }
        // Fractional part
        if self.peek() == Some('.') {
            self.advance();
            if !self.peek().map_or(false, |c|c.is_ascii_digit()) { return Err(self.err(ErrorKind::InvalidNumber)); }
            while self.peek().map_or(false, |c|c.is_ascii_digit()) { self.advance(); }
        }
        // Exponent part
        if self.peek().map_or(false, |c| c == 'e' || c == 'E') {
            self.advance();
            if self.peek().map_or(false, |c| c == '+' || c == '-') { self.advance(); }
            if !self.peek().map_or(false, |c|c.is_ascii_digit()) { return Err(self.err(ErrorKind::InvalidNumber)); }
            while self.peek().map_or(false, |c|c.is_ascii_digit()) { self.advance(); }
        }
        let num_str = &self.input[start..self.pos];
        let n: f64 = num_str.parse().map_err(|_| self.err(ErrorKind::InvalidNumber))?;
        if !n.is_finite() { return Err(self.err(ErrorKind::InvalidNumber)); }
        Ok(Value::Number(Number(n)))
    }
    
    fn parse_array(&mut self, depth: usize) -> Result<Value, Error> {
        self.expect_char('[')?;
        let mut arr = Vec::new();
        self.skip_whitespace();
        if self.peek() == Some(']') { self.advance(); return Ok(Value::Array(arr)); }
        loop {
            arr.push(self.parse_value(depth + 1)?);
            self.skip_whitespace();
            match self.peek() {
                Some(',') => { self.advance(); self.skip_whitespace(); }
                Some(']') => { self.advance(); return Ok(Value::Array(arr)); }
                _ => return Err(self.err(ErrorKind::UnexpectedToken { expected: "]", found: self.peek().unwrap_or('\0') })),
            }
        }
    }
    
    fn parse_object(&mut self, depth: usize) -> Result<Value, Error> {
        self.expect_char('{')?;
        let mut map = BTreeMap::new();
        self.skip_whitespace();
        if self.peek() == Some('}') { self.advance(); return Ok(Value::Object(map)); }
        loop {
            self.skip_whitespace();
            let key_val = self.parse_string()?;
            let key = match key_val { Value::String(s) => s, _ => return Err(self.err(ErrorKind::InvalidUtf8)) };
            self.skip_whitespace();
            self.expect_char(':')?;
            let val = self.parse_value(depth + 1)?;
            map.insert(key, val);
            self.skip_whitespace();
            match self.peek() {
                Some(',') => { self.advance(); }
                Some('}') => { self.advance(); return Ok(Value::Object(map)); }
                _ => return Err(self.err(ErrorKind::UnexpectedToken { expected: "}", found: self.peek().unwrap_or('\0') })),
            }
        }
    }
}
```

- [ ] **Step 3: Create and run parse_value tests**

Create `tests/parse_value.rs`:

```rust
use cjson::{parse, Value};

#[test]
fn parse_null() { assert_eq!(parse("null").unwrap(), Value::Null); }
#[test]
fn parse_true() { assert_eq!(parse("true").unwrap(), Value::Bool(true)); }
#[test]
fn parse_false() { assert_eq!(parse("false").unwrap(), Value::Bool(false)); }
#[test]
fn parse_number_integer() { assert_eq!(parse("42").unwrap(), Value::Number(42.into())); }
#[test]
fn parse_number_negative() { assert_eq!(parse("-17").unwrap(), Value::Number((-17).into())); }
#[test]
fn parse_number_float() { let v=parse("3.14").unwrap(); assert!((v.as_f64().unwrap()-3.14).abs()<1e-10); }
#[test]
fn parse_number_exponent() { let v=parse("1.5e2").unwrap(); assert!((v.as_f64().unwrap()-150.0).abs()<1e-10); }
#[test]
fn parse_string_empty() { assert_eq!(parse(r#"""#).unwrap(), Value::String("".into())); }
#[test]
fn parse_string_hello() { assert_eq!(parse(r#""hello""#).unwrap(), Value::String("hello".into())); }
#[test]
fn parse_string_escapes() { assert_eq!(parse(r#""\"\\\/\b\f\n\r\t""#).unwrap(), Value::String("\"\\/\x08\x0C\n\r\t".into())); }
#[test]
fn parse_empty_array() { assert_eq!(parse("[]").unwrap(), Value::Array(vec![])); }
#[test]
fn parse_array_numbers() {
    let v=parse("[1,2,3]").unwrap();
    let arr=v.as_array().unwrap();
    assert_eq!(arr.len(),3);
    assert_eq!(arr[0],Value::Number(1.into()));
    assert_eq!(arr[2],Value::Number(3.into()));
}
#[test]
fn parse_empty_object() { assert_eq!(parse("{}").unwrap(), Value::Object(std::collections::BTreeMap::new())); }
#[test]
fn parse_object_basic() {
    let v=parse(r#"{"a":1,"b":2}"#).unwrap();
    assert_eq!(v.get("a"), Some(&Value::Number(1.into())));
    assert_eq!(v.get("b"), Some(&Value::Number(2.into())));
}
#[test]
fn parse_nested() {
    let v=parse(r#"{"arr":[1,{"x":2}]}"#).unwrap();
    assert_eq!(v.get("arr").and_then(|a|a.get_index(1)).and_then(|o|o.get("x")), Some(&Value::Number(2.into())));
}
#[test]
fn parse_error_trailing_garbage() { assert!(parse("null x").is_err()); }
```

Actually, write each test file minified for space. Tests should be placed at `tests/parse_value.rs`, `tests/parse_string.rs`, `tests/parse_number.rs`. Stub each file with cargo-test compatible imports:

```rust
use cjson::{parse, Value};
```

And port at least the first 5-10 tests from each corresponding /tmp/cJSON/tests/ file. Key tests to port:
- parse_value.c: all value type parsing
- parse_string.c: escape