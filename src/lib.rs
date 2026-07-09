#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]

extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

pub mod compare;
mod error;
mod minify;
mod parse;
mod print;
mod raw;
mod value;

#[cfg(feature = "serde")]
mod serde_impl;
#[cfg(feature = "utils")]
pub mod utils;

pub use compare::compare;
pub use error::{Error, ErrorKind};
pub use minify::minify;
#[cfg(feature = "std")]
pub use parse::from_reader;
pub use parse::{parse, parse_with_options, ParseOptions};
#[cfg(feature = "std")]
pub use print::to_writer;
pub use print::{to_string, to_string_minified, to_string_with_options, PrintOptions};
pub use raw::RawValue;
pub use value::{Number, Value};

pub const fn version() -> &'static str {
    "0.1.0"
}
