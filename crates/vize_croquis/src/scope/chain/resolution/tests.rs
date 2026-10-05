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
            "missing", "local", "shared", "lexical", "global", "root", "Math", "$event", "shared",
            "missing",
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
