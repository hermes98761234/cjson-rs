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
