//! JSON formatting for non-SFC sources (e.g. `package.json`, `tsconfig.json`).
//!
//! Adds the smallest first step toward replacing Prettier on project config
//! files: tokenize the source character-by-character and re-emit it with the
//! indent/newline configured in `FormatOptions`. Because the tokenizer reads
//! keys sequentially and never inserts them into a sorted collection, object
//! key order is preserved exactly. JSONC (JSON-with-comments) is **not**
//! supported in this pass — see #2249 for the follow-up.

use crate::error::FormatError;
use crate::options::FormatOptions;
use vize_carton::{String, ToCompactString, cstr};

/// Format a JSON source string.
///
/// The output ends with the configured line terminator so it round-trips
/// through `vize fmt --check` (idempotent) and matches the convention used by
/// every other formatter path.
pub fn format_json_source(source: &str, options: &FormatOptions) -> Result<String, FormatError> {
    let trimmed = source.trim();
    if trimmed.is_empty() {
        return Ok(String::default());
    }

    let newline = options.newline_string();
    let indent = options.indent_string();

    let mut output = String::with_capacity(source.len() + 32);
    let mut scanner = Scanner::new(trimmed);

    scanner.format_value(&mut output, 0, indent.as_str(), newline)?;
    scanner.skip_whitespace();

    if scanner.peek().is_some() {
        return Err(FormatError::JsonFormatError(
            "trailing content after JSON value".to_compact_string(),
        ));
    }

    output.push_str(newline);
    Ok(output)
}

// ---------------------------------------------------------------------------
// Streaming tokenizer / reformatter
// ---------------------------------------------------------------------------

struct Scanner<'a> {
    iter: std::iter::Peekable<std::str::Chars<'a>>,
}

impl<'a> Scanner<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            iter: source.chars().peekable(),
        }
    }

    fn peek(&mut self) -> Option<char> {
        self.iter.peek().copied()
    }

    fn advance(&mut self) -> Option<char> {
        self.iter.next()
    }

    fn skip_whitespace(&mut self) {
        while matches!(self.peek(), Some(' ' | '\t' | '\n' | '\r')) {
            self.advance();
        }
    }

    fn format_value(
        &mut self,
        output: &mut String,
        depth: usize,
        indent: &str,
        newline: &str,
    ) -> Result<(), FormatError> {
        self.skip_whitespace();
        match self.peek() {
            Some('{') => self.format_object(output, depth, indent, newline),
            Some('[') => self.format_array(output, depth, indent, newline),
            Some('"') => self.scan_string(output),
            Some('t') => self.expect_keyword(output, "true"),
            Some('f') => self.expect_keyword(output, "false"),
            Some('n') => self.expect_keyword(output, "null"),
            Some('-') | Some('0'..='9') => self.scan_number(output),
            Some(c) => Err(json_error(cstr!("unexpected character '{c}'"))),
            None => Err(json_error("unexpected end of input")),
        }
    }

    fn format_object(
        &mut self,
        output: &mut String,
        depth: usize,
        indent: &str,
        newline: &str,
    ) -> Result<(), FormatError> {
        self.advance(); // consume '{'
        self.skip_whitespace();

        if self.peek() == Some('}') {
            self.advance();
            output.push_str("{}");
            return Ok(());
        }

        output.push('{');

        loop {
            output.push_str(newline);
            write_indent(output, depth + 1, indent);

            // key
            self.skip_whitespace();
            if self.peek() != Some('"') {
                return Err(json_error("expected string key in object"));
            }
            self.scan_string(output)?;

            // colon
            self.skip_whitespace();
            match self.advance() {
                Some(':') => output.push_str(": "),
                got => return Err(json_error(cstr!("expected ':', got {got:?}"))),
            }

            // value
            self.format_value(output, depth + 1, indent, newline)?;

            // ',' or '}'
            self.skip_whitespace();
            match self.peek() {
                Some(',') => {
                    self.advance();
                    output.push(',');
                    // continue to next key-value pair
                }
                Some('}') => {
                    self.advance();
                    output.push_str(newline);
                    write_indent(output, depth, indent);
                    output.push('}');
                    return Ok(());
                }
                got => return Err(json_error(cstr!("expected ',' or '}}', got {got:?}"))),
            }
        }
    }

    fn format_array(
        &mut self,
        output: &mut String,
        depth: usize,
        indent: &str,
        newline: &str,
    ) -> Result<(), FormatError> {
        self.advance(); // consume '['
        self.skip_whitespace();

        if self.peek() == Some(']') {
            self.advance();
            output.push_str("[]");
            return Ok(());
        }

        output.push('[');

        loop {
            output.push_str(newline);
            write_indent(output, depth + 1, indent);

            self.format_value(output, depth + 1, indent, newline)?;

            // ',' or ']'
            self.skip_whitespace();
            match self.peek() {
                Some(',') => {
                    self.advance();
                    output.push(',');
                    // continue to next element
                }
                Some(']') => {
                    self.advance();
                    output.push_str(newline);
                    write_indent(output, depth, indent);
                    output.push(']');
                    return Ok(());
                }
                got => return Err(json_error(cstr!("expected ',' or ']', got {got:?}"))),
            }
        }
    }

    /// Copy a JSON string from the source to `output`, verbatim (including
    /// escape sequences). The opening `"` has not yet been consumed.
    fn scan_string(&mut self, output: &mut String) -> Result<(), FormatError> {
        self.advance(); // consume '"'
        output.push('"');

        loop {
            match self.advance() {
                None => return Err(json_error("unterminated string")),
                Some('"') => {
                    output.push('"');
                    return Ok(());
                }
                Some('\\') => {
                    output.push('\\');
                    match self.advance() {
                        None => return Err(json_error("unterminated escape in string")),
                        Some('u') => {
                            output.push('u');
                            for _ in 0..4 {
                                match self.advance() {
                                    Some(c) if c.is_ascii_hexdigit() => output.push(c),
                                    Some(c) => {
                                        return Err(json_error(cstr!(
                                            "invalid hex digit '{c}' in \\u escape"
                                        )));
                                    }
                                    None => return Err(json_error("truncated \\u escape")),
                                }
                            }
                        }
                        Some(c) => output.push(c),
                    }
                }
                Some(c) if (c as u32) < 0x20 => {
                    return Err(json_error("unescaped control character in string"));
                }
                Some(c) => output.push(c),
            }
        }
    }

    /// Scan a JSON number and copy it verbatim to `output`.
    ///
    /// JSON numbers are: `-? (0 | [1-9][0-9]*) (. [0-9]+)? ([eE] [+-]? [0-9]+)?`
    /// Since we only reach this after the leading `-` or digit is confirmed,
    /// we consume greedily until the next non-number character.
    fn scan_number(&mut self, output: &mut String) -> Result<(), FormatError> {
        while let Some(c @ ('0'..='9' | '-' | '+' | '.' | 'e' | 'E')) = self.peek() {
            output.push(c);
            self.advance();
        }
        Ok(())
    }

    /// Consume and emit an exact keyword (`true`, `false`, `null`).
    fn expect_keyword(&mut self, output: &mut String, kw: &str) -> Result<(), FormatError> {
        for expected in kw.chars() {
            match self.advance() {
                Some(c) if c == expected => output.push(c),
                Some(c) => {
                    return Err(json_error(cstr!(
                        "expected keyword '{kw}', got unexpected char '{c}'"
                    )));
                }
                None => {
                    return Err(json_error(cstr!(
                        "expected keyword '{kw}', got end of input"
                    )));
                }
            }
        }
        Ok(())
    }
}

fn write_indent(output: &mut String, depth: usize, indent: &str) {
    for _ in 0..depth {
        output.push_str(indent);
    }
}

fn json_error(msg: impl Into<String>) -> FormatError {
    FormatError::JsonFormatError(msg.into())
}

#[cfg(test)]
mod tests {
    use super::{FormatOptions, format_json_source};

    fn opts() -> FormatOptions {
        FormatOptions::default()
    }

    #[test]
    fn pretty_prints_minified_object() {
        let source = r#"{"name":"vize","version":"0.259.0","keywords":["vue","toolchain"]}"#;
        let result = format_json_source(source, &opts()).unwrap();
        assert_eq!(
            result.as_str(),
            "{\n  \"name\": \"vize\",\n  \"version\": \"0.259.0\",\n  \"keywords\": [\n    \"vue\",\n    \"toolchain\"\n  ]\n}\n",
        );
    }

    #[test]
    fn preserves_key_order_from_source() {
        let source = r#"{"z":1,"a":2,"m":3}"#;
        let result = format_json_source(source, &opts()).unwrap();
        assert_eq!(
            result.as_str(),
            "{\n  \"z\": 1,\n  \"a\": 2,\n  \"m\": 3\n}\n"
        );
    }

    #[test]
    fn already_formatted_is_idempotent() {
        let source = "{\n  \"a\": 1,\n  \"b\": [\n    true,\n    null\n  ]\n}\n";
        let first = format_json_source(source, &opts()).unwrap();
        let second = format_json_source(first.as_str(), &opts()).unwrap();
        assert_eq!(first.as_str(), second.as_str());
    }

    #[test]
    fn empty_collections_stay_compact() {
        let result = format_json_source(r#"{"a":[],"b":{}}"#, &opts()).unwrap();
        assert_eq!(result.as_str(), "{\n  \"a\": [],\n  \"b\": {}\n}\n");
    }

    #[test]
    fn empty_input_yields_empty_output() {
        assert!(format_json_source("", &opts()).unwrap().is_empty());
        assert!(format_json_source("   \n\t  ", &opts()).unwrap().is_empty());
    }

    #[test]
    fn escapes_required_string_characters() {
        let source = r#"{"k":"line\nbreak\t\"quoted\""}"#;
        let result = format_json_source(source, &opts()).unwrap();
        assert!(result.contains(r#""line\nbreak\t\"quoted\"""#));
    }

    #[test]
    fn invalid_json_returns_error() {
        assert!(format_json_source("{\"a\":}", &opts()).is_err());
    }

    #[test]
    fn honors_custom_indent_width() {
        let mut options = opts();
        options.tab_width = 4;
        let result = format_json_source(r#"{"a":1}"#, &options).unwrap();
        assert_eq!(result.as_str(), "{\n    \"a\": 1\n}\n");
    }
}
