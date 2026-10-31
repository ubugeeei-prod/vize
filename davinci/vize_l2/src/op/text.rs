//! `ui.text` / `ui.interpolation` / `ui.comment` payloads.

use vize_l0::Span;

use crate::expr::ExprRef;

/// `ui.text` - static text content.
#[derive(Debug)]
pub struct TextOp<'a> {
    /// The text exactly as it should render, a slice of the source (or of
    /// a lowering's arena where entity decoding rewrote it).
    pub content: &'a str,
    /// The text's source range.
    pub span: Span,
}

/// The producer-owned interpretation of one dynamic UI child.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterpolationRole {
    /// Ordinary template interpolation, rendered through display-string conversion.
    TemplateText,
    /// A genuine JSX child value, normalized by the target's child renderer.
    JsxChild,
}

/// `ui.interpolation` - a dynamic child with an explicit producer-owned role.
#[derive(Debug)]
pub struct InterpolationOp<'a> {
    /// The rendered expression.
    pub expression: ExprRef<'a>,
    /// Text interpolation and JSX values have different runtime contracts.
    pub role: InterpolationRole,
    /// The interpolation's full source range, delimiters included.
    pub span: Span,
}

/// `ui.comment` - an authored template comment kept for DOM output.
#[derive(Debug)]
pub struct CommentOp<'a> {
    /// The comment body, without the `<!--` / `-->` delimiters.
    pub content: &'a str,
    /// The comment's full source range.
    pub span: Span,
}

/// See [`crate::op`] for the guard rationale.
#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<TextOp<'_>>() == 24);
    // The explicit role adds eight aligned bytes; this is not a performance claim.
    assert!(core::mem::size_of::<InterpolationOp<'_>>() == 32);
    assert!(core::mem::size_of::<CommentOp<'_>>() == 24);
};
