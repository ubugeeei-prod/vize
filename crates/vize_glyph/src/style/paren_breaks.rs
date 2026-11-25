//! Keep a multi-line function argument list broken at its parentheses.
//!
//! The CSS printer keeps the authored line breaks between the arguments of a
//! value it does not parse, but trims the whitespace at both ends of every
//! argument list. A list authored one argument per line therefore came back
//! with its first argument pulled onto the opening line and the closing
//! parenthesis pulled onto the last one. Where the printed arguments still
//! break, the line breaks at both parentheses are restored: after `(` with the
//! arguments' own indentation, and before `)` with the indentation of the line
//! that opened the list. A list the printer joined stays joined.

use super::blank_lines::Tokens;
use vize_l0::{String, cstr};

/// Restore the parenthesis line breaks the printer trimmed from `printed`.
///
/// `authored` and `printed` must hold the same tokens; any difference leaves
/// `printed` unchanged.
pub(super) fn restore(authored: &str, printed: String) -> String {
    if !has_broken_parenthesis(authored.as_bytes()) {
        return printed;
    }
    let mut edits = match collect(authored, printed.as_str()) {
        Some(edits) if !edits.is_empty() => edits,
        _ => return printed,
    };
    // A list's opening edit is known only once the list closes.
    edits.sort_by_key(|edit| edit.at);
    let mut output = String::with_capacity(printed.len() + edits.len() * 8);
    let mut cursor = 0;
    for edit in edits {
        output.push_str(printed.get(cursor..edit.at).unwrap_or_default());
        output.push_str(edit.gap.as_str());
        cursor = edit.at;
    }
    output.push_str(printed.get(cursor..).unwrap_or_default());
    output
}

/// Whether any `(` is followed, or any `)` preceded, by a line break.
fn has_broken_parenthesis(bytes: &[u8]) -> bool {
    let breaks = |byte: Option<&u8>| byte.is_some_and(|byte| matches!(byte, b'\n' | b'\r'));
    memchr::memchr2_iter(b'(', b')', bytes).any(|index| {
        let inline = |byte: &&u8| !matches!(byte, b' ' | b'\t');
        if bytes.get(index) == Some(&b'(') {
            breaks(
                bytes
                    .get(index + 1..)
                    .unwrap_or_default()
                    .iter()
                    .find(inline),
            )
        } else {
            breaks(
                bytes
                    .get(..index)
                    .unwrap_or_default()
                    .iter()
                    .rev()
                    .find(inline),
            )
        }
    })
}

struct Edit {
    at: usize,
    gap: String,
}

struct Frame<'a> {
    /// Printed offset of the first argument when the printer trimmed its break.
    open: Option<usize>,
    /// The first printed line break between this list's own arguments.
    interior: Option<&'a str>,
    /// Indentation of the printed line holding the opening parenthesis.
    indentation: &'a str,
}

fn collect(authored: &str, printed: &str) -> Option<Vec<Edit>> {
    let mut original = Tokens::new(authored);
    let mut target = Tokens::new(printed);
    let mut frames: Vec<Frame<'_>> = Vec::new();
    let mut edits = Vec::new();
    let mut after_open = false;
    loop {
        let (source, token) = match (original.next(), target.next()) {
            (None, None) => break,
            (Some(source), Some(token)) if source.text == token.text => (source, token),
            _ => return None,
        };
        let trimmed_break = source.gap.contains(['\r', '\n']) && token.gap.is_empty();
        let depth = frames.len();
        if token.text == ")" {
            if let Some(frame) = frames.pop()
                && let Some(interior) = frame.interior
            {
                let (terminator, indentation) = split_gap(interior);
                if let Some(at) = frame.open {
                    edits.push(Edit {
                        at,
                        gap: cstr!("{terminator}{indentation}"),
                    });
                }
                if trimmed_break {
                    edits.push(Edit {
                        at: token.start,
                        gap: cstr!("{terminator}{}", frame.indentation),
                    });
                }
            }
        } else if let Some(frame) = frames.last_mut() {
            if after_open {
                if trimmed_break {
                    frame.open = Some(token.start);
                }
            } else if frame.interior.is_none()
                && token.parens == depth
                && token.gap.contains(['\r', '\n'])
            {
                frame.interior = Some(token.gap);
            }
        }
        after_open = token.text == "(";
        if after_open {
            frames.push(Frame {
                open: None,
                interior: None,
                indentation: line_indentation(printed, token.start),
            });
        }
    }
    Some(edits)
}

/// Split a printed line-break gap into its last line terminator and the
/// indentation after it.
fn split_gap(gap: &str) -> (&str, &str) {
    let indentation_start = gap.rfind(['\r', '\n']).map_or(0, |index| index + 1);
    let head = gap.get(..indentation_start).unwrap_or_default();
    let terminator = if head.ends_with("\r\n") {
        "\r\n"
    } else if head.ends_with('\r') {
        "\r"
    } else {
        "\n"
    };
    (terminator, gap.get(indentation_start..).unwrap_or_default())
}

/// Leading whitespace of the line containing `offset`.
fn line_indentation(source: &str, offset: usize) -> &str {
    let line_start = source
        .get(..offset)
        .and_then(|before| before.rfind(['\r', '\n']))
        .map_or(0, |index| index + 1);
    let line = source.get(line_start..offset).unwrap_or_default();
    line.get(..line.len() - line.trim_start().len())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::restore;

    #[test]
    fn printed_argument_breaks_take_back_both_parenthesis_breaks() {
        let authored = ".a {\n  b: f(\n    x,\n    y\n  );\n}";
        let printed = ".a {\n  b: f(x,\n    y);\n}\n";
        assert_eq!(
            restore(authored, printed.into()),
            ".a {\n  b: f(\n    x,\n    y\n  );\n}\n"
        );
    }

    #[test]
    fn joined_or_different_lists_stay_as_printed() {
        let authored = ".a {\n  b: f(\n    x, y\n  );\n}";
        let joined = ".a {\n  b: f(x, y);\n}\n";
        assert_eq!(restore(authored, joined.into()), joined);
        let changed = ".a {\n  b: g(x,\n    y);\n}\n";
        assert_eq!(restore(authored, changed.into()), changed);
    }
}
