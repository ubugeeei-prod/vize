//! Recursive-descent parser for JSON / JSONC into the [`super::ast`] value tree.

use super::ast::{Comment, Element, Member, Node, split_trailing};
use super::{json_error, trim_end};
use crate::error::FormatError;
use vize_carton::{String, cstr};

pub(super) struct Parser<'a> {
    iter: std::iter::Peekable<std::str::Chars<'a>>,
    jsonc: bool,
}

impl<'a> Parser<'a> {
    pub(super) fn new(source: &'a str, jsonc: bool) -> Self {
        Self {
            iter: source.chars().peekable(),
            jsonc,
        }
    }

    pub(super) fn peek(&mut self) -> Option<char> {
        self.iter.peek().copied()
    }

    fn advance(&mut self) -> Option<char> {
        self.iter.next()
    }

    /// Skip whitespace, returning whether at least one newline was consumed.
    pub(super) fn skip_whitespace(&mut self) -> bool {
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
    pub(super) fn collect_comments(&mut self) -> Result<Vec<Comment>, FormatError> {
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

    pub(super) fn parse_value(&mut self) -> Result<Node, FormatError> {
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
