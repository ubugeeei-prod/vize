//! JSON / JSONC formatting for non-SFC sources (e.g. `package.json`,
//! `tsconfig.json`).
//!
//! This is the formatter path that lets a project replace Prettier on its
//! project config files. Two entry points share one parser + printer:
//!
//! - [`format_json_source`] — strict JSON. Comments and trailing commas are
//!   errors, exactly as before.
//! - [`format_jsonc_source`] — JSON-with-comments (`.jsonc`, and the comment /
//!   trailing-comma dialect TypeScript accepts in `tsconfig.json`). Comments
//!   are preserved and trailing commas are tolerated on input and dropped on
//!   output (#2249, the follow-up to the strict-JSON pass).
//!
//! Both paths parse into a small value tree, then print it. Object key order
//! and scalar token text are preserved verbatim; only structural whitespace is
//! rewritten to the indent/newline configured in [`FormatOptions`]. The output
//! is idempotent: formatting already-formatted source is a no-op.

use crate::error::FormatError;
use crate::options::FormatOptions;
use vize_carton::{String, cstr};

/// Format a strict JSON source string.
///
/// Comments and trailing commas are rejected. The output ends with the
/// configured line terminator so it round-trips through `vize fmt --check`.
pub fn format_json_source(source: &str, options: &FormatOptions) -> Result<String, FormatError> {
    format_document(source, options, false)
}

/// Format a JSONC source string (JSON with `//` and `/* */` comments and
/// optional trailing commas).
///
/// Comments are preserved in source order. A comment that trails a value on the
/// same source line stays a trailing comment; a comment on its own line stays on
/// its own line. Trailing commas are accepted on input and removed on output.
pub fn format_jsonc_source(source: &str, options: &FormatOptions) -> Result<String, FormatError> {
    format_document(source, options, true)
}

fn format_document(
    source: &str,
    options: &FormatOptions,
    jsonc: bool,
) -> Result<String, FormatError> {
    if source.trim().is_empty() {
        return Ok(String::default());
    }

    let mut parser = Parser::new(source, jsonc);
    let leading = parser.collect_comments()?;
    let value = parser.parse_value()?;
    let trailing = parser.collect_comments()?;

    parser.skip_whitespace();
    if parser.peek().is_some() {
        return Err(json_error("trailing content after JSON value"));
    }

    let newline = options.newline_string();
    let indent = options.indent_string();
    let printer = Printer {
        indent: indent.as_str(),
        newline,
    };

    let mut output = String::with_capacity(source.len() + 32);
    for comment in &leading {
        printer.write_comment(&mut output, comment);
        output.push_str(newline);
    }
    printer.write_value(&mut output, &value, 0);
    for comment in &trailing {
        output.push_str(newline);
        printer.write_comment(&mut output, comment);
    }
    output.push_str(newline);
    Ok(output)
}

// ---------------------------------------------------------------------------
// Value tree
// ---------------------------------------------------------------------------

enum Node {
    /// `{}` (compact when empty) or a multi-line mapping.
    Object {
        members: Vec<Member>,
        /// Comments on their own line after the last member, before `}`.
        dangling: Vec<Comment>,
    },
    /// `[]` (compact when empty) or a multi-line sequence.
    Array {
        elements: Vec<Element>,
        dangling: Vec<Comment>,
    },
    /// A string, number, `true`, `false`, or `null`, copied verbatim.
    Scalar(String),
}

struct Member {
    /// Comments printed on their own lines before the key.
    leading: Vec<Comment>,
    /// The key, including its surrounding quotes, verbatim.
    key: String,
    value: Node,
    /// Comments printed on the same line after the value (and comma).
    trailing: Vec<Comment>,
}

struct Element {
    leading: Vec<Comment>,
    value: Node,
    trailing: Vec<Comment>,
}

struct Comment {
    /// `true` for `/* ... */`, `false` for `// ...`.
    block: bool,
    /// The text between the comment markers, verbatim (line comments are
    /// trimmed at the end so reformatting does not leave trailing whitespace).
    text: String,
    /// Whether a newline separated this comment from the previous token. Used to
    /// decide whether a comment trails a value or belongs on its own line.
    own_line: bool,
}

// ---------------------------------------------------------------------------
// Parser
// ---------------------------------------------------------------------------

struct Parser<'a> {
    iter: std::iter::Peekable<std::str::Chars<'a>>,
    jsonc: bool,
}

impl<'a> Parser<'a> {
    fn new(source: &'a str, jsonc: bool) -> Self {
        Self {
            iter: source.chars().peekable(),
            jsonc,
        }
    }

    fn peek(&mut self) -> Option<char> {
        self.iter.peek().copied()
    }

    fn advance(&mut self) -> Option<char> {
        self.iter.next()
    }

    /// Skip whitespace, returning whether at least one newline was consumed.
    fn skip_whitespace(&mut self) -> bool {
        let mut saw_newline = false;
        while let Some(c) = self.peek() {
            match c {
                '\n' => {
                    saw_newline = true;
                    self.advance();
                }
                ' ' | '\t' | '\r' => {
                    self.advance();
                }
                _ => break,
            }
        }
        saw_newline
    }

    /// Skip whitespace and, in JSONC mode, collect any comments encountered.
    /// In strict JSON mode this only skips whitespace and always returns an
    /// empty list, so a stray `/` is left for the caller to reject.
    fn collect_comments(&mut self) -> Result<Vec<Comment>, FormatError> {
        let mut comments = Vec::new();
        loop {
            let saw_newline = self.skip_whitespace();
            if !self.jsonc || self.peek() != Some('/') {
                break;
            }
            comments.push(self.parse_comment(saw_newline)?);
        }
        Ok(comments)
    }

    /// Parse a `//` or `/* */` comment. The leading `/` has not been consumed.
    fn parse_comment(&mut self, own_line: bool) -> Result<Comment, FormatError> {
        self.advance(); // consume '/'
        match self.advance() {
            Some('/') => {
                let mut text = String::default();
                while let Some(c) = self.peek() {
                    if c == '\n' {
                        break;
                    }
                    text.push(c);
                    self.advance();
                }
                Ok(Comment {
                    block: false,
                    text: trim_end(&text),
                    own_line,
                })
            }
            Some('*') => {
                let mut text = String::default();
                loop {
                    match self.advance() {
                        Some('*') if self.peek() == Some('/') => {
                            self.advance(); // consume '/'
                            return Ok(Comment {
                                block: true,
                                text,
                                own_line,
                            });
                        }
                        Some(c) => text.push(c),
                        None => return Err(json_error("unterminated block comment")),
                    }
                }
            }
            Some(c) => Err(json_error(cstr!("unexpected character '{c}' after '/'"))),
            None => Err(json_error("unexpected end of input after '/'")),
        }
    }

    fn parse_value(&mut self) -> Result<Node, FormatError> {
        self.skip_whitespace();
        match self.peek() {
            Some('{') => self.parse_object(),
            Some('[') => self.parse_array(),
            Some('"') => Ok(Node::Scalar(self.parse_string()?)),
            Some('t') => Ok(Node::Scalar(self.parse_keyword("true")?)),
            Some('f') => Ok(Node::Scalar(self.parse_keyword("false")?)),
            Some('n') => Ok(Node::Scalar(self.parse_keyword("null")?)),
            Some('-' | '0'..='9') => Ok(Node::Scalar(self.parse_number())),
            Some(c) => Err(json_error(cstr!("unexpected character '{c}'"))),
            None => Err(json_error("unexpected end of input")),
        }
    }

    fn parse_object(&mut self) -> Result<Node, FormatError> {
        self.advance(); // consume '{'
        let mut members = Vec::new();
        let mut carry: Vec<Comment> = Vec::new();
        let mut after_comma = false;

        loop {
            let mut leading = std::mem::take(&mut carry);
            leading.extend(self.collect_comments()?);

            match self.peek() {
                Some('}') => {
                    if after_comma && !self.jsonc {
                        return Err(json_error("trailing comma in object"));
                    }
                    self.advance();
                    return Ok(Node::Object {
                        members,
                        dangling: leading,
                    });
                }
                Some('"') => {
                    let key = self.parse_string()?;
                    leading.extend(self.collect_comments()?); // between key and ':'
                    match self.advance() {
                        Some(':') => {}
                        got => return Err(json_error(cstr!("expected ':', got {got:?}"))),
                    }
                    leading.extend(self.collect_comments()?); // between ':' and value
                    let value = self.parse_value()?;

                    let (mut trailing, mut spill) = split_trailing(self.collect_comments()?);
                    match self.peek() {
                        Some(',') => {
                            after_comma = true;
                            self.advance();
                            if trailing.is_empty() {
                                let (post_trailing, post_spill) =
                                    split_trailing(self.collect_comments()?);
                                trailing = post_trailing;
                                spill.extend(post_spill);
                            }
                            carry = spill;
                            members.push(Member {
                                leading,
                                key,
                                value,
                                trailing,
                            });
                        }
                        Some('}') => {
                            self.advance();
                            members.push(Member {
                                leading,
                                key,
                                value,
                                trailing,
                            });
                            return Ok(Node::Object {
                                members,
                                dangling: spill,
                            });
                        }
                        got => {
                            return Err(json_error(cstr!("expected ',' or '}}', got {got:?}")));
                        }
                    }
                }
                Some(c) => return Err(json_error(cstr!("unexpected character '{c}' in object"))),
                None => return Err(json_error("unterminated object")),
            }
        }
    }

    fn parse_array(&mut self) -> Result<Node, FormatError> {
        self.advance(); // consume '['
        let mut elements = Vec::new();
        let mut carry: Vec<Comment> = Vec::new();
        let mut after_comma = false;

        loop {
            let mut leading = std::mem::take(&mut carry);
            leading.extend(self.collect_comments()?);

            match self.peek() {
                Some(']') => {
                    if after_comma && !self.jsonc {
                        return Err(json_error("trailing comma in array"));
                    }
                    self.advance();
                    return Ok(Node::Array {
                        elements,
                        dangling: leading,
                    });
                }
                None => return Err(json_error("unterminated array")),
                _ => {
                    let value = self.parse_value()?;

                    let (mut trailing, mut spill) = split_trailing(self.collect_comments()?);
                    match self.peek() {
                        Some(',') => {
                            after_comma = true;
                            self.advance();
                            if trailing.is_empty() {
                                let (post_trailing, post_spill) =
                                    split_trailing(self.collect_comments()?);
                                trailing = post_trailing;
                                spill.extend(post_spill);
                            }
                            carry = spill;
                            elements.push(Element {
                                leading,
                                value,
                                trailing,
                            });
                        }
                        Some(']') => {
                            self.advance();
                            elements.push(Element {
                                leading,
                                value,
                                trailing,
                            });
                            return Ok(Node::Array {
                                elements,
                                dangling: spill,
                            });
                        }
                        got => {
                            return Err(json_error(cstr!("expected ',' or ']', got {got:?}")));
                        }
                    }
                }
            }
        }
    }

    /// Copy a JSON string verbatim (including escape sequences and the
    /// surrounding quotes). The opening `"` has not yet been consumed.
    fn parse_string(&mut self) -> Result<String, FormatError> {
        let mut out = String::default();
        self.advance(); // consume '"'
        out.push('"');

        loop {
            match self.advance() {
                None => return Err(json_error("unterminated string")),
                Some('"') => {
                    out.push('"');
                    return Ok(out);
                }
                Some('\\') => {
                    out.push('\\');
                    match self.advance() {
                        None => return Err(json_error("unterminated escape in string")),
                        Some('u') => {
                            out.push('u');
                            for _ in 0..4 {
                                match self.advance() {
                                    Some(c) if c.is_ascii_hexdigit() => out.push(c),
                                    Some(c) => {
                                        return Err(json_error(cstr!(
                                            "invalid hex digit '{c}' in \\u escape"
                                        )));
                                    }
                                    None => return Err(json_error("truncated \\u escape")),
                                }
                            }
                        }
                        Some(c) => out.push(c),
                    }
                }
                Some(c) if (c as u32) < 0x20 => {
                    return Err(json_error("unescaped control character in string"));
                }
                Some(c) => out.push(c),
            }
        }
    }

    /// Scan a JSON number and copy it verbatim.
    ///
    /// JSON numbers are `-? (0 | [1-9][0-9]*) (. [0-9]+)? ([eE] [+-]? [0-9]+)?`.
    /// We only reach this after the leading `-` or digit is confirmed, so we
    /// consume greedily until the next non-number character.
    fn parse_number(&mut self) -> String {
        let mut out = String::default();
        while let Some(c @ ('0'..='9' | '-' | '+' | '.' | 'e' | 'E')) = self.peek() {
            out.push(c);
            self.advance();
        }
        out
    }

    /// Consume and return an exact keyword (`true`, `false`, `null`).
    fn parse_keyword(&mut self, kw: &str) -> Result<String, FormatError> {
        for expected in kw.chars() {
            match self.advance() {
                Some(c) if c == expected => {}
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
        Ok(String::from(kw))
    }
}

/// Split comments collected after a value into the run that trails the value on
/// the same line and the remaining comments that belong on their own lines.
///
/// The trailing run starts only if the first comment shares the value's line. A
/// `//` line comment ends the run (anything after it is on a later line), while
/// `/* */` block comments can chain on one line.
fn split_trailing(mut comments: Vec<Comment>) -> (Vec<Comment>, Vec<Comment>) {
    if comments.first().is_none_or(|c| c.own_line) {
        return (Vec::new(), comments);
    }

    let mut cut = 0;
    for (i, comment) in comments.iter().enumerate() {
        if i > 0 && comment.own_line {
            break;
        }
        cut = i + 1;
        if !comment.block {
            break; // a line comment runs to end of line
        }
    }

    let spill = comments.split_off(cut);
    (comments, spill)
}

fn trim_end(text: &str) -> String {
    String::from(text.trim_end())
}

fn json_error(msg: impl Into<String>) -> FormatError {
    FormatError::JsonFormatError(msg.into())
}

// ---------------------------------------------------------------------------
// Printer
// ---------------------------------------------------------------------------

struct Printer<'a> {
    indent: &'a str,
    newline: &'a str,
}

impl Printer<'_> {
    fn write_indent(&self, output: &mut String, depth: usize) {
        for _ in 0..depth {
            output.push_str(self.indent);
        }
    }

    fn write_comment(&self, output: &mut String, comment: &Comment) {
        if comment.block {
            output.push_str("/*");
            output.push_str(comment.text.as_str());
            output.push_str("*/");
        } else {
            output.push_str("//");
            output.push_str(comment.text.as_str());
        }
    }

    fn write_value(&self, output: &mut String, node: &Node, depth: usize) {
        match node {
            Node::Scalar(text) => output.push_str(text.as_str()),
            Node::Object { members, dangling } => {
                self.write_block(
                    output,
                    depth,
                    ['{', '}'],
                    members.len(),
                    dangling,
                    |p, out| {
                        for (index, member) in members.iter().enumerate() {
                            p.write_leading(out, &member.leading, depth + 1);
                            out.push_str(p.newline);
                            p.write_indent(out, depth + 1);
                            out.push_str(member.key.as_str());
                            out.push_str(": ");
                            p.write_value(out, &member.value, depth + 1);
                            if index + 1 < members.len() {
                                out.push(',');
                            }
                            p.write_trailing(out, &member.trailing);
                        }
                    },
                );
            }
            Node::Array { elements, dangling } => {
                self.write_block(
                    output,
                    depth,
                    ['[', ']'],
                    elements.len(),
                    dangling,
                    |p, out| {
                        for (index, element) in elements.iter().enumerate() {
                            p.write_leading(out, &element.leading, depth + 1);
                            out.push_str(p.newline);
                            p.write_indent(out, depth + 1);
                            p.write_value(out, &element.value, depth + 1);
                            if index + 1 < elements.len() {
                                out.push(',');
                            }
                            p.write_trailing(out, &element.trailing);
                        }
                    },
                );
            }
        }
    }

    fn write_block(
        &self,
        output: &mut String,
        depth: usize,
        delims: [char; 2],
        item_count: usize,
        dangling: &[Comment],
        write_items: impl FnOnce(&Self, &mut String),
    ) {
        let [open, close] = delims;
        if item_count == 0 && dangling.is_empty() {
            output.push(open);
            output.push(close);
            return;
        }
        output.push(open);
        write_items(self, output);
        for comment in dangling {
            output.push_str(self.newline);
            self.write_indent(output, depth + 1);
            self.write_comment(output, comment);
        }
        output.push_str(self.newline);
        self.write_indent(output, depth);
        output.push(close);
    }

    fn write_leading(&self, output: &mut String, comments: &[Comment], depth: usize) {
        for comment in comments {
            output.push_str(self.newline);
            self.write_indent(output, depth);
            self.write_comment(output, comment);
        }
    }

    fn write_trailing(&self, output: &mut String, comments: &[Comment]) {
        for comment in comments {
            output.push(' ');
            self.write_comment(output, comment);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{FormatOptions, format_json_source, format_jsonc_source};

    fn opts() -> FormatOptions {
        FormatOptions::default()
    }

    // -- strict JSON (unchanged behaviour) ---------------------------------

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

    #[test]
    fn strict_json_rejects_comments_and_trailing_commas() {
        assert!(format_json_source("{\n  // hi\n  \"a\": 1\n}", &opts()).is_err());
        assert!(format_json_source(r#"{"a":1,}"#, &opts()).is_err());
        assert!(format_json_source("[1,2,]", &opts()).is_err());
    }

    // -- JSONC -------------------------------------------------------------

    #[test]
    fn jsonc_without_comments_matches_strict_json() {
        for source in [
            r#"{"name":"vize","keywords":["vue","toolchain"],"nested":{"a":[1,2]}}"#,
            r#"{"a":[],"b":{}}"#,
            r#"[1,2,3]"#,
            r#""scalar""#,
        ] {
            let json = format_json_source(source, &opts()).unwrap();
            let jsonc = format_jsonc_source(source, &opts()).unwrap();
            assert_eq!(json.as_str(), jsonc.as_str(), "diverged for {source}");
        }
    }

    #[test]
    fn jsonc_keeps_leading_line_comment_on_member() {
        let source = "{\n  // the package name\n  \"name\": \"vize\"\n}\n";
        let result = format_jsonc_source(source, &opts()).unwrap();
        assert_eq!(
            result.as_str(),
            "{\n  // the package name\n  \"name\": \"vize\"\n}\n"
        );
    }

    #[test]
    fn jsonc_keeps_trailing_line_comment_after_comma() {
        let source = "{\n  \"a\": 1, // first\n  \"b\": 2 // last\n}\n";
        let result = format_jsonc_source(source, &opts()).unwrap();
        assert_eq!(
            result.as_str(),
            "{\n  \"a\": 1, // first\n  \"b\": 2 // last\n}\n"
        );
    }

    #[test]
    fn jsonc_normalizes_indentation_but_keeps_comments() {
        let source = "{\n// compilerOptions\n\"compilerOptions\":{\n\"strict\":true, // be strict\n\"target\":\"ES2022\"\n}\n}";
        let result = format_jsonc_source(source, &opts()).unwrap();
        assert_eq!(
            result.as_str(),
            "{\n  // compilerOptions\n  \"compilerOptions\": {\n    \"strict\": true, // be strict\n    \"target\": \"ES2022\"\n  }\n}\n"
        );
    }

    #[test]
    fn jsonc_drops_trailing_comma() {
        let source = "{\n  \"a\": 1,\n  \"b\": [\n    1,\n    2,\n  ],\n}\n";
        let result = format_jsonc_source(source, &opts()).unwrap();
        assert_eq!(
            result.as_str(),
            "{\n  \"a\": 1,\n  \"b\": [\n    1,\n    2\n  ]\n}\n"
        );
    }

    #[test]
    fn jsonc_keeps_dangling_comment_before_close() {
        // A comma whose only follower is an own-line comment then `}` is a
        // genuine trailing comma: the comment is preserved, the comma dropped.
        let source = "{\n  \"a\": 1,\n  // nothing else yet\n}\n";
        let result = format_jsonc_source(source, &opts()).unwrap();
        assert_eq!(result.as_str(), "{\n  \"a\": 1\n  // nothing else yet\n}\n");
    }

    #[test]
    fn jsonc_keeps_block_comment() {
        let source = "{ /* header */ \"a\": 1 }";
        let result = format_jsonc_source(source, &opts()).unwrap();
        assert_eq!(result.as_str(), "{\n  /* header */\n  \"a\": 1\n}\n");
    }

    #[test]
    fn jsonc_keeps_leading_file_comment() {
        let source = "// vize config\n{\n  \"a\": 1\n}\n";
        let result = format_jsonc_source(source, &opts()).unwrap();
        assert_eq!(result.as_str(), "// vize config\n{\n  \"a\": 1\n}\n");
    }

    #[test]
    fn jsonc_is_idempotent_across_comment_positions() {
        let source = "// top\n{\n  // lead a\n  \"a\": 1, // trail a\n  \"b\": [\n    // lead 0\n    10,\n    20, // trail 1\n  ],\n  /* block */ \"c\": true,\n  // dangling\n}\n";
        let first = format_jsonc_source(source, &opts()).unwrap();
        let second = format_jsonc_source(first.as_str(), &opts()).unwrap();
        assert_eq!(first.as_str(), second.as_str(), "first pass:\n{first}");
    }

    #[test]
    fn jsonc_unterminated_block_comment_errors() {
        assert!(format_jsonc_source("{ /* oops \n \"a\": 1 }", &opts()).is_err());
    }
}
