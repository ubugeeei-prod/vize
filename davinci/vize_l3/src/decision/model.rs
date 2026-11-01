//! Shared original-Model identity; target access policies remain separate.

use vize_l0::id::NodeId;
use vize_l2::{
    expr::ExprRef,
    file::{
        DeclarationKind, FileArtifact, FileResolution, InitializerKind, Namespace,
        NativeFileModelState,
    },
    op::{BindingOp, ElementOp, ModelOp},
    resolution::Usage,
};

/// Authenticate the normally completed original factory, not numeric IDs alone.
/// Callers separately establish their canonical root/target event and exposure.
pub(super) fn checked<'owner, 'arena>(
    file: &'owner FileArtifact<'arena>,
    element_node: NodeId,
    element: &'owner ElementOp<'arena>,
    node: NodeId,
    binding: &'owner BindingOp<'arena>,
    model: &'owner ModelOp<'arena>,
) -> Option<FileResolution<'owner, 'arena>> {
    let [actual_binding] = element.bindings.as_slice() else {
        return None;
    };
    if model.argument.is_some()
        || element.tag != "input"
        || element.namespace != vize_l2::op::Namespace::Html
        || !element.attributes.is_empty()
        || !element.children.ops.is_empty()
        || !core::ptr::eq(actual_binding, binding)
    {
        return None;
    }
    let original = file.native_model(node)?;
    if original.state() != NativeFileModelState::Admitted(node)
        || !original.accepts(model)
        || !original.accepts_element(element_node, element)
        || original.element_node() != Some(element_node)
        || original.input().attribute_span() != Some(model.span)
    {
        return None;
    }
    match (
        original.input().operand().trim_span(),
        model.attributes.as_slice(),
    ) {
        (None, []) => {}
        (Some(span), [attribute]) => {
            let authored = file
                .artifact()
                .source()
                .get(span.start as usize..span.end as usize)?;
            if attribute.span != span
                || attribute.value.is_some()
                || attribute.name != "trim"
                || !core::ptr::eq(attribute.name, authored)
            {
                return None;
            }
        }
        _ => return None,
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
    let table = resolution.table()?;
    let retained = table.expression();
    let coordinates = match (retained.coordinates, read.coordinates) {
        (None, None) => true,
        (Some(left), Some(right)) => core::ptr::eq(left, right),
        _ => false,
    };
    if resolution.scope() != Some(original.scope())
        || !core::ptr::eq(retained.ast, read.ast)
        || !core::ptr::eq(retained.source, read.source)
        || retained.span != read.span
        || !coordinates
        || !file
            .scopes()
            .get(original.scope().index() as usize)
            .is_some_and(|row| row.id == original.scope())
    {
        return None;
    }
    let [occurrence] = table.occurrences() else {
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
