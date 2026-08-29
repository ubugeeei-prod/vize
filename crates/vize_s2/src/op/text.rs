//! `ui.text` / `ui.interpolation` payloads.

use vize_s0::Span;

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

/// `ui.interpolation` - an expression rendered as text.
#[derive(Debug)]
pub struct InterpolationOp<'a> {
    /// The rendered expression.
    pub expression: ExprRef<'a>,
    /// The interpolation's full source range, delimiters included.
    pub span: Span,
}

/// See [`crate::op`] for the guard rationale.
#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<TextOp<'_>>() == 24);
    assert!(core::mem::size_of::<InterpolationOp<'_>>() == 24);
};
