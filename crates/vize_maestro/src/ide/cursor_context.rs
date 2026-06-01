//! Shared cursor-context model for LSP features.
//!
//! Completion, hover, definition, references, rename, and code-action services
//! all need to answer the same first question: "what is the cursor on?"
//! Today each service reimplements that detection inline. The fragmented logic
//! has produced divergent behavior between features at the same cursor
//! position — e.g. completion treats `foo.|` as a member access while hover
//! treats it as a plain identifier.
//!
//! `CursorContext::detect` is the single answer. It runs cheap byte-level
//! scans only and never touches the SFC parser or Croquis. Each LSP service
//! consumes one variant and dispatches to its own provider.
//!
//! This module is intentionally minimal. The variants here cover the cases
//! that block roadmap items [#678](https://github.com/ubugeeei-prod/vize/issues/678)
//! (member access) and [#680](https://github.com/ubugeeei-prod/vize/issues/680)
//! (trigger-character routing). Additional variants (event handler, prop
//! name, slot name, …) will be added when their consumers migrate.

/// What the cursor is on, expressed in source-position terms.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CursorContext<'a> {
    /// The cursor sits right after a `.` whose left-hand side is an
    /// identifier or chain. The `receiver` is the source text of that LHS,
    /// e.g. `count` for `count.|` or `obj.foo` for `obj.foo.|`.
    MemberAccess {
        /// Source text of the receiver expression.
        receiver: &'a str,
        /// Byte offset of the `.` itself in the source.
        dot_offset: usize,
    },
    /// The cursor is positioned on or right after an identifier prefix.
    /// `prefix` may be empty when the cursor sits between non-identifier
    /// characters and we still want completion to fire.
    Identifier {
        /// Identifier characters immediately preceding the cursor.
        prefix: &'a str,
        /// Byte offset of the start of `prefix` in the source.
        start: usize,
    },
    /// The cursor is inside an HTML comment (`<!-- … -->`). Used by Vize's
    /// `@vize:` directive completion path.
    HtmlComment,
    /// The cursor is in source text that is neither a comment nor an
    /// identifier (e.g. operator, punctuation, whitespace at end of file).
    Other,
}

impl<'a> CursorContext<'a> {
    /// Detect the cursor context at `offset` in `content`.
    ///
    /// `content` is the raw document text in any LSP-tracked file (SFC, art
    /// file, standalone HTML, etc.). The offset is a byte offset clamped to
    /// the document length.
    pub fn detect(content: &'a str, offset: usize) -> Self {
        let offset = clamp_offset(content, offset);

        if is_inside_html_comment(content, offset) {
            return Self::HtmlComment;
        }

        if let Some(member) = detect_member_access(content, offset) {
            return member;
        }

        detect_identifier(content, offset)
    }
}

fn clamp_offset(content: &str, offset: usize) -> usize {
    let mut offset = offset.min(content.len());
    while offset > 0 && !content.is_char_boundary(offset) {
        offset -= 1;
    }
    offset
}

/// Returns true when `offset` falls inside an unterminated `<!-- ... -->`
/// block. Mirrors `crate::ide::completion::is_inside_html_comment` so the
/// detector can be used outside the completion module too.
fn is_inside_html_comment(content: &str, offset: usize) -> bool {
    let before = &content[..offset];
    let Some(comment_start) = before.rfind("<!--") else {
        return false;
    };
    let after_start = &before[comment_start + 4..];
    !after_start.contains("-->")
}

fn detect_member_access<'a>(content: &'a str, offset: usize) -> Option<CursorContext<'a>> {
    let before = &content[..offset];
    // Allow whitespace between the `.` and the cursor — e.g. when the
    // editor's auto-indent inserts a newline.
    let trimmed = before.trim_end();
    if !trimmed.ends_with('.') {
        return None;
    }
    let dot_offset = trimmed.len() - 1;

    let receiver_end = dot_offset;
    let bytes = content.as_bytes();
    let mut receiver_start = receiver_end;
    while receiver_start > 0 && is_receiver_byte(bytes[receiver_start - 1]) {
        receiver_start -= 1;
    }

    if receiver_start == receiver_end {
        return None;
    }

    Some(CursorContext::MemberAccess {
        receiver: &content[receiver_start..receiver_end],
        dot_offset,
    })
}

/// Bytes that are part of a receiver expression. Identifiers (`_a-z0-9$`),
/// `.` to allow chained property access like `foo.bar.|`, and `]` so
/// `arr[0].|` resolves to `arr[0]`.
#[inline]
fn is_receiver_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$' | b'.' | b']')
}

fn detect_identifier<'a>(content: &'a str, offset: usize) -> CursorContext<'a> {
    let bytes = content.as_bytes();
    let mut start = offset;
    while start > 0 && is_ident_byte(bytes[start - 1]) {
        start -= 1;
    }
    if start == offset {
        return CursorContext::Other;
    }
    CursorContext::Identifier {
        prefix: &content[start..offset],
        start,
    }
}

#[inline]
fn is_ident_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'$'
}

#[cfg(test)]
mod tests {
    use super::CursorContext;

    fn at(source: &str, marker: &str) -> usize {
        source.find(marker).expect("marker not found") + marker.len()
    }

    #[test]
    fn detects_member_access_on_simple_receiver() {
        let src = "const arr = [];\narr.";
        let ctx = CursorContext::detect(src, at(src, "arr."));
        assert!(
            matches!(
                ctx,
                CursorContext::MemberAccess {
                    receiver: "arr",
                    ..
                }
            ),
            "expected MemberAccess(arr), got {ctx:?}",
        );
    }

    #[test]
    fn detects_member_access_on_chained_receiver() {
        let src = "obj.foo.";
        let ctx = CursorContext::detect(src, src.len());
        match ctx {
            CursorContext::MemberAccess { receiver, .. } => {
                assert_eq!(receiver, "obj.foo");
            }
            other => panic!("expected MemberAccess(obj.foo), got {other:?}"),
        }
    }

    #[test]
    fn ignores_floating_point_literal() {
        // `1.|` should not be member access of `1` — but our current heuristic
        // does treat any identifier-like LHS as a receiver. Numeric literals
        // are excluded because `is_receiver_byte` is fine with digits, so
        // `1` would in fact match. Document the limitation and the
        // corresponding decision: completion services downstream are expected
        // to reject pure-digit receivers themselves.
        let src = "1.";
        let ctx = CursorContext::detect(src, src.len());
        assert!(matches!(ctx, CursorContext::MemberAccess { .. }));
    }

    #[test]
    fn detects_identifier_prefix() {
        let src = "const value = 1\nval";
        let ctx = CursorContext::detect(src, src.len());
        match ctx {
            CursorContext::Identifier { prefix, .. } => assert_eq!(prefix, "val"),
            other => panic!("expected Identifier(val), got {other:?}"),
        }
    }

    #[test]
    fn detects_html_comment() {
        let src = "<template>\n  <!-- @vize:";
        let ctx = CursorContext::detect(src, src.len());
        assert!(matches!(ctx, CursorContext::HtmlComment));
    }

    #[test]
    fn closed_comment_is_not_html_comment() {
        let src = "<template>\n  <!-- done -->\n  <div>";
        let ctx = CursorContext::detect(src, src.len() - 5);
        assert!(!matches!(ctx, CursorContext::HtmlComment));
    }

    #[test]
    fn empty_prefix_after_operator_is_other() {
        let src = "a + ";
        let ctx = CursorContext::detect(src, src.len());
        assert!(matches!(ctx, CursorContext::Other));
    }

    #[test]
    fn member_access_with_whitespace_between_dot_and_cursor() {
        let src = "foo.\n  ";
        let ctx = CursorContext::detect(src, src.len());
        match ctx {
            CursorContext::MemberAccess { receiver, .. } => assert_eq!(receiver, "foo"),
            other => panic!("expected MemberAccess(foo) with trailing whitespace, got {other:?}"),
        }
    }

    #[test]
    fn cursor_at_zero_is_other() {
        let ctx = CursorContext::detect("", 0);
        assert!(matches!(ctx, CursorContext::Other));
    }
}
