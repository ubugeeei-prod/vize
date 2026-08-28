//! P2-11 named / scoped slot witness: `<template #name>` groups,
//! component-root `v-slot` (bare defaults, named keys preserved), dynamic
//! names, simple scoped params, and `createSlots` (`v-if` / `v-for`
//! slot templates), compared **byte-for-byte** including helper usage.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

mod support;

const BATTERY: &[(&str, &str)] = &[
    (
        "named_header_text",
        "<Foo><template #header>title</template></Foo>",
    ),
    (
        "named_header_interp",
        "<Foo><template #header>hello {{ msg }}</template></Foo>",
    ),
    (
        "named_then_default",
        "<Foo><template #header>title</template>hello</Foo>",
    ),
    ("bare_template_default", "<Foo><template>x</template></Foo>"),
    (
        "default_then_named",
        "<Foo>hello<template #header>title</template></Foo>",
    ),
    (
        "named_header_span",
        "<Foo><template #header><span></span></template></Foo>",
    ),
    (
        "named_header_extra_attr",
        r#"<Foo><template #header id="x">x</template></Foo>"#,
    ),
    (
        "named_header_v_once",
        r#"<Foo><template #header v-once>x</template></Foo>"#,
    ),
    (
        "named_header_v_memo",
        r#"<Foo><template #header v-memo="[ok]">x</template></Foo>"#,
    ),
    (
        "hyphenated_slot",
        "<Foo><template #foo-bar>x</template></Foo>",
    ),
    (
        "two_named",
        "<Foo><template #header>title</template><template #footer>end</template></Foo>",
    ),
    ("empty_named", "<Foo><template #header></template></Foo>"),
    ("ws_named", "<Foo><template #header>  </template></Foo>"),
    ("component_v_slot_header", "<Foo v-slot:header>title</Foo>"),
    ("component_v_slot", "<Foo v-slot>title</Foo>"),
    (
        "component_v_slot_empty_params",
        r#"<Foo v-slot="">title</Foo>"#,
    ),
    ("component_hash_header", "<Foo #header>title</Foo>"),
    (
        "dynamic_slot_name",
        r#"<Foo><template #[name]>x</template></Foo>"#,
    ),
    (
        "named_slot_empty_params",
        r#"<Foo><template #header="">x</template></Foo>"#,
    ),
    (
        "scoped_ident",
        r#"<Foo><template #header="p">x</template></Foo>"#,
    ),
    (
        "scoped_destructure",
        r#"<Foo><template #header="{ foo }">x</template></Foo>"#,
    ),
    (
        "create_slots_if",
        r#"<Foo><template #header v-if="ok">x</template></Foo>"#,
    ),
    (
        "create_slots_if_v_once",
        r#"<Foo><template #header v-if="ok" v-once>x</template></Foo>"#,
    ),
    (
        "create_slots_if_v_memo",
        r#"<Foo><template #header v-if="ok" v-memo="[ok]">x</template></Foo>"#,
    ),
    (
        "create_slots_empty_params",
        r#"<Foo><template #header="" v-if="ok">x</template></Foo>"#,
    ),
    (
        "create_slots_if_extra_attr",
        r#"<Foo><template #header id="x" v-if="ok">x</template></Foo>"#,
    ),
    (
        "create_slots_for",
        r#"<Foo><template v-for="i in n" #header>x</template></Foo>"#,
    ),
    (
        "create_slots_for_v_once",
        r#"<Foo><template v-for="i in n" #header v-once>x</template></Foo>"#,
    ),
    (
        "create_slots_for_v_memo",
        r#"<Foo><template v-for="i in n" #header v-memo="[i]">x</template></Foo>"#,
    ),
    (
        "create_slots_for_extra_attr",
        r#"<Foo><template v-for="i in n" #header id="x">x</template></Foo>"#,
    ),
    (
        "create_slots_if_and_static",
        r#"<Foo><template #header v-if="ok">x</template><template #footer>end</template></Foo>"#,
    ),
    (
        "create_slots_default_and_if",
        r#"<Foo>hello<template #header v-if="ok">x</template></Foo>"#,
    ),
    (
        "create_slots_if_else",
        r#"<Foo><template #header v-if="a">x</template><template #header v-else>y</template></Foo>"#,
    ),
    (
        "create_slots_if_else_if",
        r#"<Foo><template #header v-if="a">x</template><template #header v-else-if="b">y</template></Foo>"#,
    ),
    (
        "create_slots_if_span",
        r#"<Foo><template #header v-if="ok"><span></span></template></Foo>"#,
    ),
    (
        "create_slots_scoped",
        r#"<Foo><template #header="p" v-if="ok">x</template></Foo>"#,
    ),
    (
        "create_slots_dynamic_name",
        r#"<Foo><template #[name] v-if="ok">x</template></Foo>"#,
    ),
    (
        "create_slots_hyphenated",
        r#"<Foo><template #foo-bar v-if="ok">x</template></Foo>"#,
    ),
    (
        "create_slots_empty",
        r#"<Foo><template #header v-if="ok"></template></Foo>"#,
    ),
    (
        "create_slots_for_aliases",
        r#"<Foo><template v-for="(v, k, i) in n" #header>x</template></Foo>"#,
    ),
    (
        "create_slots_default_interp",
        r#"<Foo>hello {{ msg }}<template #header v-if="ok">x</template></Foo>"#,
    ),
    (
        "create_slots_nested_template",
        r#"<Foo><template #header v-if="ok"><template #inner>x</template></template></Foo>"#,
    ),
    (
        "create_slots_nested_template_interp",
        r#"<Foo><template v-for="i in n" #header><template #inner>{{ i }}</template></template></Foo>"#,
    ),
    (
        "nested_named_slot_template",
        r#"<Foo><template #header><template #inner>x</template></template></Foo>"#,
    ),
    (
        "nested_named_slot_template_interp",
        r#"<Foo><template #header><template #inner>{{ msg }}</template></template></Foo>"#,
    ),
    (
        "nested_named_slot_template_multiple",
        r#"<Foo><template #header><template #inner><b></b><i></i></template></template></Foo>"#,
    ),
    (
        "nested_named_slot_template_empty",
        r#"<Foo><template #header><template #inner></template></template></Foo>"#,
    ),
    (
        "stray_named_slot_template_inside_native",
        r#"<div><template #inner>x</template></div>"#,
    ),
    (
        "stray_named_slot_template_interp",
        r#"<div><template #inner>{{ msg }}</template></div>"#,
    ),
    (
        "stray_named_slot_template_multiple",
        r#"<div><template #inner><b></b><i></i></template></div>"#,
    ),
    (
        "stray_named_slot_template_empty",
        r#"<div><template #inner></template></div>"#,
    ),
    (
        "slot_outlet_fallback_stray_template",
        r#"<slot><template #inner>{{ msg }}</template></slot>"#,
    ),
    (
        "unwrapped_if_nested_slot_keeps_siblings",
        r#"<Foo><template v-if="ok"><span>x</span><template #header>y</template></template></Foo>"#,
    ),
    (
        "unwrapped_for_nested_slot_keeps_siblings",
        r#"<Foo><template v-for="i in n"><span>x</span><template #header>y</template></template></Foo>"#,
    ),
    (
        "dynamic_slot_name_hole",
        r#"<Foo><template #[]>x</template></Foo>"#,
    ),
];

const UNSUPPORTED_BATTERY: &[(&str, &str, support::ExpectedRefusal)] = &[(
    "mixed_component_root_and_named_template",
    r#"<Foo v-slot><template #header>x</template></Foo>"#,
    support::ExpectedRefusal::Diagnostics,
)];

#[test]
fn s2_named_slots_match_the_shipped_dom_lane_byte_for_byte() {
    support::assert_s2_matches_shipped(BATTERY);
}

#[test]
fn s2_slot_forms_that_stay_unsupported_are_pinned_negative_cases() {
    support::assert_s2_refuses(UNSUPPORTED_BATTERY);
}
