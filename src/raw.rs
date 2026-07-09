use alloc::string::String;
#[derive(Clone, Debug, PartialEq)]
pub struct RawValue(pub String);
impl RawValue {
    pub fn new(s: impl Into<String>) -> Self {
        RawValue(s.into())
    }
}
