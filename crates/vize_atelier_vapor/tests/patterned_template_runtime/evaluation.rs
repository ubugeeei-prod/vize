use crate::{check, json, paragraph};

#[test]
fn guards_and_render_share_one_property_read() {
    check(
        r#"<template v-match="subject"><p v-when="{ const field } if (field === 1)">{{ field }}</p><p v-when="_">other</p></template>"#,
        json!([{"scenario": "changing-getter", "trees": [paragraph("1")], "reads": 1}]),
    );
}

#[test]
fn nested_shapes_do_not_repeat_parent_getters() {
    check(
        r#"<template v-match="subject"><p v-when="{ field: { const value } }">{{ value }}</p><p v-when="_">other</p></template>"#,
        json!([{"scenario": "nested-getter", "trees": [paragraph("nested")], "reads": 1}]),
    );
}

#[test]
fn guards_and_render_share_the_same_object_and_array_rest() {
    for (pattern, subject) in [
        (
            "{ const field, ...const rest }",
            json!({"field": 1, "extra": 2}),
        ),
        ("[const field, ...const rest]", json!([1, 2, 3])),
    ] {
        check(
            &format!(
                r#"<template v-match="subject"><p v-when="{pattern} if (remember(rest))">{{{{ same(rest) }}}}</p><p v-when="_">other</p></template>"#
            ),
            json!([{"scenario": "rest-identity", "context": {"subject": subject}, "trees": [paragraph("true")]}]),
        );
    }
}

#[test]
fn as_patterns_bind_without_an_obsolete_const_keyword() {
    check(
        r#"<template v-match="subject"><p v-when="{ field: 1 as value } as whole if (whole.field === value)">{{ value }}</p><p v-when="_">other</p></template>"#,
        json!([{"context": {"subject": {"field": 1}}, "trees": [paragraph("1")]}]),
    );
}

#[test]
fn generated_match_scope_does_not_shadow_enclosing_values() {
    check(
        r#"<template v-match="subject"><p v-when="const value if (__vize_match === 'outer')">{{ value }}</p><p v-when="_">other</p></template>"#,
        json!([{"context": {"subject": "inner", "__vize_match": "outer"}, "trees": [paragraph("inner")]}]),
    );
}

#[test]
fn generated_names_respect_entities_and_escaped_identifiers() {
    for reference in [
        "&#95;_vize_match_0_select_0",
        r"\u005f_vize_match_0_select_0",
    ] {
        check(
            &format!(
                r#"<template v-match="subject"><p v-when="const value if ({reference} === 'outer')">{{{{ value }}}}</p><p v-when="_">other</p></template>"#
            ),
            json!([{"context": {"subject": "inner", "__vize_match_0_select_0": "outer"}, "trees": [paragraph("inner")]}]),
        );
    }
}

#[test]
fn value_patterns_resolve_outside_the_binding_scope() {
    check(
        r#"<template v-match="subject"><p v-when="expected as expected if (expected === 'ok')">{{ expected }}</p><p v-when="_">other</p></template>"#,
        json!([{"context": {"subject": "ok", "expected": "ok"}, "trees": [paragraph("ok")]}]),
    );
}

#[test]
fn object_rest_preserves_symbols_descriptors_and_proto_values_without_extra_reads() {
    check(
        r#"<template v-match="subject"><p v-when="{ field: 1, ...const rest } if (remember(rest))">{{ inspectRest(rest) }}</p><p v-when="_">other</p></template>"#,
        json!([{"scenario": "object-rest-copies", "trees": [paragraph("verified")], "reads": 2}]),
    );
}

#[test]
fn rest_copies_wait_for_the_entire_shape_to_match() {
    for (pattern, array) in [
        ("{ field: { ...const rest }, missing: _ }", false),
        ("[{ ...const rest }, 2]", true),
    ] {
        check(
            &format!(
                r#"<template v-match="subject"><p v-when="{pattern}">matched</p><p v-when="_">other</p></template>"#
            ),
            json!([{"scenario": "deferred-rest-copy", "array": array, "trees": [paragraph("other")], "reads": 0}]),
        );
    }
}

#[test]
fn array_rest_avoids_slice_and_species() {
    check(
        r#"<template v-match="subject"><p v-when="[1, ...const rest] if (remember(rest))">{{ inspectRest(rest) }}</p><p v-when="_">other</p></template>"#,
        json!([{"scenario": "array-rest-copies", "trees": [paragraph("verified")]}]),
    );
}

#[test]
fn value_patterns_read_once_and_match_nan() {
    check(
        r#"<template v-match="subject"><p v-when="expected.value as expected if (Number.isNaN(expected))">matched</p><p v-when="_">other</p></template>"#,
        json!([{"scenario": "nan-value", "trees": [paragraph("matched")], "reads": 1}]),
    );
}

#[test]
fn nested_matches_and_same_name_loop_sources_keep_lexical_scopes_on_updates() {
    let section =
        |value: &str| json!({"tag": "section", "attributes": {}, "children": paragraph(value)});
    check(
        r#"<template v-match="subject"><template v-when="{ const rows }"><section v-for="rows in rows" :key="rows.value"><template v-match="rows"><p v-when="{ const value }">{{ value }}</p></template></section></template><p v-when="_">other</p></template>"#,
        json!([{"context": {"subject": {"rows": [{"value": "a"}, {"value": "b"}]}}, "steps": [{"patch": {"subject": {"rows": [{"value": "b"}, {"value": "c"}]}}}], "trees": [[section("a"), section("b")], [section("b"), section("c")]]}]),
    );
}

#[test]
fn comma_subjects_are_one_expression_and_empty_arms_do_not_fall_through() {
    check(
        r#"<template v-match="readSubject(), 'selected'"><template v-when="'selected'"></template><p v-when="_">other</p></template>"#,
        json!([{"scenario": "subject-evaluation", "context": {"subject": "ignored"}, "trees": [[]], "reads": 1}]),
    );
}

#[test]
fn selection_skips_later_arms_and_guards_after_a_shape_failure() {
    check(
        r#"<template v-match="subject"><p v-when="'missing' if (unreachable())">wrong</p><p v-when="'selected'">selected</p><p v-when="unreachable.value if (unreachable())">wrong</p><p v-when="_">other</p></template>"#,
        json!([{"context": {"subject": "selected"}, "trees": [paragraph("selected")]}]),
    );
}
