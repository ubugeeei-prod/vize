//! Input eligibility is specific to a normally completed original Model root.

use super::VaporBuilder;
use crate::decision::vapor::{VaporPart, VaporUnsupported, model};
use vize_l0::id::NodeId;
use vize_l2::{
    file::NativeFileModelState,
    op::{BindingOp, ElementOp, ModelOp},
};

impl<'owner, 'arena> VaporBuilder<'_, 'owner, 'arena> {
    pub(super) fn model_opening(&self, node: NodeId, element: &'owner ElementOp<'arena>) -> bool {
        let Some(setup) = self.setup else {
            return false;
        };
        let [original] = setup.file().native_models() else {
            return false;
        };
        let NativeFileModelState::Admitted(model_node) = original.state() else {
            return false;
        };
        let [binding @ BindingOp::Model(model)] = element.bindings.as_slice() else {
            return false;
        };
        self.frames.is_empty()
            && self.facts.roots.len() == 1
            && self.non_comments == 1
            && self.root_element == Some(node)
            && model::classify(setup, node, element, model_node, binding, model).is_ok()
    }

    pub(super) fn model_binding(
        &mut self,
        node: NodeId,
        binding: &'owner BindingOp<'arena>,
        model: &'owner ModelOp<'arena>,
    ) -> Result<(), VaporUnsupported> {
        let setup = self.setup.ok_or(VaporUnsupported::ModelOrigin)?;
        let [(parent, Some(element), true)] = self.frames.as_slice() else {
            return Err(VaporUnsupported::ModelShape);
        };
        let [
            VaporPart::Open {
                node: opening,
                element: actual,
                void: true,
            },
        ] = self.facts.parts.as_slice()
        else {
            return Err(VaporUnsupported::ModelShape);
        };
        if self.facts.roots.len() != 1
            || self.non_comments != 1
            || self.root_element != Some(*parent)
            || opening != parent
            || !core::ptr::eq(*actual, *element)
        {
            return Err(VaporUnsupported::ModelShape);
        }
        let model = model::classify(setup, *parent, element, node, binding, model)?;
        self.facts.parts.push(VaporPart::Model { model });
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
        if self.facts.parts.iter().any(|part| {
            matches!(part,
            VaporPart::Model { model } if model.resolution().node() == node)
        }) && !(self.frames.is_empty()
            && self.facts.roots.len() == 1
            && self.non_comments == 1
            && self.root_element == original.element_node()
            && matches!(
                self.facts.parts.as_slice(),
                [VaporPart::Open { void: true, .. }, VaporPart::Model { .. }]
            ))
        {
            self.reject(
                node,
                original.input().operand().value_span(),
                VaporUnsupported::ModelShape,
            );
        }
    }
}
