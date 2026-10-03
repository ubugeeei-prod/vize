//! Authenticate stored references; never look up or rewrite authored names.

use vize_l0::Span;
use vize_l2::resolution::{
    HandlerBindingRef, HandlerDeclarationKind, HandlerReference, HandlerResolution, HandlerScopeId,
};

pub(super) fn original(
    resolution: &HandlerResolution<'_>,
    index: usize,
    reference: &HandlerReference<'_>,
) -> bool {
    if !resolution
        .references()
        .get(index)
        .is_some_and(|actual| core::ptr::eq(actual, reference))
    {
        return false;
    }
    let HandlerBindingRef::Local(id) = reference.binding else {
        return false;
    };
    let Some(binding) = resolution
        .bindings()
        .get(id.index() as usize)
        .filter(|binding| binding.id == id && binding.name == reference.name)
    else {
        return false;
    };
    let source = resolution.input().operand().syntax().source();
    let plain = |span: Span| {
        source.text().get(span.start as usize..span.end as usize) == Some(binding.name)
            && source.authored_span(span).ok().and_then(|authored| {
                source
                    .authored_root()
                    .get(authored.start as usize..authored.end as usize)
            }) == Some(binding.name)
    };
    if !plain(reference.span) || !within(resolution, reference.scope, binding.scope) {
        return false;
    }
    if binding.kind == HandlerDeclarationKind::EventParameter {
        return id.index() == 0 && binding.scope.index() == 0 && binding.name == "$event";
    }
    // Vue's pinned statement transform prefixes root/sibling-block reads. An
    // active original block declaration preserves this writer's spelling.
    // Declaration identity does not promise a successful read before lexical
    // initialization: the unchanged body still performs JavaScript's TDZ checks.
    resolution.declarations().iter().any(|declaration| {
        declaration.binding == id
            && declaration.name == binding.name
            && declaration.kind == binding.kind
            && resolution
                .scopes()
                .get(declaration.scope.index() as usize)
                .is_some_and(|scope| scope.id == declaration.scope && scope.span.is_some())
            && within(resolution, reference.scope, declaration.scope)
            && (binding.kind == HandlerDeclarationKind::Var || binding.scope == declaration.scope)
            && plain(declaration.span)
    })
}

fn within(
    resolution: &HandlerResolution<'_>,
    mut scope: HandlerScopeId,
    ancestor: HandlerScopeId,
) -> bool {
    for _ in 0..resolution.scopes().len() {
        let Some(row) = resolution
            .scopes()
            .get(scope.index() as usize)
            .filter(|row| row.id == scope)
        else {
            return false;
        };
        if scope == ancestor {
            return true;
        }
        let Some(parent) = row.parent.filter(|parent| parent.index() < scope.index()) else {
            return false;
        };
        scope = parent;
    }
    false
}

#[cfg(test)]
mod tests;
