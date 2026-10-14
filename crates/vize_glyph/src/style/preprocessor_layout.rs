//! Re-indent SCSS and Less blocks by nesting depth, as Prettier lays them out.
//!
//! Only whitespace at the start and end of lines and blank lines change; every
//! token stays as written. A line's level is its `{}` and `()` depth, plus one
//! for a declaration continued over several lines. Lines that start inside a
//! block comment or a string stay as written. Blank lines collapse to one and
//! are dropped after `{` and before `}`. Anything unbalanced keeps the whole
//! block as written.

use vize_l0::String;

/// What one line contributes, scanned outside comments and strings.
#[derive(Clone, Copy, Default)]
struct Line {
    /// The line starts inside a block comment or a string.
    verbatim: bool,
    /// The trimmed line is empty and outside comments and strings.
    blank: bool,
    /// `}` and `)` before the first other character.
    leading_closers: usize,
    /// The trimmed line starts with `}`.
    leading_brace: bool,
    /// Depth before the line.
    depth: usize,
    /// Last character outside comments, if any.
    last: Option<u8>,
    /// The line holds only a comment.
    comment_only: bool,
}

#[derive(Clone, Copy, PartialEq)]
enum Mode {
    Code,
    BlockComment,
    String(u8),
}

pub(super) fn reindent(source: &str, indent: &str, newline: &str) -> Option<String> {
    let lines: Vec<&str> = source.lines().collect();
    let scanned = scan(&lines)?;
    let terminators = statement_terminators(&scanned);
    let mut output = String::with_capacity(source.len());
    let mut previous: Option<Line> = None;
    let mut previous_code: Option<Line> = None;
    let mut statement_depth = 0;
    let mut pending_blank = false;
    for (index, (text, line)) in lines.iter().zip(&scanned).enumerate() {
        if line.blank {
            pending_blank = previous.is_some();
            continue;
        }
        let after_open = previous.is_some_and(|line| !line.verbatim && line.last == Some(b'{'));
        if pending_blank && !after_open && !(line.leading_brace && !line.verbatim) {
            output.push_str(newline);
        }
        pending_blank = false;
        if previous.is_some() {
            output.push_str(newline);
        }
        if line.verbatim {
            output.push_str(text.trim_end());
        } else {
            // A statement starts after code that ended one. Lines inside its
            // parentheses already carry that depth; a declaration continued at
            // its own depth gets one more level.
            let starts = line.leading_brace
                || previous_code
                    .is_none_or(|code| matches!(code.last, None | Some(b';' | b'{' | b'}')));
            if starts && !line.comment_only {
                statement_depth = line.depth;
            }
            let continued = !line.comment_only
                && !starts
                && line.depth == statement_depth
                && matches!(terminators.get(index), Some(Some(b';' | b'}')));
            let level = line.depth - line.leading_closers + usize::from(continued);
            for _ in 0..level {
                output.push_str(indent);
            }
            output.push_str(text.trim());
        }
        if !line.verbatim && !line.comment_only {
            previous_code = Some(*line);
        }
        previous = Some(*line);
    }
    Some(output)
}

fn scan(lines: &[&str]) -> Option<Vec<Line>> {
    let mut mode = Mode::Code;
    let mut depth = 0usize;
    let mut scanned = Vec::with_capacity(lines.len());
    for text in lines {
        let verbatim = mode != Mode::Code;
        let trimmed = text.trim();
        let mut line = Line {
            verbatim,
            blank: !verbatim && trimmed.is_empty(),
            depth,
            ..Line::default()
        };
        if !verbatim {
            for byte in trimmed.bytes() {
                match byte {
                    b'}' | b')' => line.leading_closers += 1,
                    b' ' | b'\t' => {}
                    _ => break,
                }
            }
            line.leading_brace = trimmed.starts_with('}');
            line.leading_closers = line.leading_closers.min(depth);
        }
        let bytes = text.as_bytes();
        let mut index = 0;
        let mut saw_code = false;
        while let Some(&byte) = bytes.get(index) {
            match mode {
                Mode::BlockComment => {
                    if byte == b'*' && bytes.get(index + 1) == Some(&b'/') {
                        mode = Mode::Code;
                        index += 1;
                    }
                }
                Mode::String(quote) => {
                    if byte == b'\\' {
                        index += 1;
                    } else if byte == quote {
                        mode = Mode::Code;
                        line.last = Some(byte);
                    }
                }
                Mode::Code => match byte {
                    b'/' if bytes.get(index + 1) == Some(&b'*') => {
                        mode = Mode::BlockComment;
                        index += 1;
                    }
                    b'/' if bytes.get(index + 1) == Some(&b'/') => break,
                    b'"' | b'\'' => {
                        mode = Mode::String(byte);
                        saw_code = true;
                    }
                    b'{' | b'(' => {
                        depth += 1;
                        saw_code = true;
                        line.last = Some(byte);
                    }
                    b'}' | b')' => {
                        depth = depth.checked_sub(1)?;
                        saw_code = true;
                        line.last = Some(byte);
                    }
                    b' ' | b'\t' | b'\r' => {}
                    _ => {
                        saw_code = true;
                        line.last = Some(byte);
                    }
                },
            }
            index += 1;
        }
        // A string may continue only through an escaped newline.
        if let Mode::String(_) = mode
            && !text.trim_end().ends_with('\\')
        {
            return None;
        }
        line.comment_only = !verbatim && !line.blank && !saw_code;
        scanned.push(line);
    }
    (mode == Mode::Code && depth == 0).then_some(scanned)
}

/// For each line, the character that ends its statement: `{`, `;`, `}` or
/// `None` at the end of the block.
fn statement_terminators(lines: &[Line]) -> Vec<Option<u8>> {
    let mut terminators = vec![None; lines.len()];
    let mut current = None;
    for (index, line) in lines.iter().enumerate().rev() {
        if line.blank || line.comment_only || line.verbatim {
            if let Some(slot) = terminators.get_mut(index) {
                *slot = current;
            }
            continue;
        }
        if matches!(line.last, Some(b';' | b'{' | b'}')) {
            current = line.last;
        }
        if let Some(slot) = terminators.get_mut(index) {
            *slot = current;
        }
        if line.leading_brace {
            current = Some(b'}');
        }
    }
    terminators
}

#[cfg(test)]
mod tests {
    use super::reindent;

    #[track_caller]
    fn check(source: &str, expected: &str) {
        let output = reindent(source, "  ", "\n");
        assert_eq!(output.as_deref(), Some(expected));
        assert_eq!(reindent(expected, "  ", "\n").as_deref(), Some(expected));
    }

    #[test]
    fn indents_by_nesting_depth() {
        check(
            ".a {\n\tcolor: red;\n\n\n\t&:hover {\n\t\tcolor: blue\n\t}\n\n}",
            ".a {\n  color: red;\n\n  &:hover {\n    color: blue\n  }\n}",
        );
    }

    #[test]
    fn continues_declarations_and_keeps_selector_lists() {
        check(
            ".a,\n.b {\ntransition:\nopacity 1s,\ntransform 1s;\n}",
            ".a,\n.b {\n  transition:\n    opacity 1s,\n    transform 1s;\n}",
        );
    }

    #[test]
    fn indents_parentheses_once() {
        check(
            "$map: (\nkey: value,\nother: (\na: 1,\n),\n);",
            "$map: (\n  key: value,\n  other: (\n    a: 1,\n  ),\n);",
        );
    }

    #[test]
    fn keeps_comments_and_strings_as_written() {
        check(
            ".a {\n\t/* one\n\t   two { */\n\tcontent: \"{\"; // }\n}",
            ".a {\n  /* one\n\t   two { */\n  content: \"{\"; // }\n}",
        );
    }

    #[test]
    fn keeps_unbalanced_blocks() {
        assert_eq!(reindent(".a {\n\tcolor: red;", "  ", "\n"), None);
        assert_eq!(reindent(".a { content: 'b\n}", "  ", "\n"), None);
    }
}
