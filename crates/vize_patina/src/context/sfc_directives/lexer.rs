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
    BlockComment,
    LineComment,
}

pub(super) struct DirectiveLexer {
    stack: Vec<ScriptContext>,
}

impl Default for DirectiveLexer {
    fn default() -> Self {
        Self {
            stack: vec![ScriptContext::Code],
        }
    }
}

impl DirectiveLexer {
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
