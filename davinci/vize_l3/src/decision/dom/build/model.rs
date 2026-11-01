//! Authenticate the original Model's dedicated mutable setup access.

use super::{
    DomBinding, DomBuilder, DomDependency, DomExpressionFacts, DomUnsupported, PropertyRole,
    ValueKind,
};
use crate::decision::dom::{
    file::{DomFileExpression, matches_expression},
    vue::policy::FileReads,
};
use vize_l0::id::NodeId;
use vize_l2::{
    expr::ExprRef,
    file::{DeclarationKind, FileResolution, InitializerKind, Namespace, NativeFileModelState},
    op::{BindingOp, ModelOp, Op},
    resolution::Usage,
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
        self.demand(DomDependency::TextModel);
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
        if !R::NATIVE_SETUP
            || self.root_count != 1
            || model.argument.is_some()
            || !model.attributes.is_empty()
        {
            return None;
        }
        let [(element_node, frame)] = self.frames.as_slice() else {
            return None;
        };
        let Op::Element(element) = frame.node.op else {
            return None;
        };
        let [actual_binding] = element.bindings.as_slice() else {
            return None;
        };
        if element.tag != "input"
            || element.namespace != vize_l2::op::Namespace::Html
            || !element.attributes.is_empty()
            || !element.children.ops.is_empty()
            || !core::ptr::eq(actual_binding, binding)
            || !frame.binding_names.is_empty()
        {
            return None;
        }
        let file = self.file?;
        let original = file.native_model(node)?;
        if original.state() != NativeFileModelState::Admitted(node)
            || !original.accepts(model)
            || !original.accepts_element(*element_node, element)
            || original.element_node() != Some(*element_node)
            || original.input().attribute_span() != Some(model.span)
        {
            return None;
        }
        let (ExprRef::Js(read), ExprRef::Js(write)) = (model.contract.read, model.contract.write)
        else {
            return None;
        };
        if !core::ptr::eq(read, write)
            || !original.input().matches_expression(read)
            || !core::ptr::eq(
                original.input().operand().syntax().source().authored_root(),
                file.artifact().source(),
            )
        {
            return None;
        }
        let resolution = file.expression(node)?;
        if resolution.scope() != Some(original.scope())
            || !matches_expression(resolution, read)
            || !file
                .scopes()
                .get(original.scope().index() as usize)
                .is_some_and(|row| row.id == original.scope())
        {
            return None;
        }
        let [occurrence] = resolution.table()?.occurrences() else {
            return None;
        };
        let target = resolution.binding(occurrence.binding)?;
        let declaration = target.declaration()?;
        let [unit] = file.units() else { return None };
        let identifier = read.ast.get_identifier_reference()?;
        let authored = read.authored_span(occurrence.span)?;
        if !resolution.accepts(target)
            || original.binding() != Some(target.id())
            || occurrence.usage != Usage::ReadWrite
            || occurrence.shorthand
            || occurrence.constructor
            || declaration.id != occurrence.binding
            || declaration.namespace != Namespace::Value
            || !declaration.is_direct_program()
            || declaration.unit != unit.id
            || declaration.scope != original.scope()
            || unit.scope != original.scope()
            || !matches!(
                declaration.kind,
                DeclarationKind::Let | DeclarationKind::Var
            )
            || declaration.initializer != InitializerKind::PrimitiveLiteral
            || declaration.name != occurrence.name
            || identifier.name.as_str() != occurrence.name
            || read.source != occurrence.name
            || read.ast_span_to_source(identifier.span) != Some(occurrence.span)
            || file
                .artifact()
                .source()
                .get(authored.start as usize..authored.end as usize)
                != Some(occurrence.name)
        {
            return None;
        }
        Some(resolution)
    }
}
