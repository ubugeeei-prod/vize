//! Binding resolution, mutation tracking, and scope depth computation.
//!
//! Provides lookup, mark-used, mark-mutated, and depth calculation methods
//! for the [`ScopeChain`].

use super::{smallvec, CompactString, Scope, ScopeBinding, ScopeChain, ScopeId, SmallVec};

impl ScopeChain {
    /// Look up a binding by name, searching through all parent scopes
    /// Uses BFS to search all accessible scopes (lexical parents + additional parents like Vue globals)
    #[inline]
    pub fn lookup(&self, name: &str) -> Option<(&Scope, &ScopeBinding)> {
        let mut visited: SmallVec<[ScopeId; 8]> = SmallVec::new();
        let mut queue: SmallVec<[ScopeId; 8]> = smallvec![self.current];

        while let Some(id) = queue.pop() {
            if visited.contains(&id) {
                continue;
            }
            visited.push(id);

            let scope = unsafe { self.scopes.get_unchecked(id.as_u32() as usize) };
            if let Some(binding) = scope.get_binding(name) {
                return Some((scope, binding));
            }

            // Add all parents to queue
            for &parent_id in &scope.parents {
                if !visited.contains(&parent_id) {
                    queue.push(parent_id);
                }
            }
        }

        None
    }

    /// Check if a name is defined in any accessible scope
    #[inline]
    pub fn is_defined(&self, name: &str) -> bool {
        self.lookup(name).is_some()
    }

    /// Add a binding to the current scope
    #[inline]
    pub fn add_binding(&mut self, name: CompactString, binding: ScopeBinding) {
        self.current_scope_mut().add_binding(name, binding);
    }

    /// Mark a binding as used (searches through all parent scopes)
    pub fn mark_used(&mut self, name: &str) {
        let mut visited: SmallVec<[ScopeId; 8]> = SmallVec::new();
        let mut queue: SmallVec<[ScopeId; 8]> = smallvec![self.current];

        while let Some(id) = queue.pop() {
            if visited.contains(&id) {
                continue;
            }
            visited.push(id);

            let scope = &mut self.scopes[id.as_u32() as usize];
            if let Some(binding) = scope.get_binding_mut(name) {
                binding.mark_used();
                return;
            }

            // Collect parents before continuing (to avoid borrow issues)
            let parents: SmallVec<[ScopeId; 2]> = scope.parents.clone();
            for parent_id in parents {
                if !visited.contains(&parent_id) {
                    queue.push(parent_id);
                }
            }
        }
    }

    /// Check if a binding has been marked as used (searches through all scopes)
    pub fn is_used(&self, name: &str) -> bool {
        for scope in &self.scopes {
            if let Some(binding) = scope.get_binding(name) {
                return binding.is_used();
            }
        }
        false
    }

    /// Mark a binding as mutated (searches through all parent scopes)
    pub fn mark_mutated(&mut self, name: &str) {
        let mut visited: SmallVec<[ScopeId; 8]> = SmallVec::new();
        let mut queue: SmallVec<[ScopeId; 8]> = smallvec![self.current];

        while let Some(id) = queue.pop() {
            if visited.contains(&id) {
                continue;
            }
            visited.push(id);

            let scope = &mut self.scopes[id.as_u32() as usize];
            if let Some(binding) = scope.get_binding_mut(name) {
                binding.mark_mutated();
                return;
            }

            // Collect parents before continuing (to avoid borrow issues)
            let parents: SmallVec<[ScopeId; 2]> = scope.parents.clone();
            for parent_id in parents {
                if !visited.contains(&parent_id) {
                    queue.push(parent_id);
                }
            }
        }
    }

    /// Calculate the depth of a scope (distance from root via primary parent chain)
    #[inline]
    pub fn depth(&self, id: ScopeId) -> u32 {
        let mut depth = 0u32;
        let mut current_id = self.get_scope(id).and_then(|s| s.parent());
        while let Some(pid) = current_id {
            depth += 1;
            current_id = self.get_scope(pid).and_then(|s| s.parent());
        }
        depth
    }
}
