//! Indented syntax: a line opens a block when the next code line is indented
//! deeper. A selector list continues over lines that end with `,`.

use alloc::vec::Vec;

use super::lex::{Lexeme, char_end, lex, plain_run};
use super::{Builder, Prelude};

/// One logical line: its indentation and code, comments excluded.
struct Line {
    indent: usize,
    prelude: Prelude,
}

pub(super) fn read(builder: &mut Builder<'_>) {
    let lines = lines(builder);
    let mut indents: Vec<usize> = Vec::new();
    let mut last_end = 0usize;
    for (index, line) in lines.iter().enumerate() {
        while indents.last().is_some_and(|&open| open >= line.indent) {
            indents.pop();
            builder.close(last_end);
        }
        match lines
            .get(index + 1)
            .filter(|next| next.indent > line.indent)
        {
            Some(child) => {
                builder.open(line.prelude, child.prelude.start);
                indents.push(line.indent);
            }
            None => builder.statement(line.prelude, line.prelude.end),
        }
        last_end = line.prelude.end;
    }
    for _ in indents {
        builder.close(last_end);
    }
}

fn lines(builder: &mut Builder<'_>) -> Vec<Line> {
    let bytes = builder.bytes();
    let lexis = builder.sheet.lexis;
    let mut lines = Vec::new();
    let mut pending: Option<Line> = None;
    let mut cursor = 0usize;
    while cursor < bytes.len() {
        let mut line_end = line_end(bytes, cursor);
        let first = (cursor..line_end).find(|&index| {
            bytes
                .get(index)
                .is_some_and(|byte| !byte.is_ascii_whitespace())
        });
        let Some(first) = first else {
            cursor = line_end + 1;
            continue;
        };
        let indent = first - cursor;
        if lexis.indented_comments && pending.is_none() && starts_comment(bytes, first) {
            let end = indented_comment_end(bytes, line_end, indent);
            let line = bytes.get(first + 1) == Some(&b'/');
            builder.comment(first, end, line);
            cursor = end + 1;
            continue;
        }
        let mut code: Option<Prelude> = None;
        let mut depth = 0usize;
        let mut index = first;
        while index < line_end {
            let run = plain_run(bytes, index);
            if run > index {
                code.get_or_insert(Prelude::new(index)).end = run;
                index = run;
                continue;
            }
            let mut interpolated = false;
            let end = match lex(bytes, index, &lexis, false) {
                Lexeme::Comment {
                    end,
                    line,
                    terminated,
                } => {
                    builder.comment(index, end, line);
                    if !terminated {
                        builder.unterminated();
                    }
                    // A block comment over several lines joins them.
                    if end > line_end {
                        line_end = self::line_end(bytes, end);
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
                Lexeme::Byte if bytes.get(index).is_some_and(u8::is_ascii_whitespace) => {
                    index += 1;
                    continue;
                }
                Lexeme::Byte => {
                    match bytes.get(index) {
                        Some(b'(' | b'[') => depth += 1,
                        Some(b')' | b']') => depth = depth.saturating_sub(1),
                        _ => {}
                    }
                    char_end(bytes, index)
                }
            };
            let prelude = code.get_or_insert(Prelude::new(index));
            if bytes.get(index) == Some(&b':') && depth == 0 && prelude.colon.is_none() {
                prelude.colon = Some(index);
            }
            prelude.end = end;
            prelude.interpolated |= interpolated;
            index = end;
        }
        cursor = line_end + 1;
        let Some(code) = code else {
            continue;
        };
        let line = match pending.take() {
            Some(mut line) => {
                line.prelude.end = code.end;
                line.prelude.colon = line.prelude.colon.or(code.colon);
                line.prelude.interpolated |= code.interpolated;
                line
            }
            None => Line {
                indent,
                prelude: code,
            },
        };
        if bytes.get(code.end.saturating_sub(1)) == Some(&b',') {
            pending = Some(line);
        } else {
            lines.push(line);
        }
    }
    lines.extend(pending);
    lines
}

fn line_end(bytes: &[u8], from: usize) -> usize {
    bytes
        .get(from..)
        .and_then(|rest| rest.iter().position(|&byte| byte == b'\n'))
        .map_or(bytes.len(), |offset| from + offset)
}

fn starts_comment(bytes: &[u8], index: usize) -> bool {
    bytes.get(index) == Some(&b'/') && matches!(bytes.get(index + 1), Some(b'/' | b'*'))
}

/// A comment that starts a line covers every deeper-indented line after it.
fn indented_comment_end(bytes: &[u8], first_line_end: usize, indent: usize) -> usize {
    let mut end = first_line_end;
    let mut cursor = first_line_end + 1;
    while cursor < bytes.len() {
        let next_end = line_end(bytes, cursor);
        let code = (cursor..next_end).find(|&index| {
            bytes
                .get(index)
                .is_some_and(|byte| !byte.is_ascii_whitespace())
        });
        match code {
            Some(code) if code - cursor > indent => end = next_end,
            Some(_) => break,
            None => {}
        }
        cursor = next_end + 1;
    }
    end
}
