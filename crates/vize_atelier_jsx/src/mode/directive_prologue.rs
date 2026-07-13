//! Allocation-light source classification used while Atlas builds a plan.
//!
//! Planning happens before `JsxSyntaxProduct` may execute, so it cannot ask OXC
//! which component modes were parsed. This scanner recognizes only authored
//! string-expression directive prologues at function-like body boundaries. It
//! owns no syntax tree or source strings; execution later checks the resulting
//! backend closure against the cached JSX root metadata.

use super::JsxOutputMode;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
pub(crate) struct ModeDirectiveSet {
    pub(crate) vdom: bool,
    pub(crate) vapor: bool,
}

impl ModeDirectiveSet {
    fn insert(&mut self, mode: JsxOutputMode) {
        match mode {
            JsxOutputMode::Vdom => self.vdom = true,
            JsxOutputMode::Vapor => self.vapor = true,
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum ParenKind {
    Control,
    Function,
    Other,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum LastToken {
    Arrow,
    CloseParen(ParenKind),
    LessThan,
    Value,
    Other,
}

/// Find the client backends required by authored function directive prologues.
pub(crate) fn classify_source_directives(source: &str) -> ModeDirectiveSet {
    let bytes = source.as_bytes();
    let mut modes = ModeDirectiveSet::default();
    let mut parens = Vec::with_capacity(8);
    let mut index = 0;
    let mut last = LastToken::Other;
    let mut awaiting_function_params = false;
    let mut awaiting_function_body = false;
    let mut awaiting_control_params = false;
    let mut awaiting_callable_body = false;
    let mut regex_allowed = true;

    while index < bytes.len() {
        match bytes[index] {
            byte if byte.is_ascii_whitespace() => index += 1,
            b'/' if bytes.get(index + 1) == Some(&b'/') => {
                index = skip_line_comment(bytes, index + 2);
            }
            b'/' if bytes.get(index + 1) == Some(&b'*') => {
                index = skip_block_comment(bytes, index + 2);
            }
            b'/' if regex_allowed && last != LastToken::LessThan => {
                index = skip_regex(bytes, index + 1);
                last = LastToken::Value;
                regex_allowed = false;
            }
            quote @ (b'\'' | b'"') => {
                index = skip_string(bytes, index, quote).unwrap_or(bytes.len());
                last = LastToken::Value;
                regex_allowed = false;
            }
            b'`' => {
                index = skip_template(bytes, index + 1);
                last = LastToken::Value;
                regex_allowed = false;
            }
            b'=' if bytes.get(index + 1) == Some(&b'>') => {
                index += 2;
                last = LastToken::Arrow;
                regex_allowed = true;
            }
            b'(' => {
                let kind = if awaiting_function_params {
                    awaiting_function_params = false;
                    ParenKind::Function
                } else if awaiting_control_params {
                    awaiting_control_params = false;
                    ParenKind::Control
                } else {
                    ParenKind::Other
                };
                parens.push(kind);
                index += 1;
                last = LastToken::Other;
                regex_allowed = true;
            }
            b')' => {
                let kind = parens.pop().unwrap_or(ParenKind::Other);
                awaiting_function_body |= kind == ParenKind::Function;
                awaiting_callable_body = kind == ParenKind::Other;
                index += 1;
                last = LastToken::CloseParen(kind);
                regex_allowed = false;
            }
            b'{' => {
                let function_body = awaiting_function_body
                    || awaiting_callable_body
                    || last == LastToken::Arrow
                    || matches!(last, LastToken::CloseParen(ParenKind::Other));
                if function_body {
                    scan_prologue(bytes, index + 1, &mut modes);
                }
                awaiting_function_body = false;
                awaiting_callable_body = false;
                awaiting_control_params = false;
                index += 1;
                last = LastToken::Other;
                regex_allowed = true;
            }
            b'}' | b']' => {
                index += 1;
                last = LastToken::Value;
                regex_allowed = false;
            }
            b'<' => {
                index += 1;
                last = LastToken::LessThan;
                regex_allowed = true;
            }
            b';' => {
                awaiting_function_body = false;
                awaiting_callable_body = false;
                awaiting_control_params = false;
                index += 1;
                last = LastToken::Other;
                regex_allowed = true;
            }
            byte if is_ident_start(byte) => {
                let start = index;
                index += 1;
                while bytes
                    .get(index)
                    .is_some_and(|byte| is_ident_continue(*byte))
                {
                    index += 1;
                }
                let token = &bytes[start..index];
                if token == b"function" {
                    awaiting_function_params = true;
                    last = LastToken::Other;
                    regex_allowed = true;
                } else if matches!(
                    token,
                    b"if" | b"for" | b"while" | b"switch" | b"catch" | b"with"
                ) {
                    awaiting_control_params = true;
                    last = LastToken::Other;
                    regex_allowed = true;
                } else {
                    last = LastToken::Value;
                    regex_allowed = false;
                }
            }
            _ => {
                index += 1;
                last = LastToken::Other;
                regex_allowed = true;
            }
        }
    }
    modes
}

fn scan_prologue(bytes: &[u8], mut index: usize, modes: &mut ModeDirectiveSet) {
    loop {
        let (next, _) = skip_trivia(bytes, index);
        index = next;
        let Some(&quote @ (b'\'' | b'"')) = bytes.get(index) else {
            return;
        };
        let content_start = index + 1;
        let Some(end) = skip_string(bytes, index, quote) else {
            return;
        };
        let content_end = end - 1;
        let (after_trivia, line_break) = skip_trivia(bytes, end);
        let (terminated, next) = match bytes.get(after_trivia) {
            Some(b';') => (true, after_trivia + 1),
            Some(b'}') | None => (true, after_trivia),
            Some(byte) if line_break && !continues_expression(*byte) => (true, after_trivia),
            _ => (false, after_trivia),
        };
        if !terminated {
            return;
        }
        match &bytes[content_start..content_end] {
            b"use vue:vdom" => modes.insert(JsxOutputMode::Vdom),
            b"use vue:vapor" => modes.insert(JsxOutputMode::Vapor),
            _ => {}
        }
        index = next;
    }
}

fn skip_trivia(bytes: &[u8], mut index: usize) -> (usize, bool) {
    let mut line_break = false;
    loop {
        while bytes.get(index).is_some_and(u8::is_ascii_whitespace) {
            let byte = bytes[index];
            line_break |= byte == b'\n' || byte == b'\r';
            index += 1;
        }
        if bytes[index..].starts_with(b"//") {
            index = skip_line_comment(bytes, index + 2);
            line_break = true;
        } else if bytes[index..].starts_with(b"/*") {
            let start = index;
            index = skip_block_comment(bytes, index + 2);
            line_break |= bytes[start..index].contains(&b'\n');
        } else {
            return (index, line_break);
        }
    }
}

fn skip_string(bytes: &[u8], start: usize, quote: u8) -> Option<usize> {
    let mut index = start + 1;
    while let Some(&byte) = bytes.get(index) {
        match byte {
            b'\\' => index = (index + 2).min(bytes.len()),
            byte if byte == quote => return Some(index + 1),
            b'\n' | b'\r' => return None,
            _ => index += 1,
        }
    }
    None
}

fn skip_template(bytes: &[u8], mut index: usize) -> usize {
    while let Some(&byte) = bytes.get(index) {
        match byte {
            b'\\' => index = (index + 2).min(bytes.len()),
            b'`' => return index + 1,
            _ => index += 1,
        }
    }
    bytes.len()
}

fn skip_regex(bytes: &[u8], mut index: usize) -> usize {
    let mut class = false;
    while let Some(&byte) = bytes.get(index) {
        match byte {
            b'\\' => index = (index + 2).min(bytes.len()),
            b'[' => {
                class = true;
                index += 1;
            }
            b']' => {
                class = false;
                index += 1;
            }
            b'/' if !class => {
                index += 1;
                while bytes
                    .get(index)
                    .is_some_and(|byte| byte.is_ascii_alphabetic())
                {
                    index += 1;
                }
                return index;
            }
            b'\n' | b'\r' => return index,
            _ => index += 1,
        }
    }
    bytes.len()
}

fn skip_line_comment(bytes: &[u8], mut index: usize) -> usize {
    while bytes
        .get(index)
        .is_some_and(|byte| !matches!(byte, b'\n' | b'\r'))
    {
        index += 1;
    }
    index
}

fn skip_block_comment(bytes: &[u8], mut index: usize) -> usize {
    while index + 1 < bytes.len() {
        if bytes[index..].starts_with(b"*/") {
            return index + 2;
        }
        index += 1;
    }
    bytes.len()
}

fn continues_expression(byte: u8) -> bool {
    matches!(
        byte,
        b'(' | b'['
            | b'.'
            | b'+'
            | b'-'
            | b'/'
            | b'*'
            | b'%'
            | b'?'
            | b','
            | b':'
            | b'='
            | b'<'
            | b'>'
            | b'&'
            | b'|'
            | b'^'
            | b'`'
    )
}

fn is_ident_start(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || matches!(byte, b'_' | b'$')
}
fn is_ident_continue(byte: u8) -> bool {
    is_ident_start(byte) || byte.is_ascii_digit()
}

#[cfg(test)]
#[path = "directive_prologue/tests.rs"]
mod tests;
