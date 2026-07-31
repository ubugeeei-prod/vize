//! Comment lexer for SFC style block contents.

use super::token::ends_with_unescaped_backslash;
use super::{CommentMarkers, record_markers};

#[derive(Clone, Copy, PartialEq, Eq)]
enum StyleContext {
    Code,
    SingleQuote,
    DoubleQuote,
    BlockComment,
    LineComment,
}

pub(crate) struct StyleDirectiveLexer {
    context: StyleContext,
    allow_line_comments: bool,
}

impl StyleDirectiveLexer {
    pub(crate) fn new(allow_line_comments: bool) -> Self {
        Self {
            context: StyleContext::Code,
            allow_line_comments,
        }
    }

    pub(crate) fn scan_line(&mut self, line: &str) -> CommentMarkers {
        let bytes = line.as_bytes();
        let mut markers = CommentMarkers::default();
        let mut index = 0;
        while index < bytes.len() {
            if matches!(
                self.context,
                StyleContext::BlockComment | StyleContext::LineComment
            ) {
                record_markers(bytes, index, &mut markers);
            }
            let current = bytes[index];
            let next = bytes.get(index + 1).copied();
            match self.context {
                StyleContext::Code => match (current, next) {
                    (b'/', Some(b'*')) => {
                        self.context = StyleContext::BlockComment;
                        index += 1;
                    }
                    (b'/', Some(b'/')) if self.allow_line_comments => {
                        self.context = StyleContext::LineComment;
                        index += 1;
                    }
                    (b'\'', _) => self.context = StyleContext::SingleQuote,
                    (b'"', _) => self.context = StyleContext::DoubleQuote,
                    _ => {}
                },
                StyleContext::SingleQuote => match (current, next) {
                    (b'\\', Some(_)) => index += 1,
                    (b'\'', _) => self.context = StyleContext::Code,
                    _ => {}
                },
                StyleContext::DoubleQuote => match (current, next) {
                    (b'\\', Some(_)) => index += 1,
                    (b'"', _) => self.context = StyleContext::Code,
                    _ => {}
                },
                StyleContext::BlockComment => {
                    if (current, next) == (b'*', Some(b'/')) {
                        self.context = StyleContext::Code;
                        index += 1;
                    }
                }
                StyleContext::LineComment => {}
            }
            index += 1;
        }
        if self.context == StyleContext::LineComment {
            self.context = StyleContext::Code;
        }
        if !ends_with_unescaped_backslash(bytes)
            && matches!(
                self.context,
                StyleContext::SingleQuote | StyleContext::DoubleQuote
            )
        {
            self.context = StyleContext::Code;
        }
        markers
    }
}
