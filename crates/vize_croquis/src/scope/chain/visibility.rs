//! Binding visibility from either a coordinate or an explicit semantic scope.

use super::{FxHashSet, ScopeBinding, ScopeChain, ScopeId, ScopeKind};

impl ScopeChain {
    /// Collect every binding visible at `offset`, walking from the deepest
    /// containing scope outward through its parents.
    ///
    /// Inner-scope bindings shadow outer ones — the first occurrence of each
    /// name wins. Returns `(name, binding, scope_kind)` triples. The order is
    /// inner-most first, which the LSP uses to prioritize closer scopes in
    /// completion sort order.
    pub fn bindings_visible_at(&self, offset: u32) -> Vec<(&str, ScopeBinding, ScopeKind)> {
        let Some(start_id) = self.scope_at_offset(offset) else {
            return Vec::new();
        };

        self.bindings_visible_from(start_id)
    }

    /// Collect bindings from a known semantic scope, without comparing offsets
    /// from script and template coordinate spaces.
    pub fn bindings_visible_from(&self, start_id: ScopeId) -> Vec<(&str, ScopeBinding, ScopeKind)> {
        let mut seen: FxHashSet<&str> = FxHashSet::default();
        let mut out: Vec<(&str, ScopeBinding, ScopeKind)> = Vec::new();
        let mut stack: Vec<ScopeId> = Vec::new();
        let mut to_visit: Vec<ScopeId> = vec![start_id];

        while let Some(id) = to_visit.pop() {
            // Avoid revisiting the same scope through multiple parent paths.
            if stack.contains(&id) {
                continue;
            }
            stack.push(id);

            let Some(scope) = self.get_scope(id) else {
                continue;
            };

            for (name, binding) in scope.bindings() {
                if seen.insert(name) {
                    out.push((name, *binding, scope.kind));
                }
            }

            for parent in scope.parents.iter().copied() {
                to_visit.push(parent);
            }
        }

        out
    }
}
