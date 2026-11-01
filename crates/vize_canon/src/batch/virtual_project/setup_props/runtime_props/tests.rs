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
