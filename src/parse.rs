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
