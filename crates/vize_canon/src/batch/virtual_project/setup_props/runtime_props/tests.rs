use std::path::Path;

use super::{RuntimePropResolveCache, resolve_inline_runtime_props};
use crate::batch::virtual_project::setup_props::imports::RuntimePropVisitSet;

#[test]
fn inline_expressions_cannot_be_type_query_roots_but_named_spreads_can() {
    for (argument, expected) in [
        ("{ value: { type: null, required: true } }", None),
        (
            "{ ...shared }",
            Some("__RuntimePropShape<typeof shared>[\"value\"]"),
        ),
        ("({ value: { type: null, required: true } } as const)", None),
        (
            "({ value: { type: null, required: true } } satisfies Record<string, unknown>)",
            None,
        ),
        ("({ value: { type: null, required: true } }!)", None),
        (
            "{ ...\\u0073hared }",
            Some("__RuntimePropShape<typeof shared>[\"value\"]"),
        ),
    ] {
        let source = vize_carton::cstr!(
            "const shared = {{ value: {{ type: String, required: true }} }}; defineProps({argument});"
        );
        let props = resolve_inline_runtime_props(
            &source,
            Path::new("fixture.ts"),
            &mut RuntimePropVisitSet::default(),
            &RuntimePropResolveCache::default(),
        );
        let actual: Vec<_> = props
            .iter()
            .map(|prop| {
                (
                    prop.name.as_str(),
                    prop.prop_type.as_deref(),
                    prop.required,
                    prop.default_value.as_deref(),
                )
            })
            .collect();
        assert_eq!(actual, [("value", expected, true, None)], "{argument}");
    }
}

#[test]
fn required_keeps_its_literal_type_through_value_preserving_wrappers() {
    let source = "defineProps({ asConst: { type: String, required: true as const }, angle: { type: String, required: <const>true }, paren: { type: String, required: (true) }, satisfied: { type: String, required: true satisfies boolean }, asTrue: { type: String, required: true as true }, asBoolean: { type: String, required: true as boolean }, off: { type: String, required: false as const } });";
    let props = resolve_inline_runtime_props(
        source,
        Path::new("fixture.ts"),
        &mut RuntimePropVisitSet::default(),
        &RuntimePropResolveCache::default(),
    );
    let actual: Vec<_> = props
        .iter()
        .map(|prop| (prop.name.as_str(), prop.required))
        .collect();
    assert_eq!(
        actual,
        [
            ("asConst", true),
            ("angle", true),
            ("paren", true),
            ("satisfied", true),
            ("asTrue", true),
            ("asBoolean", false),
            ("off", false),
        ]
    );
}
