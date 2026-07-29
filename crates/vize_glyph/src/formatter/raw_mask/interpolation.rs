/// Lexer state for the JS expression inside a `{{ … }}` interpolation.
///
/// `}}` and backticks are only structural in code position: inside a string,
/// a comment, or a nested template literal they are ordinary characters. A
/// scan that ignored that let `{{ "}}" + ` … ` }}` close the interpolation at
/// the string's `}}`, so the literal's lines lost their raw marking and their
/// runtime value was reformatted, and a single boolean could not represent a
/// template literal nested inside a `${ … }` substitution.
///
/// This mirrors `template_literal_quasi_line_starts` (#3247), which decides
/// the same question for the formatted expression, so both layers agree on
/// which lines are quasi text. Regex literals are not modelled; a backtick
/// inside one is rare and would at worst leave a line indented.
#[derive(Default)]
pub(super) struct InterpolationScan {
    /// Set by `{{`, cleared by the matching `}}`.
    pub(super) active: bool,
    /// Nested template-literal / `${ … }` frames within the expression.
    frames: Vec<ExprFrame>,
    /// Open `'…'` / `"…"` string, if any. Reset at every line boundary,
    /// since such strings cannot span a newline.
    pub(super) string: Option<u8>,
    /// Inside a `/* … */` comment.
    in_block_comment: bool,
}

enum ExprFrame {
    /// Inside backticks, in quasi text (part of the string's runtime value).
    TemplateLiteral,
    /// Inside `${ … }`; tracks `{`/`}` nesting within the substitution.
    Substitution(u32),
}

impl InterpolationScan {
    /// Whether the current line's first byte lies in template-literal quasi
    /// text, which the SFC layer must leave unindented.
    pub(super) fn line_starts_in_quasi(&self) -> bool {
        self.string.is_none()
            && !self.in_block_comment
            && matches!(self.frames.last(), Some(ExprFrame::TemplateLiteral))
    }

    /// Consume one token at `cursor`, returning the next cursor position.
    pub(super) fn step(&mut self, bytes: &[u8], cursor: usize) -> usize {
        let byte = bytes[cursor];
        if self.in_block_comment {
            if bytes[cursor..].starts_with(b"*/") {
                self.in_block_comment = false;
                return cursor + 2;
            }
            return cursor + 1;
        }
        if let Some(quote) = self.string {
            if byte == b'\\' {
                return cursor + 2;
            }
            if byte == quote {
                self.string = None;
            }
            return cursor + 1;
        }
        if matches!(self.frames.last(), Some(ExprFrame::TemplateLiteral)) {
            return match byte {
                b'\\' => cursor + 2,
                b'`' => {
                    self.frames.pop();
                    cursor + 1
                }
                b'$' if bytes.get(cursor + 1) == Some(&b'{') => {
                    self.frames.push(ExprFrame::Substitution(0));
                    cursor + 2
                }
                _ => cursor + 1,
            };
        }
        match byte {
            b'/' if bytes.get(cursor + 1) == Some(&b'/') => bytes.len(),
            b'/' if bytes.get(cursor + 1) == Some(&b'*') => {
                self.in_block_comment = true;
                cursor + 2
            }
            b'\'' | b'"' => {
                self.string = Some(byte);
                cursor + 1
            }
            b'`' => {
                self.frames.push(ExprFrame::TemplateLiteral);
                cursor + 1
            }
            b'{' => {
                if let Some(ExprFrame::Substitution(depth)) = self.frames.last_mut() {
                    *depth += 1;
                }
                cursor + 1
            }
            b'}' if self.frames.is_empty() && bytes[cursor..].starts_with(b"}}") => {
                self.active = false;
                cursor + 2
            }
            b'}' => {
                match self.frames.last_mut() {
                    Some(ExprFrame::Substitution(0)) => {
                        self.frames.pop();
                    }
                    Some(ExprFrame::Substitution(depth)) => *depth -= 1,
                    _ => {}
                }
                cursor + 1
            }
            _ => cursor + 1,
        }
    }
}
