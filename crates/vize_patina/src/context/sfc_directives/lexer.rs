//! Lightweight comment lexers for SFC script and style block contents.

mod token;

use token::{
    ends_with_unescaped_backslash, identifier_allows_expression, identifier_end,
    is_identifier_start, is_jsx_start,
};

#[derive(Default)]
pub(super) struct CommentMarkers {
    pub(super) eslint: Option<usize>,
    pub(super) vize: Option<usize>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ScriptContext {
    Code,
    Interpolation(u32),
    SingleQuote,
    DoubleQuote,
    Template,
    Regex(bool),
    JsxText,
    JsxTag { closing: bool },
    BlockComment,
    LineComment,
}

pub(super) struct DirectiveLexer {
    stack: Vec<ScriptContext>,
    jsx: bool,
    jsx_depth: u32,
    can_start_expression: bool,
    after_dot: bool,
}

impl Default for DirectiveLexer {
    fn default() -> Self {
        Self::new(false)
    }
}

impl DirectiveLexer {
    pub(super) fn new(jsx: bool) -> Self {
        Self {
            stack: vec![ScriptContext::Code],
            jsx,
            jsx_depth: 0,
            can_start_expression: true,
            after_dot: false,
        }
    }

    pub(super) fn scan_line(&mut self, line: &str) -> CommentMarkers {
        let bytes = line.as_bytes();
        let mut markers = CommentMarkers::default();
        let mut index = 0;
        while index < bytes.len() {
            let context = *self.stack.last().expect("lexer always has a root context");
            if matches!(
                context,
                ScriptContext::BlockComment | ScriptContext::LineComment
            ) {
                record_markers(bytes, index, &mut markers);
            }

            let current = bytes[index];
            let next = bytes.get(index + 1).copied();
            match context {
                ScriptContext::Code | ScriptContext::Interpolation(_) => match (current, next) {
                    (b'/', Some(b'/')) => {
                        self.stack.push(ScriptContext::LineComment);
                        index += 1;
                    }
                    (b'/', Some(b'*')) => {
                        self.stack.push(ScriptContext::BlockComment);
                        index += 1;
                    }
                    (b'/', _) if self.can_start_expression => {
                        self.stack.push(ScriptContext::Regex(false));
                    }
                    (b'<', _)
                        if self.jsx && self.can_start_expression && is_jsx_start(bytes, index) =>
                    {
                        self.jsx_depth += 1;
                        self.stack.push(ScriptContext::JsxText);
                        self.stack.push(ScriptContext::JsxTag { closing: false });
                        self.after_dot = false;
                    }
                    (b'\'', _) => self.stack.push(ScriptContext::SingleQuote),
                    (b'"', _) => self.stack.push(ScriptContext::DoubleQuote),
                    (b'`', _) => self.stack.push(ScriptContext::Template),
                    (b'{', _) => {
                        if let Some(ScriptContext::Interpolation(depth)) = self.stack.last_mut() {
                            *depth += 1;
                        }
                        self.can_start_expression = true;
                        self.after_dot = false;
                    }
                    (b'}', _) => {
                        if let Some(ScriptContext::Interpolation(depth)) = self.stack.last_mut() {
                            if *depth == 0 {
                                self.stack.pop();
                            } else {
                                *depth -= 1;
                            }
                        }
                        self.can_start_expression = false;
                        self.after_dot = false;
                    }
                    (byte, _) if is_identifier_start(byte) => {
                        let end = identifier_end(bytes, index);
                        self.can_start_expression =
                            !self.after_dot && identifier_allows_expression(&bytes[index..end]);
                        self.after_dot = false;
                        index = end - 1;
                    }
                    (b')' | b']' | b'0'..=b'9', _) => {
                        self.can_start_expression = false;
                        self.after_dot = false;
                    }
                    (b'.', _) => self.after_dot = true,
                    (
                        b'(' | b'[' | b'/' | b'=' | b':' | b',' | b'!' | b'?' | b';' | b'+' | b'-'
                        | b'*' | b'%' | b'&' | b'|' | b'^' | b'~' | b'<' | b'>',
                        _,
                    ) => {
                        self.can_start_expression = true;
                        self.after_dot = false;
                    }
                    _ => {}
                },
                ScriptContext::SingleQuote => match (current, next) {
                    (b'\\', Some(_)) => index += 1,
                    (b'\'', _) => {
                        self.stack.pop();
                        self.can_start_expression = false;
                    }
                    _ => {}
                },
                ScriptContext::DoubleQuote => match (current, next) {
                    (b'\\', Some(_)) => index += 1,
                    (b'"', _) => {
                        self.stack.pop();
                        self.can_start_expression = false;
                    }
                    _ => {}
                },
                ScriptContext::Template => match (current, next) {
                    (b'\\', Some(_)) => index += 1,
                    (b'`', _) => {
                        self.stack.pop();
                        self.can_start_expression = false;
                    }
                    (b'$', Some(b'{')) => {
                        self.stack.push(ScriptContext::Interpolation(0));
                        self.can_start_expression = true;
                        index += 1;
                    }
                    _ => {}
                },
                ScriptContext::Regex(in_class) => match (current, next) {
                    (b'\\', Some(_)) => index += 1,
                    (b'[', _) if !in_class => {
                        if let Some(ScriptContext::Regex(in_class)) = self.stack.last_mut() {
                            *in_class = true;
                        }
                    }
                    (b']', _) if in_class => {
                        if let Some(ScriptContext::Regex(in_class)) = self.stack.last_mut() {
                            *in_class = false;
                        }
                    }
                    (b'/', _) if !in_class => {
                        self.stack.pop();
                        self.can_start_expression = false;
                    }
                    _ => {}
                },
                ScriptContext::JsxText => match (current, next) {
                    (b'<', Some(b'/')) => {
                        self.stack.push(ScriptContext::JsxTag { closing: true });
                        index += 1;
                    }
                    (b'<', Some(next)) if next == b'>' || next.is_ascii_alphabetic() => {
                        self.jsx_depth += 1;
                        self.stack.push(ScriptContext::JsxTag { closing: false });
                    }
                    (b'{', _) => {
                        self.stack.push(ScriptContext::Interpolation(0));
                        self.can_start_expression = true;
                    }
                    _ => {}
                },
                ScriptContext::JsxTag { closing } => match (current, next) {
                    (b'\'', _) => self.stack.push(ScriptContext::SingleQuote),
                    (b'"', _) => self.stack.push(ScriptContext::DoubleQuote),
                    (b'{', _) => {
                        self.stack.push(ScriptContext::Interpolation(0));
                        self.can_start_expression = true;
                    }
                    (b'/', Some(b'>')) if !closing => {
                        self.stack.pop();
                        self.finish_jsx_element();
                        index += 1;
                    }
                    (b'>', _) => {
                        self.stack.pop();
                        if closing {
                            self.finish_jsx_element();
                        }
                    }
                    _ => {}
                },
                ScriptContext::BlockComment => {
                    if (current, next) == (b'*', Some(b'/')) {
                        self.stack.pop();
                        index += 1;
                    }
                }
                ScriptContext::LineComment => {}
            }
            index += 1;
        }

        if matches!(self.stack.last(), Some(ScriptContext::LineComment)) {
            self.stack.pop();
        }
        if !ends_with_unescaped_backslash(bytes) {
            let mut popped_string = false;
            while matches!(
                self.stack.last(),
                Some(ScriptContext::SingleQuote | ScriptContext::DoubleQuote)
            ) {
                self.stack.pop();
                popped_string = true;
            }
            if popped_string {
                self.can_start_expression = false;
            }
        }
        markers
    }

    fn finish_jsx_element(&mut self) {
        self.jsx_depth = self.jsx_depth.saturating_sub(1);
        if self.jsx_depth == 0 && matches!(self.stack.last(), Some(ScriptContext::JsxText)) {
            self.stack.pop();
            self.can_start_expression = false;
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum StyleContext {
    Code,
    SingleQuote,
    DoubleQuote,
    BlockComment,
    LineComment,
}

pub(super) struct StyleDirectiveLexer {
    context: StyleContext,
    allow_line_comments: bool,
}

impl StyleDirectiveLexer {
    pub(super) fn new(allow_line_comments: bool) -> Self {
        Self {
            context: StyleContext::Code,
            allow_line_comments,
        }
    }

    pub(super) fn scan_line(&mut self, line: &str) -> CommentMarkers {
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

fn record_markers(bytes: &[u8], index: usize, markers: &mut CommentMarkers) {
    if markers.eslint.is_none() && bytes[index..].starts_with(b"eslint-") {
        markers.eslint = Some(index);
    }
    if markers.vize.is_none() && bytes[index..].starts_with(b"@vize:") {
        markers.vize = Some(index);
    }
}
