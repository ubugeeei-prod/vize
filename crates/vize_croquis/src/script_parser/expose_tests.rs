//! The runtime public-instance object, not type-only declarations or guessed keys.

use super::parse_script_setup;
use crate::facts::reactivity_sources;
use crate::{Drawer, DrawerOptions};
use vize_l3::lattice::{ReactivityClass, Verdict};

#[test]
fn aliases_resolve_after_all_declarations_with_exact_utf8_spans() {
    let source = r#"const unicode = '東京🦀';
defineExpose({ publicCount: count, doubled, 'public method': save, unknown, expression: count.value })
const count = ref(0);
const doubled = computed(() => count.value * 2);
function save(value: 'two  spaces'): string { return value }
"#;
    let result = parse_script_setup(source);
    assert!(result.macros.expose_is_complete());
    let names: Vec<_> = result
        .macros
        .exposes()
        .iter()
        .map(|item| item.name.as_str())
        .collect();
    assert_eq!(
        names,
        [
            "publicCount",
            "doubled",
            "public method",
            "unknown",
            "expression"
        ]
    );
    let count = result.macros.expose_bindings().first().unwrap();
    assert_eq!(count.local_name.as_deref(), Some("count"));
    let (start, end) = count.declaration_span.unwrap();
    assert_eq!(source.get(start as usize..end as usize), Some("count"));
    assert_eq!(start as usize, source.find("count = ref").unwrap());
    let save = result
        .macros
        .exposes()
        .iter()
        .find(|item| item.name == "public method")
        .unwrap();
    assert_eq!(
        save.expose_type.as_deref(),
        Some("(value: 'two  spaces') => string")
    );
    for name in ["unknown", "expression"] {
        let item = result
            .macros
            .expose_bindings()
            .iter()
            .find(|item| item.name == name)
            .unwrap();
        assert_eq!(item.local_name, None);
        assert_eq!(item.declaration_span, None);
    }
}

#[test]
fn exposure_aliases_use_the_authoritative_lattice_row() {
    let source = "const count = ref(0); function nested() { const count = computed(() => 1); } defineExpose({ publicCount: count });";
    let mut drawer = Drawer::with_options(DrawerOptions::full());
    drawer.draw_script_setup(source);
    let mut croquis = drawer.finish();
    assert_eq!(croquis.semantic_summary().exposed_binding_count, 1);
    let alias = croquis.macros.expose_bindings().first().unwrap();
    let span = alias.declaration_span.unwrap();
    let rows = reactivity_sources(&croquis);
    let source = rows
        .iter()
        .find(|row| {
            Some(row.name.as_str()) == alias.local_name.as_deref()
                && span.0 == row.declaration_offset
        })
        .unwrap();
    assert_eq!(source.class, ReactivityClass::Reactive);
    assert_eq!(source.verdict, Verdict::Proven);
    croquis.shift_script_offsets(17);
    let alias = croquis.macros.expose_bindings().first().unwrap();
    assert_eq!(alias.declaration_span, Some((span.0 + 17, span.1 + 17)));
    let rows = reactivity_sources(&croquis);
    let source = rows
        .iter()
        .find(|row| {
            Some(row.name.as_str()) == alias.local_name.as_deref()
                && Some(row.declaration_offset) == alias.declaration_span.map(|span| span.0)
        })
        .unwrap();
    assert_eq!(source.class, ReactivityClass::Reactive);
    assert_eq!(source.verdict, Verdict::Proven);
}

#[test]
fn last_static_properties_win_and_unknown_overrides_are_explicit() {
    let result = parse_script_setup(
        "const first=1, last=2, spread={}; defineExpose({ same: first, ['literal']: first, ...spread, same: last, final: last });",
    );
    assert!(!result.macros.expose_is_complete());
    let bindings: Vec<_> = result
        .macros
        .expose_bindings()
        .iter()
        .map(|binding| (binding.name.as_str(), binding.local_name.as_deref()))
        .collect();
    assert_eq!(bindings, [("same", Some("last")), ("final", Some("last"))]);
    let result = parse_script_setup(
        "const first=1, last=2; defineExpose({ same:first, ['literal']: first, same:last, ['__proto__']:last });",
    );
    assert!(result.macros.expose_is_complete());
    let bindings: Vec<_> = result
        .macros
        .expose_bindings()
        .iter()
        .map(|binding| (binding.name.as_str(), binding.local_name.as_deref()))
        .collect();
    assert_eq!(
        bindings,
        [
            ("literal", Some("first")),
            ("same", Some("last")),
            ("__proto__", Some("last"))
        ]
    );
    for source in [
        "const key='count', count=ref(0); defineExpose({ count, [key]: count })",
        "const count=ref(0); defineExpose({ ...{ count } })",
        "const object={}; defineExpose(object)",
        "defineExpose<{ count: number }>()",
        "const result=defineExpose({ count: 1 })",
        "defineExpose({ one: 1 }); defineExpose({ two: 2 })",
        "defineExpose({ __proto__: { inherited: 1 } })",
    ] {
        assert!(
            !parse_script_setup(source).macros.expose_is_complete(),
            "{source}"
        );
    }
    let result = parse_script_setup(
        "const count=ref(0); defineExpose({ __proto__: { inherited: 1 }, count })",
    );
    assert!(!result.macros.expose_is_complete());
    assert_eq!(result.macros.exposes().len(), 1);
    assert_eq!(result.macros.exposes().first().unwrap().name, "count");
}

#[test]
fn lexical_macro_shadowing_and_nested_calls_do_not_expose_members() {
    for source in [
        "defineExpose({ value: 1 }); function defineExpose(value) { return value }",
        "const defineExpose=(value)=>value; defineExpose({ value:1 })",
        "import { defineExpose } from 'another-library'; defineExpose({ value:1 })",
        "function nested() { defineExpose({ value:1 }) }",
    ] {
        let result = parse_script_setup(source);
        assert!(result.macros.exposes().is_empty(), "{source}");
    }
    let result = parse_script_setup(
        "import { defineExpose } from 'vue'; const count=ref(0); defineExpose({count})",
    );
    assert_eq!(result.macros.exposes().len(), 1);
}

#[test]
fn explicit_types_preserve_literal_spaces_and_omit_function_bodies() {
    let original = "const label: 'a  b' = 'a  b'; function save(input: string, optional: number = 1, ...rest: boolean[]): void { console.log(input) } defineExpose({ label, save, inline: (item: number): number => item + 1, cast: label as 'c  d', constant: 'literal' as const, assertion: <const>{ value: 1 }, get read(): number { return 1 }, get __proto__(): number { return 1 }, untyped: (item) => item });";
    let edited = original
        .replace(
            "console.log(input)",
            "console.log('a private implementation change')",
        )
        .replace("item + 1", "item + 20");
    let types = |source: &str| {
        parse_script_setup(source)
            .macros
            .exposes()
            .iter()
            .map(|item| (item.name.clone(), item.expose_type.clone()))
            .collect::<Vec<_>>()
    };
    assert_eq!(types(original), types(&edited));
    let result = parse_script_setup(original);
    let ty = |name: &str| {
        result
            .macros
            .exposes()
            .iter()
            .find(|item| item.name == name)
            .unwrap()
            .expose_type
            .as_deref()
    };
    assert_eq!(ty("label"), Some("'a  b'"));
    assert_eq!(
        ty("save"),
        Some("(input: string, optional?: number, ...rest: boolean[]) => void")
    );
    assert_eq!(ty("inline"), Some("(item: number) => number"));
    assert_eq!(ty("cast"), Some("'c  d'"));
    assert_eq!(ty("constant"), None);
    assert_eq!(ty("assertion"), None);
    assert_eq!(ty("read"), None);
    assert_eq!(ty("__proto__"), None);
    assert_eq!(ty("untyped"), None);
}
