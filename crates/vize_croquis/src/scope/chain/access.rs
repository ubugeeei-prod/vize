//! Scope access keeps typed checked reads and the existing hot-path contract.

use super::{Scope, ScopeChain, ScopeId};

impl ScopeChain {
    /// Pre-allocate slots without rebuilding the existing universal root.
    #[inline]
    pub(crate) fn reserve_scopes(&mut self, capacity: usize) {
        let additional = capacity.saturating_sub(self.scopes.len());
        self.scopes.reserve_exact(additional);
    }

    /// Get the current scope
    #[inline]
    pub fn current_scope(&self) -> &Scope {
        // The scope table proves that its current row exists (`rows.rs`).
        // This read stays unchecked on every identifier lookup path.
        self.scopes.current()
    }

    /// Get the current scope mutably
    #[inline]
    pub fn current_scope_mut(&mut self) -> &mut Scope {
        self.scopes.current_mut()
    }

    /// Get a scope by ID
    #[inline]
    pub fn get_scope(&self, id: ScopeId) -> Option<&Scope> {
        self.scopes.get(id.as_u32())
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
