//! Authenticate the original Model's dedicated mutable setup access.

use super::{
    DomBinding, DomBuilder, DomDependency, DomExpressionFacts, DomUnsupported, PropertyRole,
    ValueKind,
};
use crate::decision::dom::{file::DomFileExpression, vue::policy::FileReads};
use vize_l0::id::NodeId;
use vize_l2::{
    file::FileResolution,
    op::{BindingOp, ModelOp, Op},
};

impl<'owner, 'arena, F: DomExpressionFacts, R: FileReads<'owner, 'arena>>
    DomBuilder<'_, 'owner, 'arena, F, R>
{
    pub(super) fn model_binding(
        &mut self,
        node: NodeId,
        binding: &'owner BindingOp<'arena>,
        model: &'owner ModelOp<'arena>,
    ) {
        let Some(resolution) = self.checked_model(node, binding, model) else {
            self.reject(node, model.span, DomUnsupported::FileBinding);
            return;
        };
        let dependency = match resolution
            .file()
            .native_model(node)
            .and_then(|original| original.kind())
        {
            Some(vize_l2::file::NativeFileModelKind::Text) => DomDependency::TextModel,
            Some(vize_l2::file::NativeFileModelKind::Checkbox) => DomDependency::CheckboxModel,
            None => {
                self.reject(node, model.span, DomUnsupported::FileBinding);
                return;
            }
        };
        let Some((_, frame)) = self.frames.last_mut() else {
            self.reject(node, model.span, DomUnsupported::Binding);
            return;
        };
        frame.node.changes.properties = true;
        frame.node.dynamic_property_bindings.push(node);
        // Neutral complete-File resolution is shared storage, not a generic
        // read or mutation permission. Only the checked Model role below
        // allows deriving the separate borrowed read/write receipt.
        self.facts
            .file_expressions
            .insert(node, DomFileExpression { resolution });
        self.facts.bindings.insert(
            node,
            DomBinding {
                binding,
                role: PropertyRole::Model,
                value: ValueKind::FileDependent,
            },
        );
        self.demand(dependency);
        self.demand(DomDependency::DirectiveApplication);
    }

    fn checked_model(
        &self,
        node: NodeId,
        binding: &'owner BindingOp<'arena>,
        model: &'owner ModelOp<'arena>,
    ) -> Option<FileResolution<'owner, 'arena>> {
        // Only the genuine selected setup specialization gets this separate
        // receipt. SelectedReads::classify continues to reject every write.
        if !R::NATIVE_SETUP || self.root_count != 1 {
            return None;
        }
        let [(element_node, frame)] = self.frames.as_slice() else {
            return None;
        };
        let Op::Element(element) = frame.node.op else {
            return None;
        };
        if !frame.binding_names.is_empty() {
            return None;
        }
        crate::decision::model::checked(self.file?, *element_node, element, node, binding, model)
    }
}
