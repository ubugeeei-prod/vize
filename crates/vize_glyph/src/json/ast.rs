//! The JSON / JSONC value tree and the comment-placement helper shared by the
//! parser and printer.

use vize_l0::String;

pub(super) enum Node {
    /// A mapping, retaining the author's explicit newline after `{`.
    Object {
        members: Vec<Member>,
        expanded: bool,
        /// Comments on their own line after the last member, before `}`.
        dangling: Vec<Comment>,
    },
    /// A sequence, collapsed when its contents fit on the current line.
    Array {
        elements: Vec<Element>,
        dangling: Vec<Comment>,
    },
    /// A string, number, `true`, `false`, or `null`, copied verbatim.
    Scalar(String),
}

pub(super) struct Member {
    pub(super) blank_line_before: bool,
    /// Comments printed on their own lines before the key.
    pub(super) leading: Vec<Comment>,
    /// The key, including its surrounding quotes, verbatim.
    pub(super) key: String,
    pub(super) value: Node,
    /// Comments printed on the same line after the value (and comma).
    pub(super) trailing: Vec<Comment>,
}

pub(super) struct Element {
    pub(super) blank_line_before: bool,
    pub(super) leading: Vec<Comment>,
    pub(super) value: Node,
    pub(super) trailing: Vec<Comment>,
}

pub(super) struct Comment {
    /// Trivia after the preceding token, kept next to this comment.
    pub(super) blank_line_before: bool,
    pub(super) end: usize,
    /// `true` for `/* ... */`, `false` for `// ...`.
    pub(super) block: bool,
    /// The text between the comment markers, verbatim (line comments are
    /// trimmed at the end so reformatting does not leave trailing whitespace).
    pub(super) text: String,
    /// Whether a newline separated this comment from the previous token. Used to
    /// decide whether a comment trails a value or belongs on its own line.
    pub(super) own_line: bool,
}

/// Split comments collected after a value into the run that trails the value on
/// the same line and the remaining comments that belong on their own lines.
///
/// The trailing run starts only if the first comment shares the value's line. A
/// `//` line comment ends the run (anything after it is on a later line), while
/// `/* */` block comments can chain on one line.
pub(super) fn split_trailing(mut comments: Vec<Comment>) -> (Vec<Comment>, Vec<Comment>) {
    if comments.first().is_none_or(|c| c.own_line) {
        return (Vec::new(), comments);
    }

    let mut cut = 0;
    for (i, comment) in comments.iter().enumerate() {
        if i > 0 && comment.own_line {
            break;
        }
        cut = i + 1;
        if !comment.block {
            break; // a line comment runs to end of line
        }
    }

    let spill = comments.split_off(cut);
    (comments, spill)
}

/// Detect an empty trivia line, excluding newlines inside block comments.
/// CRLF is one line terminator, just like LF and CR input.
pub(super) fn has_blank_line(trivia: &str) -> bool {
    let mut characters = trivia.chars().peekable();
    let mut newlines = 0;
    let mut previous_cr = false;
    while let Some(character) = characters.next() {
        match character {
            '/' if characters.peek() == Some(&'/') => {
                while characters.peek().is_some_and(|c| !matches!(c, '\r' | '\n')) {
                    characters.next();
                }
                newlines = 0;
                previous_cr = false;
            }
            '/' if characters.peek() == Some(&'*') => {
                characters.next();
                while let Some(character) = characters.next() {
                    if character == '*' && characters.peek() == Some(&'/') {
                        characters.next();
                        break;
                    }
                }
                newlines = 0;
                previous_cr = false;
            }
            '\r' | '\n' => {
                if character != '\n' || !previous_cr {
                    newlines += 1;
                    if newlines == 2 {
                        return true;
                    }
                }
                previous_cr = character == '\r';
            }
            ' ' | '\t' => previous_cr = false,
            _ => {
                newlines = 0;
                previous_cr = false;
            }
        }
    }
    false
}
