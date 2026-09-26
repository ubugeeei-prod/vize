//! Authored argument locations from the existing parse, without re-scanning.

use super::{MarkupBinding, MarkupBindingInner};
use crate::ir::ByteRange;
use crate::markup::l2::surface::SurfaceDirective;
use vize_relief::ExpressionNode;

impl MarkupBinding<'_> {
    /// The authored argument range, when the backend retains its spelling.
    /// Dynamic brackets and directive prefixes are outside this range.
    pub fn argument_range(&self) -> Option<ByteRange> {
        let attr = match self.inner {
            MarkupBindingInner::ReliefDirective(node) => {
                return match node.arg.as_ref()? {
                    ExpressionNode::Simple(arg) => Some(crate::markup::loc_to_range(&arg.loc)),
                    _ => None,
                };
            }
            MarkupBindingInner::L2Binding(binding) => binding.surface?,
            MarkupBindingInner::Surface {
                attr,
                static_name: None,
                ..
            } => attr,
            _ => return None,
        };
        let argument = SurfaceDirective::parse(attr.name.text)?.arg?;
        let offset = argument.as_ptr() as usize - attr.name.text.as_ptr() as usize;
        let start = self.range().start + offset as u32;
        Some(ByteRange::new(start, start + argument.len() as u32))
    }
}
