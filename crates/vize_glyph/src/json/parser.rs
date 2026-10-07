//! Recursive-descent parser for JSON / JSONC into the [`super::ast`] value tree.

use super::ast::{Comment, Element, Member, Node, has_blank_line, split_trailing};
use super::{json_error, number};

mod comments;
use crate::error::FormatError;
use vize_l0::{String, cstr};

pub(super) struct Parser<'a> {
    source: &'a str,
    offset: usize,
    iter: std::iter::Peekable<std::str::Chars<'a>>,
    jsonc: bool,
}

impl<'a> Parser<'a> {
    pub(super) fn new(source: &'a str, jsonc: bool) -> Self {
        Self {
            source,
            offset: 0,
            iter: source.chars().peekable(),
            jsonc,
        }
    }

    pub(super) fn peek(&mut self) -> Option<char> {
        self.iter.peek().copied()
    }

    fn advance(&mut self) -> Option<char> {
        let character = self.iter.next()?;
        self.offset += character.len_utf8();
        Some(character)
    }

    /// Skip whitespace, returning whether at least one newline was consumed.
    pub(super) fn skip_whitespace(&mut self) -> bool {
        let mut saw_newline = false;
        while let Some(c) = self.peek() {
            match c {
                '\n' | '\r' => {
                    saw_newline = true;
                    self.advance();
                }
                ' ' | '\t' => {
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
            let start = self.offset;
            let saw_newline = self.skip_whitespace();
            if !self.jsonc || self.peek() != Some('/') {
                break;
            }
            let blank_line_before = self.blank_line_since(Some(start));
            comments.push(self.parse_comment(saw_newline, blank_line_before)?);
        }
        Ok(comments)
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
            Some('-' | '0'..='9') => {
                let number = number::parse(&mut self.iter)?;
                self.offset += number.len();
                Ok(Node::Scalar(number))
            }
            Some(c) => Err(json_error(cstr!("unexpected character '{c}'"))),
            None => Err(json_error("unexpected end of input")),
        }
    }

    fn parse_object(&mut self) -> Result<Node, FormatError> {
        self.advance(); // consume '{'
        let expanded = self.skip_whitespace();
        let mut members = Vec::new();
        let mut previous_end = None;
        let mut carry: Vec<Comment> = Vec::new();
        let mut after_comma = false;

        loop {
            let mut leading = std::mem::take(&mut carry);
            leading.extend(self.collect_comments()?);
            let blank_line_before =
                self.blank_line_since(leading.last().map(|c| c.end).or(previous_end));

            match self.peek() {
                Some('}') => {
                    if after_comma && !self.jsonc {
                        return Err(json_error("trailing comma in object"));
                    }
                    self.advance();
                    return Ok(Node::Object {
                        members,
                        expanded,
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
                    previous_end = Some(self.offset);

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
                                blank_line_before,
                                leading,
                                key,
                                value,
                                trailing,
                            });
                        }
                        Some('}') => {
                            self.advance();
                            members.push(Member {
                                blank_line_before,
                                leading,
                                key,
                                value,
                                trailing,
                            });
                            return Ok(Node::Object {
                                members,
                                expanded,
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
        let mut previous_end = None;
        let mut carry: Vec<Comment> = Vec::new();
        let mut after_comma = false;

        loop {
            let mut leading = std::mem::take(&mut carry);
            leading.extend(self.collect_comments()?);
            let blank_line_before =
                self.blank_line_since(leading.last().map(|c| c.end).or(previous_end));

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
                    previous_end = Some(self.offset);

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
                                blank_line_before,
                                leading,
                                value,
                                trailing,
                            });
                        }
                        Some(']') => {
                            self.advance();
                            elements.push(Element {
                                blank_line_before,
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

    fn blank_line_since(&self, previous_end: Option<usize>) -> bool {
        previous_end
            .and_then(|start| self.source.get(start..self.offset))
            .is_some_and(has_blank_line)
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
                        Some(c @ ('"' | '\\' | '/' | 'b' | 'f' | 'n' | 'r' | 't')) => {
                            out.push(c);
                        }
                        Some(c) => {
                            return Err(json_error(cstr!("invalid escape '\\{c}' in string")));
                        }
                    }
                }
                Some(c) if (c as u32) < 0x20 => {
                    return Err(json_error("unescaped control character in string"));
                }
                Some(c) => out.push(c),
            }
        }
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
