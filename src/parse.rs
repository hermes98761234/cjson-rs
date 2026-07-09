use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use alloc::string::ToString;
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
    r.read_to_string(&mut s).map_err(|_e| Error::new(ErrorKind::InvalidUtf8, 0, 1, 1))?;
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
        let expected_str: &'static str = match expected {
            '"' => "\"",
            '[' => "[",
            ']' => "]",
            '{' => "{",
            '}' => "}",
            ':' => ":",
            ',' => ",",
            other => Box::leak(other.to_string().into_boxed_str()),
        };
        match self.peek() {
            Some(c) if c == expected => { self.advance(); Ok(()) }
            Some(c) => Err(self.err(ErrorKind::UnexpectedToken { expected: expected_str, found: c })),
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
    
    fn parse_literal(&mut self, literal: &'static str, value: Value) -> Result<Value, Error> {
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
