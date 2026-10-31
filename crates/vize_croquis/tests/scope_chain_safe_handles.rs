//! Safe handle mutation must never reach an unchecked scope read.

use std::panic::{AssertUnwindSafe, catch_unwind};
use vize_carton::cstr;
use vize_croquis::{ScopeBinding, ScopeChain, ScopeId, ScopeKind};
use vize_relief::BindingType;

fn add(chain: &mut ScopeChain, name: &str, offset: u32) {
    chain.add_binding(
        name.into(),
        ScopeBinding::new(BindingType::SetupConst, offset),
    );
}

#[test]
fn out_of_range_current_handles_panic_before_changing_the_whole_chain() {
    let mut chain = ScopeChain::new();
    let current = chain.enter_scope(ScopeKind::Block);
    add(&mut chain, "local", 17);
    for id in [ScopeId::new(chain.len() as u32), ScopeId::new(u32::MAX)] {
        let before = cstr!("{chain:?}");
        assert!(catch_unwind(AssertUnwindSafe(|| chain.set_current(id))).is_err());
        assert_eq!(cstr!("{chain:?}"), before);
        assert_eq!(chain.current_id(), current);
        assert_eq!(chain.lookup("local").unwrap().1.declaration_offset, 17);
    }
}

#[test]
fn exiting_an_out_of_range_public_parent_preserves_the_current_scope() {
    let mut chain = ScopeChain::new();
    let current = chain.enter_scope(ScopeKind::Block);
    chain.current_scope_mut().parents.clear();
    chain
        .current_scope_mut()
        .parents
        .push(ScopeId::new(u32::MAX));
    let before = cstr!("{chain:?}");
    assert!(catch_unwind(AssertUnwindSafe(|| chain.exit_scope())).is_err());
    assert_eq!(chain.current_id(), current);
    assert_eq!(cstr!("{chain:?}"), before);
}

#[test]
fn lookup_skips_absent_parents_and_preserves_the_existing_parent_order() {
    let mut chain = ScopeChain::new();
    add(&mut chain, "root_only", 1);
    let first = chain.enter_scope(ScopeKind::Block);
    add(&mut chain, "shared", 11);
    chain.set_current(ScopeId::ROOT);
    let last = chain.enter_scope(ScopeKind::Block);
    add(&mut chain, "shared", 22);
    let child = chain.enter_scope(ScopeKind::EventHandler);
    let absent = ScopeId::new(chain.len() as u32);
    chain.current_scope_mut().parents.clear();
    chain
        .current_scope_mut()
        .parents
        .extend([first, absent, last, ScopeId::new(u32::MAX)]);
    let before = cstr!("{chain:?}");
    let (scope, binding) = chain.lookup("shared").unwrap();
    assert!(core::ptr::eq(scope, chain.get_scope(last).unwrap()));
    assert_eq!(binding.declaration_offset, 22);
    assert_eq!(chain.lookup("root_only").unwrap().1.declaration_offset, 1);
    assert!(chain.lookup("missing").is_none());
    assert_eq!(chain.current_id(), child);
    assert_eq!(cstr!("{chain:?}"), before);
}

#[test]
fn cyclic_and_out_of_range_parents_keep_lookup_and_mutation_finite() {
    let mut chain = ScopeChain::new();
    let child = chain.enter_scope(ScopeKind::Block);
    add(&mut chain, "local", 31);
    chain.get_scope_mut(ScopeId::ROOT).unwrap().parents.extend([
        child,
        ScopeId::new(u32::MAX),
        ScopeId::ROOT,
    ]);
    assert!(chain.lookup("missing").is_none());
    chain.mark_used("local");
    chain.mark_mutated("local");
    let (scope, binding) = chain.lookup("local").unwrap();
    assert!(core::ptr::eq(scope, chain.get_scope(child).unwrap()));
    assert_eq!(binding.declaration_offset, 31);
    assert!(binding.is_used());
    assert!(binding.is_mutated());
    chain.mark_used("missing");
    chain.mark_mutated("missing");
}

#[test]
fn in_range_handles_still_address_physical_slots_after_public_id_edits() {
    let mut chain = ScopeChain::new();
    let child = chain.enter_scope(ScopeKind::Block);
    add(&mut chain, "local", 41);
    chain.current_scope_mut().id = ScopeId::new(u32::MAX);
    chain.set_current(ScopeId::ROOT);
    assert!(chain.lookup("local").is_none());
    // ScopeId carries an index, not a chain provenance token.
    chain.set_current(ScopeId::new(child.as_u32()));
    let (scope, binding) = chain.lookup("local").unwrap();
    assert!(core::ptr::eq(scope, chain.get_scope(child).unwrap()));
    assert_eq!(scope.id, ScopeId::new(u32::MAX));
    assert_eq!(binding.declaration_offset, 41);
    chain.exit_scope();
    assert_eq!(chain.current_id(), ScopeId::ROOT);
}

#[test]
fn replacing_public_scope_values_cannot_remove_the_current_physical_slot() {
    let mut chain = ScopeChain::new();
    let physical = chain.enter_scope(ScopeKind::Block);
    let length = chain.len();
    let mut donor = ScopeChain::new();
    donor.enter_scope(ScopeKind::Block);
    donor.current_scope_mut().id = ScopeId::new(u32::MAX);
    donor.current_scope_mut().parents.clear();
    donor
        .current_scope_mut()
        .parents
        .push(ScopeId::new(u32::MAX));
    // The public API exposes borrowed Scope values, not its private module.
    core::mem::swap(chain.current_scope_mut(), donor.current_scope_mut());
    assert_eq!(chain.len(), length);
    assert_eq!(chain.current_id(), physical);
    assert_eq!(chain.current_scope().id, ScopeId::new(u32::MAX));
    assert!(chain.lookup("missing").is_none());
    let before = cstr!("{chain:?}");
    assert!(catch_unwind(AssertUnwindSafe(|| chain.exit_scope())).is_err());
    assert_eq!(cstr!("{chain:?}"), before);
    chain.get_scope_mut(physical).unwrap().parents.clear();
    chain
        .get_scope_mut(physical)
        .unwrap()
        .parents
        .push(ScopeId::ROOT);
    chain.exit_scope();
    assert_eq!(chain.current_id(), ScopeId::ROOT);
    assert_eq!(chain.len(), length);
    assert_eq!(chain.iter().count(), length);
}

#[test]
fn depth_preserves_valid_chains_and_stops_at_public_parent_cycles() {
    let mut chain = ScopeChain::new();
    let first = chain.enter_scope(ScopeKind::Block);
    let second = chain.enter_scope(ScopeKind::Block);
    assert_eq!(chain.depth(ScopeId::ROOT), 0);
    assert_eq!(chain.depth(first), 1);
    assert_eq!(chain.depth(second), 2);
    assert_eq!(chain.depth(ScopeId::new(u32::MAX)), 0);
    chain
        .get_scope_mut(ScopeId::ROOT)
        .unwrap()
        .parents
        .push(second);
    let before = cstr!("{chain:?}");
    for id in [ScopeId::ROOT, first, second] {
        assert_eq!(chain.depth(id), 2);
    }
    assert_eq!(cstr!("{chain:?}"), before);
    chain.get_scope_mut(first).unwrap().parents.clear();
    chain.get_scope_mut(first).unwrap().parents.push(first);
    assert_eq!(chain.depth(first), 0);
    assert_eq!(chain.depth(second), 1);
    chain.get_scope_mut(first).unwrap().parents.clear();
    chain
        .get_scope_mut(first)
        .unwrap()
        .parents
        .push(ScopeId::new(u32::MAX));
    // Preserve the existing one-hop count before an absent parent terminates.
    assert_eq!(chain.depth(first), 1);
    assert_eq!(chain.depth(second), 2);
}
