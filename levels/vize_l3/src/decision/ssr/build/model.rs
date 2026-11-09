//! Model value eligibility is attached only at the real canonical binding.

use super::super::{SsrUnsupported, model};
use super::SsrBuilder;
use vize_l0::id::NodeId;
use vize_l2::{
    file::NativeFileModelState,
    op::{BindingOp, ModelOp},
};

impl<'owner, 'arena> SsrBuilder<'_, 'owner, 'arena> {
    pub(super) fn model_binding(
        &mut self,
        node: NodeId,
        binding: &'owner BindingOp<'arena>,
        model: &'owner ModelOp<'arena>,
    ) -> Result<(), SsrUnsupported> {
        let setup = self.setup.ok_or(SsrUnsupported::FileBinding)?;
        let [(parent, Some(element), true)] = self.frames.as_slice() else {
            return Err(SsrUnsupported::Binding);
        };
        if self.roots != 1 || self.non_comments != 1 || self.root_element != Some(*parent) {
            return Err(SsrUnsupported::Binding);
        }
        let row = model::classify(setup, *parent, element, node, binding, model)?;
        self.facts.expressions.insert(node, row);
        Ok(())
    }

    pub(super) fn finish_model(&mut self) {
        let Some(setup) = self.setup else { return };
        let [original] = setup.file().native_models() else {
            return;
        };
        let NativeFileModelState::Admitted(node) = original.state() else {
            return;
        };
        if self
            .facts
            .expression(node)
            .is_some_and(|row| row.is_model())
            && (self.roots != 1
                || self.non_comments != 1
                || self.root_element != original.element_node())
        {
            self.reject(
                node,
                original.input().operand().value_span(),
                SsrUnsupported::Operation,
            );
        }
    }
}
