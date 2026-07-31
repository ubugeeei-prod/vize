//! Lightweight comment lexers for SFC script and style block contents.

mod brace;
mod style;
mod token;

pub(super) use style::StyleDirectiveLexer;

use brace::DelimiterState;
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
    tsx: bool,
    jsx_depth: u32,
    can_start_expression: bool,
    after_dot: bool,
    delimiters: DelimiterState,
}

impl Default for DirectiveLexer {
    fn default() -> Self {
        Self::new(false, false)
    }
}

impl DirectiveLexer {
    pub(super) fn new(jsx: bool, tsx: bool) -> Self {
        Self {
            stack: vec![ScriptContext::Code],
            jsx,
            tsx,
            jsx_depth: 0,
            can_start_expression: true,
            after_dot: false,
            delimiters: DelimiterState::default(),
        }
    }

    pub(super) fn scan_line(&mut self, line: &str, remaining: &str) -> CommentMarkers {
        let bytes = line.as_bytes();
        let remaining_bytes = remaining.as_bytes();
        let mut markers = CommentMarkers::default();
        let mut has_code_token = false;
        let mut index = 0;
        while index < bytes.len() {
            let context = self.stack.last().copied().unwrap_or(ScriptContext::Code);
            if matches!(
                context,
                ScriptContext::BlockComment | ScriptContext::LineComment
            ) {
                record_markers(bytes, index, &mut markers);
            }

            let current = bytes[index];
            let next = bytes.get(index + 1).copied();
            let had_code_token = has_code_token;
            let in_code = matches!(
                context,
                ScriptContext::Code | ScriptContext::Interpolation(_)
            );
            let starts_comment = in_code && matches!((current, next), (b'/', Some(b'/' | b'*')));
            if in_code && !is_identifier_start(current) {
                self.delimiters
                    .before_non_identifier(current, starts_comment);
            }
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
                        if self.jsx
                            && self.can_start_expression
                            && is_jsx_start(remaining_bytes, index, self.tsx) =>
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
                        self.delimiters.open_brace(!self.can_start_expression);
                        self.can_start_expression = true;
                        self.after_dot = false;
                    }
                    (b'}', _) => {
                        let mut closes_interpolation = false;
                        if let Some(ScriptContext::Interpolation(depth)) = self.stack.last_mut() {
                            if *depth == 0 {
                                closes_interpolation = true;
                                self.stack.pop();
                            } else {
                                *depth -= 1;
                            }
                        }
                        self.can_start_expression = if closes_interpolation {
                            false
                        } else {
                            self.delimiters.close_brace()
                        };
                        self.after_dot = false;
                    }
                    (byte, _) if is_identifier_start(byte) => {
                        let end = identifier_end(bytes, index);
                        let identifier = &bytes[index..end];
                        let after_dot = self.after_dot;
                        self.delimiters.observe_identifier(identifier, after_dot);
                        self.can_start_expression =
                            !after_dot && identifier_allows_expression(identifier);
                        self.after_dot = false;
                        index = end - 1;
                    }
                    (b'(', _) => {
                        self.delimiters.open_paren();
                        self.can_start_expression = true;
                        self.after_dot = false;
                    }
                    (b')', _) => {
                        self.can_start_expression = self.delimiters.close_paren();
                        self.after_dot = false;
                    }
                    (b']' | b'0'..=b'9', _) => {
                        self.can_start_expression = false;
                        self.after_dot = false;
                    }
                    (b'.', _) => self.after_dot = true,
                    (b'+', Some(b'+')) | (b'-', Some(b'-')) => {
                        // Preserve the incoming token state: prefix ++/-- still
                        // expects an operand, while postfix ++/-- finishes one.
                        self.delimiters.observe_operator(current);
                        self.after_dot = false;
                        index += 1;
                    }
                    (b'=', Some(b'>')) => {
                        self.delimiters.observe_arrow();
                        self.can_start_expression = true;
                        self.after_dot = false;
                        index += 1;
                    }
                    (b'!', Some(b'=')) => {
                        self.delimiters.observe_operator(current);
                        self.can_start_expression = true;
                        self.after_dot = false;
                    }
                    (b'!', _) => {
                        // At an expression boundary `!` is logical-not. After
                        // an operand on this line it is TypeScript's postfix
                        // non-null assertion and must keep the operand complete.
                        self.delimiters.observe_operator(current);
                        self.can_start_expression = self.can_start_expression || !had_code_token;
                        self.after_dot = false;
                    }
                    (
                        b'[' | b'/' | b'=' | b':' | b',' | b'?' | b';' | b'+' | b'-' | b'*' | b'%'
                        | b'&' | b'|' | b'^' | b'~' | b'<' | b'>',
                        _,
                    ) => {
                        self.delimiters.observe_operator(current);
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
            let in_comment = matches!(
                context,
                ScriptContext::BlockComment | ScriptContext::LineComment
            );
            if !in_comment && !current.is_ascii_whitespace() && !starts_comment {
                has_code_token = true;
            }
            index += 1;
        }

        if matches!(self.stack.last(), Some(ScriptContext::LineComment)) {
            self.stack.pop();
        }
        let mut popped_regex = false;
        while matches!(self.stack.last(), Some(ScriptContext::Regex(_))) {
            self.stack.pop();
            popped_regex = true;
        }
        if popped_regex {
            // Regex literals never span lines, so an unterminated one means the
            // opening `/` was a division operator (or the source is mid-edit).
            self.can_start_expression = true;
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
        self.delimiters.finish_line(self.can_start_expression);
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

pub(super) fn record_markers(bytes: &[u8], index: usize, markers: &mut CommentMarkers) {
    if markers.eslint.is_none() && bytes[index..].starts_with(b"eslint-") {
        markers.eslint = Some(index);
    }
    if markers.vize.is_none() && bytes[index..].starts_with(b"@vize:") {
        markers.vize = Some(index);
    }
}
