//! The JSON / JSONC value tree and the comment-placement helper shared by the
//! parser and printer.

use vize_carton::String;

pub(super) enum Node {
    /// `{}` (compact when empty) or a multi-line mapping.
    Object {
        members: Vec<Member>,
        /// Comments on their own line after the last member, before `}`.
        dangling: Vec<Comment>,
    },
    /// `[]` (compact when empty) or a multi-line sequence.
    Array {
        elements: Vec<Element>,
        dangling: Vec<Comment>,
    },
    /// A string, number, `true`, `false`, or `null`, copied verbatim.
    Scalar(String),
}

pub(super) struct Member {
    /// Comments printed on their own lines before the key.
    pub(super) leading: Vec<Comment>,
    /// The key, including its surrounding quotes, verbatim.
    pub(super) key: String,
    pub(super) value: Node,
    /// Comments printed on the same line after the value (and comma).
    pub(super) trailing: Vec<Comment>,
}

pub(super) struct Element {
    pub(super) leading: Vec<Comment>,
    pub(super) value: Node,
    pub(super) trailing: Vec<Comment>,
}

pub(super) struct Comment {
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
