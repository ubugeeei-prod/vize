//! Checked Text leaf construction at the existing canonical mint.

use super::{ArtifactError, Builder};
use crate::op::{Op, TextOp};
use vize_l0::{Box, Span, id::NodeId};

impl<'a> Builder<'a> {
    /// Build a leaf only after its span is checked against the current owner.
    pub fn text(&mut self, content: &'a str, span: Span) -> Result<NodeId, ArtifactError> {
        let id = self.prepare(span)?;
        self.mint();
        self.push(Op::Text(Box::new_in(
            TextOp { content, span },
            &self.allocator,
        )));
        Ok(id)
    }

}
