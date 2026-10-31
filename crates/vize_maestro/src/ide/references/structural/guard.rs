//! Reject edits that would capture another lexical symbol or unresolved name.

use super::snapshot::NamedReference;
use oxc_semantic::Scoping;
use oxc_syntax::symbol::SymbolId;
use vize_l0::FxHashSet;

pub(super) fn rename_is_safe(
    scoping: &Scoping,
    references: &[NamedReference],
    selected: &FxHashSet<SymbolId>,
    new_name: &str,
) -> bool {
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
        for named in references {
            if named.name.as_str() != new_name {
                continue;
            }
            let reference = scoping.get_reference(named.reference);
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
