//! One structural slot contract per static component.
//!
//! `$StructuralSlots<typeof Component>` does not read that use's slots. A
//! `v-else-if` chain closes the previous branch before the next one, so the
//! declaration is written in the template function and each use only aliases
//! it. The per-use required-slots const stays put.

use std::cell::RefCell;

use vize_carton::{FxHashMap, String, cstr};
use vize_croquis::analysis::ComponentUsage;
use vize_relief::RootNode;

struct State {
    /// Static component key → template-function declaration name.
    hoisted: FxHashMap<String, String>,
    pending: Vec<String>,
}

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State {
        hoisted: FxHashMap::default(),
        pending: Vec::new(),
    });
}

struct Share<'a> {
    component_type_name: &'a str,
    component_ref: &'a str,
    factory: &'a str,
    dynamic: bool,
    block: Option<u64>,
}

pub(crate) fn reset() {
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        state.hoisted.clear();
        state.pending.clear();
    });
}

pub(crate) fn flush(ts: &mut String) {
    STATE.with(|state| {
        let pending = std::mem::take(&mut state.borrow_mut().pending);
        for line in &pending {
            ts.push_str(line.as_str());
        }
    });
}

/// Right-hand side of `type $SlotContract_{idx}`.
///
/// Guarded static components alias one template-function declaration. A
/// dynamic `:is` binding, or a use outside a guard, keeps the full type where
/// it is written so the declaration stays in a scope that can see `typeof`.
pub(super) fn rhs(
    usage: &ComponentUsage,
    component_ref: &str,
    template_ast: Option<&RootNode<'_>>,
    hoist_shared_preamble: bool,
) -> String {
    let factory =
        crate::virtual_ts::helpers::shared_types::STRUCTURAL_SLOTS.select(hoist_shared_preamble);
    let type_name = crate::virtual_ts::helpers::to_safe_identifier_fragment(usage.name.as_str());
    let dynamic = super::super::is_owned_dynamic_component_alias(template_ast, usage)
        || super::super::is_owned_named_dynamic_component(template_ast, usage);
    let share = Share {
        component_type_name: type_name.as_str(),
        component_ref,
        factory,
        dynamic,
        block: super::super::open_chained_vif_guard_block(),
    };
    alias_target(&share).unwrap_or_else(|| cstr!("{factory}<typeof {component_ref}>"))
}

fn alias_target(share: &Share<'_>) -> Option<String> {
    if share.dynamic || share.block.is_none() {
        return None;
    }
    let key = component_key(share);
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        if let Some(name) = state.hoisted.get(&key) {
            return Some(name.clone());
        }
        let name = fresh_name(share.component_type_name, &state.hoisted);
        state.pending.push(declaration(
            name.as_str(),
            share.component_ref,
            share.factory,
        ));
        state.hoisted.insert(key, name.clone());
        Some(name)
    })
}

fn component_key(share: &Share<'_>) -> String {
    cstr!(
        "{}\0{}\0{}",
        share.component_type_name,
        share.component_ref,
        share.factory
    )
}

fn fresh_name(type_name: &str, hoisted: &FxHashMap<String, String>) -> String {
    let base = cstr!("__{type_name}_slot_contract");
    if hoisted.values().all(|name| name != &base) {
        return base;
    }
    let mut nth = 2u32;
    loop {
        let name = cstr!("{base}_{nth}");
        if hoisted.values().all(|existing| existing != &name) {
            return name;
        }
        nth += 1;
    }
}

fn declaration(name: &str, component_ref: &str, factory: &str) -> String {
    cstr!("  type {name} = {factory}<typeof {component_ref}>;\n")
}
