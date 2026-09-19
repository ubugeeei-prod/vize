//! Reject edits that would capture another lexical symbol or unresolved name.

use oxc_ast::AstKind;
use oxc_semantic::Semantic;
use oxc_syntax::symbol::SymbolId;
use vize_s0::FxHashSet;

pub(super) fn rename_is_safe(
    semantic: &Semantic<'_>,
    selected: &FxHashSet<SymbolId>,
    new_name: &str,
) -> bool {
    let scoping = semantic.scoping();
    let name = new_name.into();
    for &symbol in selected {
        let scope = scoping.symbol_scope_id(symbol);
        if scoping
            .get_binding(scope, name)
            .is_some_and(|id| !selected.contains(&id))
        {
            return false;
        }
        for reference in scoping.get_resolved_references(symbol) {
            if scoping
                .find_binding(reference.scope_id(), name)
                .is_some_and(|id| {
                    !selected.contains(&id)
                        && scoping.scope_is_descendant_of(scoping.symbol_scope_id(id), scope)
                })
            {
                return false;
            }
        }
        for node in semantic.nodes().iter() {
            let AstKind::IdentifierReference(id) = node.kind() else {
                continue;
            };
            if id.name != name {
                continue;
            }
            let Some(reference) = id.reference_id.get().map(|id| scoping.get_reference(id)) else {
                continue;
            };
            if scoping
                .scope_ancestors(reference.scope_id())
                .any(|ancestor| ancestor == scope)
                && !reference.symbol_id().is_some_and(|id| {
                    selected.contains(&id)
                        || scoping.scope_is_descendant_of(scoping.symbol_scope_id(id), scope)
                })
            {
                return false;
            }
        }
    }
    true
}
