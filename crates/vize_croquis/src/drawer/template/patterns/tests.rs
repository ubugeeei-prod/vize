use crate::{Croquis, Drawer, DrawerOptions, ScopeData, ScopeKind};
use vize_armature::parse;
use vize_carton::Allocator;

fn analyze(source: &str, script: &str, enabled: bool) -> Croquis {
    let allocator = Allocator::new();
    let (root, errors) = parse(&allocator, source);
    assert!(
        errors.iter().all(|error| error.is_recoverable()),
        "{errors:?}"
    );
    let mut options = DrawerOptions::full();
    options.experimental_patterned_template = enabled;
    let mut drawer = Drawer::with_options(options);
    drawer.draw_script_setup(script);
    drawer.draw_template(&root);
    drawer.finish()
}

#[test]
fn arm_bindings_guards_and_rest_are_lexical() {
    let source = r#"<template v-match="result">
      <p v-when="{ kind: 'ok', data: const rows, ...const rest } as whole if (rows.length > 0)">
        {{ rows }} {{ rest }} {{ whole }}
      </p>
      <p v-when="_">{{ rows }}</p>
    </template><p>{{ rest }} {{ whole }}</p>"#;
    let result = analyze(source, "const result = {};", true);
    assert!(result.pattern_diagnostics.is_empty());
    let missing: Vec<_> = result
        .undefined_refs
        .iter()
        .map(|r| r.name.as_str())
        .collect();
    assert_eq!(missing, ["rows", "rest", "whole"]);
    let scopes: Vec<_> = result
        .scopes
        .iter()
        .filter(|s| s.kind == ScopeKind::VWhen)
        .collect();
    assert_eq!(scopes.len(), 2);
    assert_eq!(result.semantic_summary().template_scope_count, 3);
    for (name, binding) in scopes[0].bindings() {
        let at = binding.declaration_offset as usize;
        assert_eq!(&source[at..at + name.len()], name);
        assert!(binding.is_used(), "{name}");
        assert!(!scopes[1].has_binding(name));
    }
    let guard = result
        .template_expressions
        .iter()
        .find(|e| e.content == "rows.length > 0")
        .unwrap();
    assert_eq!(guard.scope_id, scopes[0].id);
    assert_eq!(
        &source[guard.start as usize..guard.end as usize],
        guard.content.as_str()
    );
}

#[test]
fn values_resolve_before_bindings_and_guards_resolve_after() {
    let source = r#"<div v-match="subject"><p v-when="external as external if (external)">{{ external }}</p></div>"#;
    let result = analyze(source, "const subject = {};", true);
    assert_eq!(result.undefined_refs.len(), 1);
    assert_eq!(result.undefined_refs[0].name, "external");
    assert_eq!(
        result.undefined_refs[0].offset as usize,
        source.find("external").unwrap()
    );
    let values: Vec<_> = result
        .template_expressions
        .iter()
        .filter(|e| e.content == "external")
        .collect();
    assert_eq!(values.len(), 3);
    assert_ne!(values[0].scope_id, values[1].scope_id);
    assert_eq!(values[1].scope_id, values[2].scope_id);
}

#[test]
fn nested_matches_restore_sibling_and_enclosing_scopes() {
    let source = r#"<template v-match="rows">
      <template v-when="const rows">
        <div v-for="rows in rows">
          <template v-match="rows">
            <p v-when="[const item, ...const tail]">{{ item }}{{ tail }}{{ rows }}</p>
            <p v-when="_">{{ item }}</p>
          </template>
        </div>
      </template>
      <p v-when="_">{{ item }}</p>
    </template><p>{{ rows }}</p>"#;
    let result = analyze(source, "const rows = [];", true);
    assert!(result.pattern_diagnostics.is_empty());
    assert_eq!(result.undefined_refs.len(), 2);
    assert!(result.undefined_refs.iter().all(|r| r.name == "item"));
    let loops: Vec<_> = result
        .scopes
        .iter()
        .filter(|s| s.kind == ScopeKind::VFor)
        .collect();
    assert_eq!(loops.len(), 1);
    let parent = result.scopes.get_scope(loops[0].parent().unwrap()).unwrap();
    assert_eq!(parent.kind, ScopeKind::VWhen);
    assert!(parent.get_binding("rows").unwrap().is_used());
    let matches: Vec<_> = result
        .scopes
        .iter()
        .filter(|s| s.kind == ScopeKind::VMatch)
        .collect();
    assert_eq!(matches.len(), 2);
    assert_eq!(matches[1].parent(), Some(loops[0].id));
    let ScopeData::VMatch(data) = matches[1].data() else {
        panic!("missing match data");
    };
    assert_eq!(&source[data.start as usize..data.end as usize], "rows");
}

#[test]
fn declarations_shadow_setup_and_reach_event_and_slot_scopes() {
    let source = r#"<template v-match="subject">
      <template v-when="{ const value }">
        <Child v-slot="{ row }">
          <button @click="() => value + row">{{ value }}</button>
        </Child>
      </template>
      <p v-when="_">{{ value }}</p>
    </template>"#;
    let result = analyze(source, "const subject = {}; const value = 1;", true);
    assert!(result.pattern_diagnostics.is_empty());
    assert!(
        result.undefined_refs.is_empty(),
        "{:?}",
        result.undefined_refs
    );
    let arm = result
        .scopes
        .iter()
        .find(|s| s.kind == ScopeKind::VWhen)
        .unwrap();
    assert!(arm.get_binding("value").unwrap().is_used());
    let event = result
        .scopes
        .iter()
        .find(|s| s.kind == ScopeKind::EventHandler)
        .unwrap();
    let slot = result.scopes.get_scope(event.parent().unwrap()).unwrap();
    assert_eq!(slot.kind, ScopeKind::VSlot);
    assert_eq!(slot.parent(), Some(arm.id));
}

#[test]
fn disabled_mode_preserves_custom_directive_behavior() {
    let source = r#"<div v-match="subject"><p v-when="value">{{ value }}</p></div>"#;
    let result = analyze(source, "const subject = 1;", false);
    assert!(result.pattern_diagnostics.is_empty());
    assert!(
        !result
            .scopes
            .iter()
            .any(|s| matches!(s.kind, ScopeKind::VMatch | ScopeKind::VWhen))
    );
    assert_eq!(
        result
            .undefined_refs
            .iter()
            .filter(|r| r.name == "value")
            .count(),
        2
    );
}

#[test]
fn entity_and_multibyte_offsets_are_authored_bytes() {
    let source = "<template v-match=\"subject\"><p v-when=\"{ kind: &quot;ok&quot;, const rows } as whole if (rows.length &gt; missing)\">{{ whole }}</p></template>";
    let result = analyze(source, "const subject = {};", true);
    assert!(result.pattern_diagnostics.is_empty());
    let arm = result
        .scopes
        .iter()
        .find(|scope| scope.kind == ScopeKind::VWhen)
        .unwrap();
    for (name, binding) in arm.bindings() {
        assert_eq!(
            &source[binding.declaration_offset as usize..][..name.len()],
            name
        );
    }
    assert_eq!(result.undefined_refs.len(), 1);
    assert_eq!(result.undefined_refs[0].name, "missing");
    assert_eq!(
        result.undefined_refs[0].offset as usize,
        source.find("missing").unwrap()
    );
}

#[test]
fn existing_scope_discriminants_remain_stable() {
    for (index, kind) in [
        ScopeKind::Module,
        ScopeKind::Function,
        ScopeKind::Block,
        ScopeKind::VFor,
        ScopeKind::VSlot,
        ScopeKind::EventHandler,
        ScopeKind::Callback,
        ScopeKind::ScriptSetup,
        ScopeKind::NonScriptSetup,
        ScopeKind::Universal,
        ScopeKind::ClientOnly,
        ScopeKind::JsGlobalUniversal,
        ScopeKind::JsGlobalBrowser,
        ScopeKind::JsGlobalNode,
        ScopeKind::JsGlobalDeno,
        ScopeKind::JsGlobalBun,
        ScopeKind::VueGlobal,
        ScopeKind::ExternalModule,
        ScopeKind::Closure,
        ScopeKind::VMatch,
        ScopeKind::VWhen,
    ]
    .into_iter()
    .enumerate()
    {
        assert_eq!(kind as usize, index);
        assert_eq!(crate::display::ScopeKind::from(kind) as usize, index);
    }
}

#[test]
fn invalid_structure_and_patterns_are_diagnosed() {
    for (source, message, warning) in [
        (
            r#"<div v-match="x" v-match="y"><p v-when="_"></p></div>"#,
            "Duplicate v-match",
            false,
        ),
        (
            r#"<div v-match="x"><p v-when="_" v-when="1"></p></div>"#,
            "Duplicate v-when",
            false,
        ),
        (r#"<p v-when="_" />"#, "direct child", false),
        (r#"<div v-match />"#, "requires an expression", false),
        (r#"<div v-match:x="x" />"#, "no arguments", false),
        (
            r#"<div v-match="x"><p v-when.mod="_" /></div>"#,
            "no arguments",
            false,
        ),
        (
            r#"<div v-match="x"><p v-when="_" v-if="x" /></div>"#,
            "another structural",
            false,
        ),
        (
            r#"<div v-match="x"><p v-when="[const x, const x]" /></div>"#,
            "Duplicate",
            false,
        ),
        (
            r#"<div v-match="x"><p v-when="_"/><p v-when="1"/></div>"#,
            "last and unique",
            false,
        ),
        (r#"<div v-match="x"><p /></div>"#, "must declare", true),
        (r#"<div v-match="x"> </div>"#, "no v-when", true),
    ] {
        let result = analyze(source, "const x = 1;", true);
        let diagnostic = result
            .pattern_diagnostics
            .iter()
            .find(|d| d.message.contains(message))
            .unwrap_or_else(|| panic!("{source}: {:?}", result.pattern_diagnostics));
        assert_eq!(diagnostic.warning, warning);
        assert!(diagnostic.start < diagnostic.end);
        assert!((diagnostic.end as usize) <= source.len());
    }
}
