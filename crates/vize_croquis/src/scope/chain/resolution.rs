//! Binding resolution, mutation tracking, and scope depth computation.
//!
//! Provides lookup, mark-used, mark-mutated, and depth calculation methods
//! for the [`ScopeChain`].

use super::{CompactString, Scope, ScopeBinding, ScopeChain, ScopeId, SmallVec, smallvec};

const INLINE_VISITED_WORDS: usize = 4;

/// Compact cycle detection for scope walks.
///
/// Lexical scopes normally form a contiguous descending ID range, represented
/// by two integers. Non-linear parents use one bit per handle, avoiding the
/// quadratic scans of a `SmallVec<ScopeId>`. Four inline words cover sparse IDs
/// below 256 on 64-bit targets. Invalid public handles stay separate so a forged
/// large ID cannot force a correspondingly large allocation.
#[derive(Default)]
struct ScopeVisitSet {
    dense_range: Option<(u32, u32)>,
    words: SmallVec<[usize; INLINE_VISITED_WORDS]>,
    invalid: SmallVec<[ScopeId; 1]>,
}

impl ScopeVisitSet {
    #[inline]
    fn contains(&self, id: ScopeId, scope_count: usize) -> bool {
        let index = id.as_u32() as usize;
        if index >= scope_count {
            return self.invalid.contains(&id);
        }
        if self
            .dense_range
            .is_some_and(|(min, max)| id.as_u32() >= min && id.as_u32() <= max)
        {
            return true;
        }

        let bits_per_word = usize::BITS as usize;
        let word_index = index / bits_per_word;
        let mask = 1usize << (index % bits_per_word);
        self.words
            .get(word_index)
            .is_some_and(|word| word & mask != 0)
    }

    /// Record a scope handle, returning whether this is its first visit.
    #[inline]
    fn insert(&mut self, id: ScopeId, scope_count: usize) -> bool {
        let index = id.as_u32() as usize;
        if index >= scope_count {
            if self.invalid.contains(&id) {
                return false;
            }
            self.invalid.push(id);
            return true;
        }
        if self.contains(id, scope_count) {
            return false;
        }

        let raw = id.as_u32();
        match &mut self.dense_range {
            None => {
                self.dense_range = Some((raw, raw));
                return true;
            }
            Some((min, _)) if raw.checked_add(1) == Some(*min) => {
                *min = raw;
                return true;
            }
            Some((_, max)) if max.checked_add(1) == Some(raw) => {
                *max = raw;
                return true;
            }
            Some(_) => {}
        }

        let bits_per_word = usize::BITS as usize;
        let word_index = index / bits_per_word;
        if self.words.len() <= word_index {
            self.words.resize(word_index + 1, 0);
        }
        let mask = 1usize << (index % bits_per_word);
        let Some(word) = self.words.get_mut(word_index) else {
            return false;
        };
        let first_visit = *word & mask == 0;
        *word |= mask;
        first_visit
    }
}

impl ScopeChain {
    /// Look up a binding by name, searching through all parent scopes
    /// Searches lexical parents and additional parents like Vue globals.
    #[inline]
    pub fn lookup(&self, name: &str) -> Option<(&Scope, &ScopeBinding)> {
        self.lookup_location(name)
            .and_then(|(id, binding)| self.scopes.get(id).map(|scope| (scope, binding)))
    }

    /// Carry the physical slot chosen by the original immutable lookup.
    fn lookup_location(&self, name: &str) -> Option<(ScopeId, &ScopeBinding)> {
        let scope_count = self.scopes.len();
        let mut visited = ScopeVisitSet::default();
        let mut queue: SmallVec<[ScopeId; 8]> = smallvec![self.current];

        while let Some(id) = queue.pop() {
            if !visited.insert(id, scope_count) {
                continue;
            }

            // Safe callers can construct ids and edit public Scope::parents.
            // Ignore absent parents, as the mutable and visibility walks do.
            let Some(scope) = self.scopes.get(id) else {
                continue;
            };
            if let Some(binding) = scope.get_binding(name) {
                return Some((id, binding));
            }

            // Add all parents to queue
            for &parent_id in &scope.parents {
                if !visited.contains(parent_id, scope_count) {
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
        let Some((id, _)) = self.lookup_location(name) else {
            return;
        };
        if let Some(binding) = self
            .scopes
            .get_mut(id)
            .and_then(|scope| scope.get_binding_mut(name))
        {
            binding.mark_used();
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
        let Some((id, _)) = self.lookup_location(name) else {
            return;
        };
        if let Some(binding) = self
            .scopes
            .get_mut(id)
            .and_then(|scope| scope.get_binding_mut(name))
        {
            binding.mark_mutated();
        }
    }

    /// Calculate the depth of a scope (distance from root via primary parent chain)
    /// Stop before a repeated handle when public parents form a cycle.
    #[inline]
    pub fn depth(&self, id: ScopeId) -> u32 {
        let scope_count = self.scopes.len();
        let mut depth = 0u32;
        let mut visited = ScopeVisitSet::default();
        visited.insert(id, scope_count);
        let mut current_id = self.get_scope(id).and_then(|s| s.parent());
        while let Some(pid) = current_id {
            if !visited.insert(pid, scope_count) {
                break;
            }
            depth += 1;
            current_id = self.get_scope(pid).and_then(|s| s.parent());
        }
        depth
    }
}

#[cfg(test)]
mod tests {
    //! Whole-state controls against the original two-traversal template contract.

    use super::{
        CompactString, ScopeBinding, ScopeChain, ScopeId, ScopeVisitSet, SmallVec, smallvec,
    };
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
        assert_eq!(left.v_for_source_spans, right.v_for_source_spans);
    }

    #[test]
    fn visit_set_keeps_deep_lexical_ranges_out_of_the_sparse_bitmap() {
        let scope_count = 4095;
        let mut visited = ScopeVisitSet::default();
        for raw in (0..scope_count as u32).rev() {
            assert!(visited.insert(ScopeId::new(raw), scope_count));
        }

        assert_eq!(visited.dense_range, Some((0, 4094)));
        assert!(visited.words.is_empty());
        assert!(!visited.insert(ScopeId::new(2048), scope_count));
        assert!(visited.insert(ScopeId::new(u32::MAX), scope_count));
        assert!(!visited.insert(ScopeId::new(u32::MAX), scope_count));
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

    #[test]
    fn deep_resolution_preserves_sparse_parents_and_stops_at_cycles() {
        let mut chain = ScopeChain::new();
        chain.add_binding(
            CompactString::new("target"),
            ScopeBinding::new(BindingType::SetupLet, 1),
        );
        for _ in 0..300 {
            chain.enter_scope(ScopeKind::Block);
        }

        let deepest = chain.current;
        assert_eq!(chain.depth(deepest), 300);
        assert_eq!(chain.lookup("target").unwrap().1.declaration_offset, 1);

        chain.mark_mutated("target");
        assert!(
            chain
                .get_scope(ScopeId::ROOT)
                .unwrap()
                .get_binding("target")
                .unwrap()
                .is_mutated()
        );

        chain
            .get_scope_mut(ScopeId::ROOT)
            .unwrap()
            .parents
            .push(deepest);
        chain
            .get_scope_mut(ScopeId::ROOT)
            .unwrap()
            .parents
            .push(ScopeId::new(u32::MAX));
        assert!(chain.lookup("missing").is_none());
        chain.mark_used("missing");
        chain.mark_mutated("missing");
    }
}
