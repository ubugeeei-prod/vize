//! Braced syntax: `{` opens a block, `}` closes it, `;` ends a statement
//! outside parentheses. The last statement before `}` needs no `;`. A
//! custom property's value is any token run, so its braces nest inside it.

use super::lex::{Lexeme, char_end, is_name_byte, lex, plain_run};
use super::{Builder, Prelude};

pub(super) fn read(builder: &mut Builder<'_>) {
    let bytes = builder.bytes();
    let lexis = builder.sheet.lexis;
    let mut statement: Option<Prelude> = None;
    // Parentheses and square brackets open in the current statement.
    let mut parens = 0usize;
    let mut brackets = 0usize;
    let mut index = 0usize;
    while let Some(&byte) = bytes.get(index) {
        if byte.is_ascii_whitespace() {
            index += 1;
            continue;
        }
        let run = plain_run(bytes, index);
        if run > index {
            statement.get_or_insert(Prelude::new(index)).end = run;
            index = run;
            continue;
        }
        let mut interpolated = false;
        // Structure bytes never start a lexeme.
        let lexeme = if matches!(byte, b';' | b'}' | b'(' | b')' | b'[' | b']' | b':') {
            Lexeme::Byte
        } else {
            lex(bytes, index, &lexis, true)
        };
        let end = match lexeme {
            Lexeme::Comment {
                end,
                line,
                terminated,
            } => {
                builder.comment(index, end, line);
                if !terminated {
                    builder.unterminated();
                }
                index = end;
                continue;
            }
            Lexeme::Opaque { end, terminated } => {
                if !terminated {
                    builder.unterminated();
                }
                end
            }
            Lexeme::Interpolation { end, terminated } => {
                if !terminated {
                    builder.unterminated();
                }
                interpolated = true;
                end
            }
            Lexeme::Byte => match byte {
                b'{' if statement
                    .is_some_and(|prelude| custom_property(bytes, prelude.start, index)) =>
                {
                    group_end(builder, index)
                }
                b'{' => {
                    let prelude = statement.take().unwrap_or(Prelude::new(index));
                    builder.open(prelude, index);
                    (parens, brackets) = (0, 0);
                    index += 1;
                    continue;
                }
                b'}' => {
                    if let Some(prelude) = statement.take() {
                        builder.statement(prelude, prelude.end);
                    }
                    builder.close(index + 1);
                    (parens, brackets) = (0, 0);
                    index += 1;
                    continue;
                }
                b';' if parens == 0 => {
                    if let Some(prelude) = statement.take() {
                        builder.statement(prelude, index + 1);
                    }
                    brackets = 0;
                    index += 1;
                    continue;
                }
                b'(' => {
                    parens += 1;
                    index + 1
                }
                b')' => {
                    parens = parens.saturating_sub(1);
                    index + 1
                }
                b'[' => {
                    brackets += 1;
                    index + 1
                }
                b']' => {
                    brackets = brackets.saturating_sub(1);
                    index + 1
                }
                _ => char_end(bytes, index),
            },
        };
        let prelude = statement.get_or_insert(Prelude::new(index));
        if byte == b':' && parens == 0 && brackets == 0 && prelude.colon.is_none() {
            prelude.colon = Some(index);
        }
        prelude.end = end;
        prelude.interpolated |= interpolated;
        index = end;
    }
    if let Some(prelude) = statement {
        builder.statement(prelude, prelude.end);
    }
}

/// `--name:` starts the statement.
fn custom_property(bytes: &[u8], start: usize, end: usize) -> bool {
    let Some(text) = bytes.get(start..end) else {
        return false;
    };
    text.starts_with(b"--")
        && text
            .iter()
            .position(|&byte| byte == b':')
            .and_then(|colon| text.get(2..colon))
            .is_some_and(|name| name.trim_ascii_end().iter().all(|&byte| is_name_byte(byte)))
}

/// The end of the `{…}` group at `index`, after its matching `}`.
fn group_end(builder: &mut Builder<'_>, index: usize) -> usize {
    let bytes = builder.bytes();
    let lexis = builder.sheet.lexis;
    let mut depth = 0usize;
    let mut cursor = index;
    while let Some(&byte) = bytes.get(cursor) {
        cursor = match lex(bytes, cursor, &lexis, true) {
            Lexeme::Comment { end, .. }
            | Lexeme::Opaque { end, .. }
            | Lexeme::Interpolation { end, .. } => end,
            Lexeme::Byte => {
                match byte {
                    b'{' => depth += 1,
                    b'}' => {
                        depth -= 1;
                        if depth == 0 {
                            return cursor + 1;
                        }
                    }
                    _ => {}
                }
                char_end(bytes, cursor)
            }
        };
    }
    builder.unbalanced();
    bytes.len()
}
