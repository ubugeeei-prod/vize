//! Scope access keeps typed checked reads and the existing hot-path contract.

use super::{Scope, ScopeChain, ScopeId};

impl ScopeChain {
    /// Pre-allocate slots without rebuilding the existing universal root.
    #[inline]
    pub(crate) fn reserve_scopes(&mut self, capacity: usize) {
        let additional = capacity.saturating_sub(self.scopes.len());
        self.scopes.raw.reserve_exact(additional);
    }

    /// Get the current scope
    #[inline]
    pub fn current_scope(&self) -> &Scope {
        // SAFETY: `current` is initialized to `ROOT`, and every scope transition
        // writes an id returned by `push_scope`/`enter_scope`, both of which append
        // to `self.scopes` before exposing the id. Exiting a scope moves to a
        // stored parent id, never an arbitrary index. This unchecked access is on
        // every identifier lookup path, so we keep the invariant centralized here.
        unsafe {
            self.scopes
                .raw
                .get_unchecked(self.current.as_u32() as usize)
        }
    }

    /// Get the current scope mutably
    #[inline]
    pub fn current_scope_mut(&mut self) -> &mut Scope {
        let idx = self.current.as_u32() as usize;
        // SAFETY: same invariant as `current_scope`: `idx` comes from a
        // ScopeId minted by this chain and therefore addresses an existing scope.
        // The `&mut self` receiver guarantees no competing borrow of the scope
        // vector while returning this mutable reference.
        unsafe { self.scopes.raw.get_unchecked_mut(idx) }
    }

    /// Get a scope by ID
    #[inline]
    pub fn get_scope(&self, id: ScopeId) -> Option<&Scope> {
        self.scopes.get(id)
    }

    /// Get a scope by ID (unchecked)
    ///
    /// # Safety
    /// Caller must ensure `id` was produced by this `ScopeChain` and the chain has
    /// not been rebuilt since. The method exists for analyzer hot paths where the
    /// caller already proved the id through registry traversal.
    #[inline]
    pub unsafe fn get_scope_unchecked(&self, id: ScopeId) -> &Scope {
        // SAFETY: upheld by the caller contract above.
        unsafe { self.scopes.raw.get_unchecked(id.as_u32() as usize) }
    }
}

#[cfg(test)]
mod tests {
    //! Complete original root and transitions survive storage reservation.

    use super::super::{BindingType, CompactString, ScopeBinding, ScopeChain, ScopeKind};

    #[test]
    fn reserved_existing_root_keeps_original_whole_scope_graph() {
        for capacity in [0, 1, 16, 64] {
            let mut original = ScopeChain::with_capacity(capacity);
            let mut reserved = ScopeChain::default();
            reserved.reserve_scopes(capacity);
            assert_eq!(format!("{reserved:?}"), format!("{original:?}"));

            for chain in [&mut original, &mut reserved] {
                chain.enter_scope(ScopeKind::Block);
                chain.add_binding(
                    CompactString::new("Math"),
                    ScopeBinding::new(BindingType::SetupRef, 17),
                );
                chain.enter_scope(ScopeKind::Function);
                chain.add_binding(
                    CompactString::new("local"),
                    ScopeBinding::new(BindingType::SetupLet, 29),
                );
                chain.mark_used("Math");
                chain.mark_used("Array");
                chain.mark_mutated("local");
            }
            assert_eq!(format!("{reserved:?}"), format!("{original:?}"));
            assert_eq!(reserved.current_id(), original.current_id());
            reserved.reserve_scopes(128);
            assert_eq!(format!("{reserved:?}"), format!("{original:?}"));
        }
    }
}
