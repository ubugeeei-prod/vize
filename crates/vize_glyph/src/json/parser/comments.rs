//! Comment tokens and their adjoining blank-line trivia.

use super::Parser;
use crate::error::FormatError;
use crate::json::ast::Comment;
use crate::json::{json_error, trim_end};
use vize_l0::{String, cstr};

impl Parser<'_> {
    /// Parse a `//` or `/* */` comment. The leading `/` has not been consumed.
    pub(super) fn parse_comment(
        &mut self,
        own_line: bool,
        blank_line_before: bool,
    ) -> Result<Comment, FormatError> {
        self.advance(); // consume '/'
        match self.advance() {
            Some('/') => {
                let mut text = String::default();
                while let Some(c) = self.peek() {
                    if matches!(c, '\n' | '\r') {
                        break;
                    }
                    text.push(c);
                    self.advance();
                }
                Ok(Comment {
                    blank_line_before,
                    end: self.offset,
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
                                blank_line_before,
                                end: self.offset,
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
}
