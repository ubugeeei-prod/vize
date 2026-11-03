//! Shared standard global scopes against the per-file insert loop they replace.

use std::borrow::Cow;

use super::{BindingType, CompactString, Scope, ScopeBinding, ScopeChain, ScopeId, ScopeKind};
use crate::scope::globals::{
    self, BROWSER_GLOBALS, JS_UNIVERSAL_GLOBALS, NODE_GLOBALS, VUE_GLOBALS,
};
use crate::scope::{JsGlobalScopeData, JsRuntime, ParamNames, VueGlobalScopeData};

fn names(list: &[&str]) -> ParamNames {
    list.iter().map(|&name| CompactString::new(name)).collect()
}

/// The seeding as it ran before the shared tables: an owned root filled by
/// one insert per name, then the generic insert loop for each global scope.
fn per_file(source_len: u32) -> ScopeChain {
    let mut root = Scope::new(ScopeId::ROOT, None, ScopeKind::JsGlobalUniversal);
    for name in JS_UNIVERSAL_GLOBALS {
        root.add_binding(
            CompactString::new(name),
            ScopeBinding::new(BindingType::JsGlobalUniversal, 0),
        );
    }
    let mut chain = ScopeChain::with_root(index_vec::IndexVec::from_vec(vec![root]));
    for (runtime, list) in [
        (JsRuntime::Browser, BROWSER_GLOBALS),
        (JsRuntime::Node, NODE_GLOBALS),
    ] {
        let data = JsGlobalScopeData {
            runtime,
            globals: names(list),
        };
        chain.enter_js_global_scope(data, 0, 0);
        chain.exit_scope();
    }
    let data = VueGlobalScopeData {
        globals: names(VUE_GLOBALS),
    };
    chain.enter_vue_global_scope(data, 0, 0);
    chain.exit_scope();
    chain.enter_module_scope(0, source_len);
    chain
}

fn shared(source_len: u32) -> ScopeChain {
    let mut chain = ScopeChain::new();
    crate::scope::setup_global_scopes(&mut chain, source_len);
    chain
}

/// Every scope's bindings in iteration order, with flags and offsets.
fn walk(chain: &ScopeChain) -> Vec<String> {
    chain
        .iter()
        .map(|scope| {
            let ordered: Vec<_> = scope.bindings().collect();
            format!(
                "{:?} {:?} {:?} {:?} {}",
                scope.id,
                scope.kind,
                scope.data(),
                ordered,
                scope.binding_count()
            )
        })
        .collect()
}

fn borrowed(chain: &ScopeChain) -> Vec<bool> {
    chain
        .iter()
        .map(|scope| matches!(scope.bindings, Cow::Borrowed(_)))
        .collect()
}

#[test]
fn shared_globals_answer_like_the_per_file_insert_loop() {
    let original = per_file(120);
    let seeded = shared(120);
    assert_eq!(format!("{seeded:?}"), format!("{original:?}"));
    assert_eq!(walk(&seeded), walk(&original));
    assert_eq!(seeded.current_id(), original.current_id());
    assert_eq!(borrowed(&seeded), vec![true, true, true, true, false]);
    assert_eq!(
        seeded.get_scope(ScopeId::ROOT).map(|root| root.kind),
        Some(ScopeKind::JsGlobalUniversal)
    );
    assert_eq!(
        globals::UNIVERSAL.bindings.len(),
        JS_UNIVERSAL_GLOBALS.len()
    );
}

#[test]
fn seeded_global_lookups_keep_independently_named_bindings() {
    let mut seeded = shared(120);
    for (kind, binding_type, names) in [
        (
            ScopeKind::JsGlobalUniversal,
            BindingType::JsGlobalUniversal,
            ["Math", "JSON"],
        ),
        (
            ScopeKind::JsGlobalBrowser,
            BindingType::JsGlobalBrowser,
            ["window", "document"],
        ),
        (
            ScopeKind::JsGlobalNode,
            BindingType::JsGlobalNode,
            ["process", "Buffer"],
        ),
        (
            ScopeKind::VueGlobal,
            BindingType::VueGlobal,
            ["$emit", "$slots"],
        ),
    ] {
        let scope = seeded.find_scope_by_kind(kind).expect("standard scope");
        seeded.set_current(scope);
        for name in names {
            assert_eq!(
                seeded.lookup(name).map(|(owner, binding)| (
                    owner.kind,
                    binding.binding_type,
                    binding.declaration_offset,
                )),
                Some((kind, binding_type, 0)),
                "{name}",
            );
        }
        assert_eq!(seeded.lookup("notAStandardGlobal").map(|_| ()), None);
    }
}

#[test]
fn a_miss_keeps_the_table_shared_and_a_write_takes_an_identical_copy() {
    let mut original = per_file(64);
    let mut seeded = shared(64);
    for chain in [&mut original, &mut seeded] {
        assert_eq!(
            chain
                .get_scope_mut(ScopeId::ROOT)
                .and_then(|root| root.get_binding_mut("absent"))
                .map(|binding| binding.binding_type),
            None
        );
    }
    assert_eq!(borrowed(&seeded), vec![true, true, true, true, false]);

    let browser = seeded.find_scope_by_kind(ScopeKind::JsGlobalBrowser);
    for chain in [&mut original, &mut seeded] {
        chain.enter_scope_with_vue_global(ScopeKind::VFor);
        chain.mark_used("Math");
        chain.mark_used("$emit");
        chain.mark_used("absentToo");
        if let Some(browser) = browser {
            chain.set_current(browser);
        }
        chain.mark_mutated("window");
        chain.set_current(ScopeId::ROOT);
        chain.add_binding(
            CompactString::new("Math"),
            ScopeBinding::new(BindingType::SetupRef, 17),
        );
        chain.add_binding(
            CompactString::new("local"),
            ScopeBinding::new(BindingType::SetupLet, 29),
        );
    }
    assert_eq!(format!("{seeded:?}"), format!("{original:?}"));
    assert_eq!(walk(&seeded), walk(&original));
    assert_eq!(
        borrowed(&seeded),
        vec![false, false, true, false, false, false]
    );
    assert_eq!(
        seeded.lookup("Math").map(|(scope, binding)| (
            scope.id,
            binding.binding_type,
            binding.declaration_offset
        )),
        Some((ScopeId::ROOT, BindingType::SetupRef, 17))
    );

    // Negative control: one flag the other chain lacks must show.
    seeded.set_current(ScopeId::ROOT);
    seeded.mark_used("JSON");
    assert_ne!(walk(&seeded), walk(&original));
    assert_eq!(
        globals::UNIVERSAL.bindings.get("JSON").map(|b| b.is_used()),
        Some(false)
    );
}
