use super::super::ScriptCompileContext;
use crate::croquis::{SfcCroquisOptions, analyze_sfc_descriptor_resolved};
use crate::{SfcParseOptions, parse_sfc};

#[test]
fn scoped_union_retains_accessible_names_and_combines_common_member_types() {
    let source = "type Props = { size?: string } & ({ decorative?: true; title?: never; label?: never } | { decorative?: false; title: string; label?: string }); defineProps<Props>()";
    let ctx = ScriptCompileContext::new(source);
    let world = ctx.resolve_type_world("App.vue", None);
    let resolved = world.resolve_properties("Props");
    let actual: Vec<_> = resolved
        .properties
        .iter()
        .map(|prop| (prop.name.as_str(), prop.optional, prop.prop_type.as_deref()))
        .collect();
    assert_eq!(
        actual,
        vec![
            ("size", true, Some("string")),
            ("decorative", true, Some("(true) | (false)")),
            ("title", true, Some("(never) | (string)")),
            ("label", true, Some("(never) | (string)")),
        ]
    );
    assert!(resolved.complete);
}

#[test]
fn union_branch_local_keys_keep_vues_authored_runtime_optionality() {
    let ctx = ScriptCompileContext::new(
        "type Props = { title: string; value: string } | { label?: string; value?: number }; defineProps<Props>()",
    );
    let world = ctx.resolve_type_world("App.vue", None);
    let resolved = world.resolve_properties("Props");
    let actual: Vec<_> = resolved
        .properties
        .iter()
        .map(|prop| (prop.name.as_str(), prop.optional, prop.prop_type.as_deref()))
        .collect();
    assert_eq!(
        actual,
        vec![
            ("title", false, Some("string")),
            ("value", true, Some("(string) | (number)")),
            ("label", true, Some("string")),
        ]
    );
    assert!(resolved.complete);
}

#[test]
fn imported_union_fields_reach_real_croquis_and_keep_their_lexical_scope() {
    let dir = tempfile::tempdir().unwrap();
    let types = dir.path().join("types.ts");
    std::fs::write(
        &types,
        "type Helper = string; export type Props = { size?: number } & ({ title?: never } | { title: Helper; enabled?: boolean })",
    )
    .unwrap();
    let filename = dir.path().join("App.vue");
    let descriptor = parse_sfc(
        "<script setup lang='ts'>import type { Props } from './types'; type Helper = boolean; defineProps<Props>()</script><template><div>{{ title }}</div></template>",
        SfcParseOptions::default(),
    )
    .unwrap();
    let croquis = analyze_sfc_descriptor_resolved(
        &descriptor,
        None,
        SfcCroquisOptions::full(),
        false,
        false,
        filename.to_str().unwrap(),
    )
    .croquis;
    let actual: Vec<_> = croquis
        .macros
        .props()
        .iter()
        .map(|prop| (prop.name.as_str(), prop.required, prop.prop_type.as_deref()))
        .collect();
    assert_eq!(
        actual,
        vec![
            ("size", false, Some("number")),
            ("title", false, Some("(never) | (Helper)")),
            ("enabled", false, Some("boolean")),
        ]
    );
    assert_eq!(croquis.types.resolved_props_complete(), Some(true));
    assert_eq!(
        croquis.types.resolved_prop_module("title"),
        types.canonicalize().unwrap().to_str()
    );
}

#[test]
fn union_shared_foreign_keys_never_capture_equal_alias_names() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("a.ts"),
        "type Helper = string; export interface A { value: Helper; title: string }",
    )
    .unwrap();
    std::fs::write(
        dir.path().join("b.ts"),
        "type Helper = boolean; export interface B { value?: Helper; label: string }",
    )
    .unwrap();
    let ctx = ScriptCompileContext::new(
        "import type { A } from './a'; import type { B } from './b'; type Helper = number; defineProps<A | B>()",
    );
    let world = ctx.resolve_type_world(dir.path().join("App.vue").to_str().unwrap(), None);
    let resolved = world.resolve_properties("A | B");
    assert!(!resolved.complete);
    let value = resolved
        .properties
        .iter()
        .find(|prop| prop.name == "value")
        .unwrap();
    assert!(value.optional);
    assert_eq!(value.prop_type, None);
    let title = resolved
        .properties
        .iter()
        .find(|prop| prop.name == "title")
        .unwrap();
    assert_eq!(title.prop_type.as_deref(), Some("string"));
    assert_eq!(
        title.module,
        dir.path()
            .join("a.ts")
            .canonicalize()
            .unwrap()
            .to_str()
            .unwrap()
    );
}

#[test]
fn unresolved_union_branch_keeps_observed_fields_and_incomplete_status() {
    let ctx = ScriptCompileContext::new("defineProps<{ title: string } | Missing>()");
    let world = ctx.resolve_type_world("App.vue", None);
    let resolved = world.resolve_properties("{ title: string } | Missing");
    assert!(!resolved.complete);
    assert_eq!(resolved.properties.len(), 1);
    assert_eq!(resolved.properties[0].name, "title");
    assert_eq!(resolved.properties[0].prop_type.as_deref(), Some("string"));
}
