#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]

extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

mod value;
mod error;
mod parse;
mod print;
pub mod compare;
mod raw;
mod minify;

#[cfg(feature = "utils")]
pub mod utils;
#[cfg(feature = "serde")]
mod serde_impl;

pub use value::{Value, Number};
pub use error::{Error, ErrorKind};
#[cfg(feature = "std")]
pub use parse::from_reader;
pub use parse::{parse, parse_with_options, ParseOptions};
#[cfg(feature = "std")]
pub use print::to_writer;
pub use print::{to_string, to_string_minified, to_string_with_options, PrintOptions};
pub use raw::RawValue;
pub use minify::minify;
pub use compare::compare;

pub const fn version() -> &'static str { "0.1.0" }
