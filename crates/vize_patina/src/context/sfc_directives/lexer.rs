//! Lightweight comment lexers for SFC script and style block contents.

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
                    (b'/', _) if can_start_literal(bytes, index) => {
                        self.stack.push(ScriptContext::Regex(false));
                    }
                    (b'<', _) if self.jsx && can_start_jsx(bytes, index) => {
                        self.jsx_depth += 1;
                        self.stack.push(ScriptContext::JsxText);
                        self.stack.push(ScriptContext::JsxTag { closing: false });
                    }
                    (b'\'', _) => self.stack.push(ScriptContext::SingleQuote),
                    (b'"', _) => self.stack.push(ScriptContext::DoubleQuote),
                    (b'`', _) => self.stack.push(ScriptContext::Template),
                    (b'{', _) => {
                        if let Some(ScriptContext::Interpolation(depth)) = self.stack.last_mut() {
                            *depth += 1;
                        }
                    }
                    (b'}', _) => {
                        if let Some(ScriptContext::Interpolation(depth)) = self.stack.last_mut() {
                            if *depth == 0 {
                                self.stack.pop();
                            } else {
                                *depth -= 1;
                            }
                        }
                    }
                    _ => {}
                },
                ScriptContext::SingleQuote => match (current, next) {
                    (b'\\', Some(_)) => index += 1,
                    (b'\'', _) => {
                        self.stack.pop();
                    }
                    _ => {}
                },
                ScriptContext::DoubleQuote => match (current, next) {
                    (b'\\', Some(_)) => index += 1,
                    (b'"', _) => {
                        self.stack.pop();
                    }
                    _ => {}
                },
                ScriptContext::Template => match (current, next) {
                    (b'\\', Some(_)) => index += 1,
                    (b'`', _) => {
                        self.stack.pop();
                    }
                    (b'$', Some(b'{')) => {
                        self.stack.push(ScriptContext::Interpolation(0));
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
                    (b'{', _) => self.stack.push(ScriptContext::Interpolation(0)),
                    _ => {}
                },
                ScriptContext::JsxTag { closing } => match (current, next) {
                    (b'\'', _) => self.stack.push(ScriptContext::SingleQuote),
                    (b'"', _) => self.stack.push(ScriptContext::DoubleQuote),
                    (b'{', _) => self.stack.push(ScriptContext::Interpolation(0)),
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
            while matches!(
                self.stack.last(),
                Some(ScriptContext::SingleQuote | ScriptContext::DoubleQuote)
            ) {
                self.stack.pop();
            }
        }
        markers
    }

    fn finish_jsx_element(&mut self) {
        self.jsx_depth = self.jsx_depth.saturating_sub(1);
        if self.jsx_depth == 0 && matches!(self.stack.last(), Some(ScriptContext::JsxText)) {
            self.stack.pop();
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

fn ends_with_unescaped_backslash(bytes: &[u8]) -> bool {
    bytes
        .iter()
        .rev()
        .take_while(|&&byte| byte == b'\\')
        .count()
        % 2
        == 1
}

fn can_start_literal(bytes: &[u8], index: usize) -> bool {
    let previous = bytes[..index]
        .iter()
        .rfind(|&&byte| !byte.is_ascii_whitespace())
        .copied();
    previous.is_none_or(|byte| {
        matches!(
            byte,
            b'(' | b'['
                | b'{'
                | b'='
                | b':'
                | b','
                | b'!'
                | b'?'
                | b';'
                | b'+'
                | b'-'
                | b'*'
                | b'%'
                | b'&'
                | b'|'
                | b'^'
                | b'~'
                | b'<'
                | b'>'
        )
    })
}

fn can_start_jsx(bytes: &[u8], index: usize) -> bool {
    let Some(next) = bytes.get(index + 1).copied() else {
        return false;
    };
    (next == b'>' || next.is_ascii_alphabetic()) && can_start_literal(bytes, index)
}
