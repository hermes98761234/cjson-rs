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
        Error {
            kind,
            position,
            line,
            column,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} at pos {} (line {}, col {})",
            self.kind, self.position, self.line, self.column
        )
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
            ErrorKind::UnexpectedToken { expected, found } => {
                write!(f, "expected {expected}, found '{found}'")
            }
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
