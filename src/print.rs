use crate::Value;
pub struct PrintOptions { pub indent: usize, pub sort_keys: bool }
impl Default for PrintOptions { fn default() -> Self { PrintOptions { indent: 2, sort_keys: false } } }
pub fn to_string(v: &Value) -> String { todo!() }
pub fn to_string_minified(v: &Value) -> String { todo!() }
pub fn to_string_with_options(v: &Value, _opts: PrintOptions) -> String { todo!() }
#[cfg(feature="std")]
pub fn to_writer<W: std::io::Write>(v: &Value, _w: W) -> Result<(), crate::Error> { todo!() }
