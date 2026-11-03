//! Checked Text leaf construction at the existing canonical mint.

use super::{ArtifactError, Builder};
use crate::op::{Op, TextOp};
use core::ptr::NonNull;
use vize_l0::{Box, Span, id::NodeId};

/// Transient receipt from the exact existing leaf allocation and mint.
pub(crate) struct TextAllocation<'a>(NonNull<TextOp<'a>>);
impl<'a> TextAllocation<'a> {
    pub(crate) fn pointer(self) -> NonNull<TextOp<'a>> {
        self.0
    }
}

impl<'a> Builder<'a> {
    /// Build a leaf only after its span is checked against the current owner.
    pub fn text(&mut self, content: &'a str, span: Span) -> Result<NodeId, ArtifactError> {
        self.native_text(content, span).map(|(node, _)| node)
    }

    pub(super) fn native_text(
        &mut self,
        content: &'a str,
        span: Span,
    ) -> Result<(NodeId, TextAllocation<'a>), ArtifactError> {
        let id = self.prepare(span)?;
        self.mint();
        let owner = Box::new_in(TextOp { content, span }, &self.allocator);
        let allocation = TextAllocation(NonNull::from(owner.as_ref()));
        self.push(Op::Text(owner));
        Ok((id, allocation))
    }
}
