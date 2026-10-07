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
mod tests;
