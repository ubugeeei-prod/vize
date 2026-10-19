//! Binding resolution, mutation tracking, and scope depth computation.
//!
//! Provides lookup, mark-used, mark-mutated, and depth calculation methods
//! for the [`ScopeChain`].

use super::{CompactString, Scope, ScopeBinding, ScopeChain, ScopeId, SmallVec, smallvec};

impl ScopeChain {
    /// Look up a binding by name, searching through all parent scopes
    /// Uses BFS to search all accessible scopes (lexical parents + additional parents like Vue globals)
    #[inline]
    pub fn lookup(&self, name: &str) -> Option<(&Scope, &ScopeBinding)> {
        self.lookup_location(name)
            .and_then(|(id, binding)| self.scopes.get(id).map(|scope| (scope, binding)))
    }

    /// Carry the physical slot chosen by the original immutable lookup.
    fn lookup_location(&self, name: &str) -> Option<(ScopeId, &ScopeBinding)> {
        let mut visited: SmallVec<[ScopeId; 8]> = SmallVec::new();
        let mut queue: SmallVec<[ScopeId; 8]> = smallvec![self.current];

        while let Some(id) = queue.pop() {
            if visited.contains(&id) {
                continue;
            }
            visited.push(id);

            // SAFETY: the queue is seeded with `self.current` and only extended
            // with parent ids stored inside scopes owned by this chain. Scope ids
            // are never synthesized from user input, so every queued id indexes
            // `self.scopes`. This keeps lexical lookup branch-light in the
            // compiler's busiest semantic-analysis loop.
            let scope = unsafe { self.scopes.raw.get_unchecked(id.as_u32() as usize) };
            if let Some(binding) = scope.get_binding(name) {
                return Some((id, binding));
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

            let Some(scope) = self.scopes.get_mut(id) else {
                continue;
            };
            if let Some(binding) = scope.get_binding_mut(name) {
                binding.mark_used();
                return;
            }

            for parent_id in &scope.parents {
                if !visited.contains(parent_id) {
                    queue.push(*parent_id);
                }
            }
        }
    }

    /// Resolve once, preserving cheap immutable misses and the physical slot.
    pub(crate) fn mark_used_if_defined(&mut self, name: &str) -> bool {
        let Some((id, _)) = self.lookup_location(name) else {
            return false;
        };
        if let Some(binding) = self
            .scopes
            .get_mut(id)
            .and_then(|scope| scope.get_binding_mut(name))
        {
            binding.mark_used();
        }
        true
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

            let Some(scope) = self.scopes.get_mut(id) else {
                continue;
            };
            if let Some(binding) = scope.get_binding_mut(name) {
                binding.mark_mutated();
                return;
            }

            for parent_id in &scope.parents {
                if !visited.contains(parent_id) {
                    queue.push(*parent_id);
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

#[cfg(test)]
mod tests {
    //! Whole-state controls against the original two-traversal template contract.

    use super::{CompactString, ScopeBinding, ScopeChain, ScopeId, SmallVec, smallvec};
    use crate::scope::types::ScopeKind;
    use vize_relief::BindingType;

    fn graph(cycle: bool) -> ScopeChain {
        let mut chain = ScopeChain::new();
        for (name, offset) in [("root", 1), ("shared", 2), ("Math", 3)] {
            chain.add_binding(
                CompactString::new(name),
                ScopeBinding::new(BindingType::SetupConst, offset),
            );
        }
        let lexical = chain.enter_scope(ScopeKind::Block);
        for (name, offset) in [("lexical", 11), ("shared", 12)] {
            chain.add_binding(
                CompactString::new(name),
                ScopeBinding::new(BindingType::SetupLet, offset),
            );
        }
        let global = chain.enter_scope(ScopeKind::VueGlobal);
        chain.current_scope_mut().parents = smallvec![ScopeId::ROOT];
        for (name, offset) in [("global", 21), ("shared", 22)] {
            chain.add_binding(
                CompactString::new(name),
                ScopeBinding::new(BindingType::SetupConst, offset),
            );
        }
        let child = chain.enter_scope(ScopeKind::EventHandler);
        chain.current_scope_mut().parents = smallvec![lexical, global, lexical];
        for (name, offset) in [("local", 31), ("$event", 32), ("shared", 33)] {
            chain.add_binding(
                CompactString::new(name),
                ScopeBinding::new(BindingType::SetupConst, offset),
            );
        }
        if cycle {
            chain
                .get_scope_mut(ScopeId::ROOT)
                .unwrap()
                .parents
                .push(child);
        }
        chain
    }

    // The original mark_used body is retained independently from the new helper.
    fn original_mark_used(chain: &mut ScopeChain, name: &str) {
        let mut visited: SmallVec<[ScopeId; 8]> = SmallVec::new();
        let mut queue: SmallVec<[ScopeId; 8]> = smallvec![chain.current];
        while let Some(id) = queue.pop() {
            if visited.contains(&id) {
                continue;
            }
            visited.push(id);
            let Some(scope) = chain.scopes.get_mut(id) else {
                continue;
            };
            if let Some(binding) = scope.get_binding_mut(name) {
                binding.mark_used();
                return;
            }
            for parent_id in &scope.parents {
                if !visited.contains(parent_id) {
                    queue.push(*parent_id);
                }
            }
        }
    }

    fn assert_whole_chain(left: &ScopeChain, right: &ScopeChain) {
        assert_eq!(format!("{left:?}"), format!("{right:?}"));
        assert_eq!(left.dynamic_v_slot_scopes, right.dynamic_v_slot_scopes);
        assert_eq!(
            left.directive_expression_offsets,
            right.directive_expression_offsets
        );
    }

    #[test]
    fn fused_resolution_preserves_whole_original_usage_state() {
        for cycle in [false, true] {
            let mut original = graph(cycle);
            let mut fused = graph(cycle);
            for name in [
                "missing", "local", "shared", "lexical", "global", "root", "Math", "$event",
                "shared", "missing",
            ] {
                let found = original.is_defined(name);
                if found {
                    original_mark_used(&mut original, name);
                }
                assert_eq!(fused.mark_used_if_defined(name), found);
                assert_whole_chain(&original, &fused);
            }
        }
    }

    #[test]
    fn fused_resolution_public_mark_used_preserves_missing_and_shadowed_bindings() {
        let mut original = graph(true);
        let mut public = graph(true);
        for name in [
            "missing", "shared", "$event", "Math", "lexical", "global", "root", "shared",
        ] {
            original_mark_used(&mut original, name);
            public.mark_used(name);
            assert_whole_chain(&original, &public);
        }
        let (scope, binding) = public.lookup("shared").unwrap();
        assert_eq!(scope.kind, ScopeKind::EventHandler);
        assert_eq!(binding.declaration_offset, 33);
        assert!(binding.is_used());
        for scope in &public.scopes {
            if let Some(binding) = scope.get_binding("shared") {
                assert_eq!(binding.is_used(), scope.id == public.current);
            }
        }
    }

    #[test]
    fn fused_resolution_uses_physical_slot_when_public_scope_id_changes() {
        let mut original = graph(true);
        let mut fused = graph(true);
        let physical = original.current;
        original.current_scope_mut().id = ScopeId::ROOT;
        fused.current_scope_mut().id = ScopeId::ROOT;
        for name in ["shared", "local", "Math", "$event", "missing", "shared"] {
            let found = original.is_defined(name);
            if found {
                original_mark_used(&mut original, name);
            }
            assert_eq!(fused.mark_used_if_defined(name), found);
            assert_whole_chain(&original, &fused);
        }
        let (scope, binding) = fused.lookup("shared").unwrap();
        assert!(core::ptr::eq(scope, fused.get_scope(physical).unwrap()));
        assert_eq!(binding.declaration_offset, 33);
        assert!(core::ptr::eq(binding, scope.get_binding("shared").unwrap()));
        assert!(
            fused
                .get_scope(physical)
                .unwrap()
                .get_binding("shared")
                .unwrap()
                .is_used()
        );
        assert!(
            !fused
                .get_scope(ScopeId::ROOT)
                .unwrap()
                .get_binding("shared")
                .unwrap()
                .is_used()
        );
    }
}
