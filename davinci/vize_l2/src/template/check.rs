//! Construction-time checks over the actual native tree, never a dump.

use alloc::collections::BTreeSet;
use oxc_span::GetSpan;
use vize_l0::{Span, id::NodeId};

use super::{IfShape, L2TemplateError, L2TemplateParts};
use crate::expr::{ExprRef, OpaqueReason};
use crate::op::{BindingOp, ForOp, HostFor, Op};
use crate::scope::ScopeOrigin;
use crate::walk::{NodeEvent, NodeRef, PageWalk};

pub(super) fn check(parts: &L2TemplateParts<'_>) -> Result<u32, L2TemplateError> {
    u32::try_from(parts.source.len()).map_err(|_| L2TemplateError::SourceLimit)?;
    let mut walk = PageWalk::new();
    let mut first_error = None;
    crate::walk::visit_events(&mut walk, &parts.root.ops, &mut |event| {
        if first_error.is_none()
            && let NodeEvent::Enter {
                id,
                node,
                owner_span,
                ..
            } = event
        {
            first_error = check_node(parts, id, node, owner_span).err();
        }
    })
    .map_err(|_| L2TemplateError::NodeLimit)?;
    if let Some(error) = first_error {
        return Err(error);
    }
    let count = walk.minted();
    for (record, provenance) in parts.provenance.iter().enumerate() {
        span(parts.source, provenance.node, provenance.span)?;
        if let Some(node) = provenance.node
            && node.index() >= count
        {
            return Err(L2TemplateError::DanglingProvenance { record, node });
        }
    }
    let mut tags = BTreeSet::new();
    let scope_count = parts.scopes.len();
    for (node, facts) in parts.scopes.sorted_entries() {
        if node.index() >= count {
            return Err(L2TemplateError::DanglingScope { node });
        }
        if facts.tag.index() as usize >= scope_count || !tags.insert(facts.tag) {
            return Err(L2TemplateError::InvalidScopeTag {
                node,
                tag: facts.tag,
            });
        }
    }
    let fact_nodes = (parts.texts.iter().map(|(node, _)| node))
        .chain(parts.for_facts.iter().map(|(node, _)| node))
        .chain(parts.if_facts.iter().map(|(node, _)| node))
        .chain(parts.branch_fragments.iter().map(|(node, _)| node))
        .chain(parts.loop_fragments.iter().map(|(node, _)| node));
    for node in fact_nodes {
        if node.index() >= count {
            return Err(L2TemplateError::DanglingFact { node });
        }
    }
    Ok(count)
}

fn check_node(
    parts: &L2TemplateParts<'_>,
    id: NodeId,
    node: NodeRef<'_, '_>,
    owner: Option<Span>,
) -> Result<(), L2TemplateError> {
    owned_span(parts.source, id, node.span(), owner)?;
    if let NodeRef::Op(Op::OriginalFor(original)) = node
        && original.id().node() != id
    {
        return Err(L2TemplateError::InvalidOriginalForReference {
            node: id,
            original: original.id().node(),
        });
    }
    if let NodeRef::Binding(BindingOp::On(on)) = node
        && let Some(handler) = on.handler.and_then(crate::op::OnHandlerRef::body)
        && handler.node() != id
    {
        return Err(L2TemplateError::InvalidHandlerReference {
            node: id,
            handler: handler.node(),
        });
    }
    for attr in node.attributes() {
        owned_span(parts.source, id, attr.span, Some(node.span()))?;
    }
    let mut error = None;
    node.for_each_expression(&mut |expr| {
        if error.is_none() {
            error = expression(parts.source, id, expr, node.span()).err();
        }
    });
    if let Some(error) = error {
        return Err(error);
    }
    if let NodeRef::Op(Op::If(if_op)) = node {
        if if_op.branches.is_empty() {
            return Err(L2TemplateError::InvalidIf {
                node: id,
                shape: IfShape::Empty,
            });
        }
        let last = if_op.branches.len().saturating_sub(1);
        for (index, branch) in if_op.branches.iter().enumerate() {
            owned_span(parts.source, id, branch.span, Some(if_op.span))?;
            if let Some(expr) = branch.condition {
                owned_span(parts.source, id, expr.span(), Some(branch.span))?;
            } else if index == 0 {
                return Err(L2TemplateError::InvalidIf {
                    node: id,
                    shape: IfShape::LeadingElse,
                });
            } else if index != last {
                return Err(L2TemplateError::InvalidIf {
                    node: id,
                    shape: IfShape::NonTrailingElse,
                });
            }
        }
    }
    if let NodeRef::Op(Op::For(for_op)) = node
        && let Some(host) = &for_op.host
    {
        host_for(parts.source, id, for_op, host)?;
    }
    let introduces = match node {
        NodeRef::Op(Op::For(_) | Op::OriginalFor(_)) => true,
        NodeRef::Binding(BindingOp::SlotContent(it)) => it.params.is_some(),
        NodeRef::Binding(BindingOp::VueSlotScope(it)) => it.params.is_some(),
        _ => false,
    };
    if let Some(scope) = parts.scopes.get(id) {
        if !introduces {
            return Err(L2TemplateError::InvalidScopeSite { node: id });
        }
        for binding in &scope.bindings {
            if let ScopeOrigin::Authored { span } = binding.origin {
                owned_span(parts.source, id, span, Some(node.span()))?;
            }
        }
    } else if introduces {
        return Err(L2TemplateError::MissingScope { node: id });
    }
    Ok(())
}

/// A host loop keeps its head as host AST; its binding names only the
/// collection and the authored left side, never a re-split key or index.
fn host_for(
    source: &str,
    id: NodeId,
    for_op: &ForOp<'_>,
    host: &HostFor<'_>,
) -> Result<(), L2TemplateError> {
    let left = host.left.span();
    let left = Span::new(left.start, left.end);
    owned_span(source, id, left, Some(for_op.span))?;
    if let Some(index) = host.index {
        owned_span(
            source,
            id,
            Span::new(index.span.start, index.span.end),
            Some(for_op.span),
        )?;
    }
    let binding = &for_op.binding;
    let value = matches!(binding.value, ExprRef::Opaque(opaque)
        if opaque.reason == OpaqueReason::ForValue && opaque.span == left && opaque.source == host.left_text);
    let authored = source.get(left.start as usize..left.end as usize) == Some(host.left_text);
    if !(value && authored && matches!(binding.source, ExprRef::Js(_)))
        || binding.key.is_some()
        || binding.index.is_some()
    {
        return Err(L2TemplateError::InvalidHostFor { node: id });
    }
    Ok(())
}

#[cfg(test)]
mod original_for_tests;

pub(super) fn expression(
    source: &str,
    id: NodeId,
    expr: ExprRef<'_>,
    owner: Span,
) -> Result<(), L2TemplateError> {
    owned_span(source, id, expr.span(), Some(owner))?;
    // A filter is a dialect payload, not an excuse to omit its authored
    // subranges. Iteration avoids recursion through user-constructed chains.
    let mut current = expr;
    while let ExprRef::Filter(filter) = current {
        for app in &filter.filters {
            owned_span(source, id, app.span, Some(current.span()))?;
        }
        owned_span(source, id, filter.base.span(), Some(current.span()))?;
        current = filter.base;
    }
    if let ExprRef::Js(js) = current {
        if js.coordinates.is_some() && !js.matches_authored_source(source) {
            return Err(L2TemplateError::MismatchedJsSource {
                node: id,
                span: js.span,
            });
        }
        let ast = js.ast.span();
        if js.ast_span_to_source(ast).is_none() {
            return Err(L2TemplateError::InvalidJsSpan {
                node: id,
                span: Span::new(ast.start, ast.end),
            });
        }
    }
    Ok(())
}

pub(super) fn owned_span(
    source: &str,
    id: NodeId,
    range: Span,
    owner: Option<Span>,
) -> Result<(), L2TemplateError> {
    span(source, Some(id), range)?;
    if let Some(owner) = owner
        && (range.start < owner.start || range.end > owner.end)
    {
        return Err(L2TemplateError::OutsideOwner {
            node: id,
            span: range,
            owner,
        });
    }
    Ok(())
}

pub(super) fn span(source: &str, node: Option<NodeId>, range: Span) -> Result<(), L2TemplateError> {
    if range.start > range.end
        || source
            .get(range.start as usize..range.end as usize)
            .is_none()
    {
        return Err(L2TemplateError::InvalidSpan { node, span: range });
    }
    Ok(())
}
