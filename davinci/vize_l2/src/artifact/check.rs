//! Construction-time checks over the actual native tree, never a dump.

use alloc::collections::BTreeSet;
use oxc_span::GetSpan;
use vize_l0::{Span, id::NodeId};

use super::{ArtifactError, ArtifactParts, IfShape};
use crate::expr::ExprRef;
use crate::op::{BindingOp, Op};
use crate::scope::ScopeOrigin;
use crate::walk::{NodeEvent, NodeRef, PageWalk};

pub(super) fn check(parts: &ArtifactParts<'_>) -> Result<u32, ArtifactError> {
    u32::try_from(parts.source.len()).map_err(|_| ArtifactError::SourceLimit)?;
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
    .map_err(|_| ArtifactError::NodeLimit)?;
    if let Some(error) = first_error {
        return Err(error);
    }
    let count = walk.minted();
    for (record, provenance) in parts.provenance.iter().enumerate() {
        span(parts.source, provenance.node, provenance.span)?;
        if let Some(node) = provenance.node
            && node.index() >= count
        {
            return Err(ArtifactError::DanglingProvenance { record, node });
        }
    }
    let mut tags = BTreeSet::new();
    let scope_count = parts.scopes.len();
    for (node, facts) in parts.scopes.sorted_entries() {
        if node.index() >= count {
            return Err(ArtifactError::DanglingScope { node });
        }
        if facts.tag.index() as usize >= scope_count || !tags.insert(facts.tag) {
            return Err(ArtifactError::InvalidScopeTag {
                node,
                tag: facts.tag,
            });
        }
    }
    Ok(count)
}

fn check_node(
    parts: &ArtifactParts<'_>,
    id: NodeId,
    node: NodeRef<'_, '_>,
    owner: Option<Span>,
) -> Result<(), ArtifactError> {
    owned_span(parts.source, id, node.span(), owner)?;
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
            return Err(ArtifactError::InvalidIf {
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
                return Err(ArtifactError::InvalidIf {
                    node: id,
                    shape: IfShape::LeadingElse,
                });
            } else if index != last {
                return Err(ArtifactError::InvalidIf {
                    node: id,
                    shape: IfShape::NonTrailingElse,
                });
            }
        }
    }
    let introduces = match node {
        NodeRef::Op(Op::For(_)) => true,
        NodeRef::Binding(BindingOp::SlotContent(it)) => it.params.is_some(),
        NodeRef::Binding(BindingOp::VueSlotScope(it)) => it.params.is_some(),
        _ => false,
    };
    if let Some(scope) = parts.scopes.get(id) {
        if !introduces {
            return Err(ArtifactError::InvalidScopeSite { node: id });
        }
        for binding in &scope.bindings {
            if let ScopeOrigin::Authored { span } = binding.origin {
                owned_span(parts.source, id, span, Some(node.span()))?;
            }
        }
    } else if introduces {
        return Err(ArtifactError::MissingScope { node: id });
    }
    Ok(())
}

pub(super) fn expression(
    source: &str,
    id: NodeId,
    expr: ExprRef<'_>,
    owner: Span,
) -> Result<(), ArtifactError> {
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
            return Err(ArtifactError::MismatchedJsSource {
                node: id,
                span: js.span,
            });
        }
        let ast = js.ast.span();
        if js.ast_span_to_source(ast).is_none() {
            return Err(ArtifactError::InvalidJsSpan {
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
) -> Result<(), ArtifactError> {
    span(source, Some(id), range)?;
    if let Some(owner) = owner
        && (range.start < owner.start || range.end > owner.end)
    {
        return Err(ArtifactError::OutsideOwner {
            node: id,
            span: range,
            owner,
        });
    }
    Ok(())
}

pub(super) fn span(source: &str, node: Option<NodeId>, range: Span) -> Result<(), ArtifactError> {
    if range.start > range.end
        || source
            .get(range.start as usize..range.end as usize)
            .is_none()
    {
        return Err(ArtifactError::InvalidSpan { node, span: range });
    }
    Ok(())
}
