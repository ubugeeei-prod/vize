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

use vize_ricalco::UnsupportedReason as Reason;

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
    (
        "default_then_named",
        "<Foo>hello<template #header>title</template></Foo>",
    ),
    (
        "named_header_span",
        "<Foo><template #header><span></span></template></Foo>",
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
    ("component_hash_header", "<Foo #header>title</Foo>"),
    (
        "dynamic_slot_name",
        r#"<Foo><template #[name]>x</template></Foo>"#,
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
        "create_slots_for",
        r#"<Foo><template v-for="i in n" #header>x</template></Foo>"#,
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
];

const UNSUPPORTED_BATTERY: &[(&str, &str, support::ExpectedRefusal)] = &[
    (
        "mixed_component_root_and_named_template",
        r#"<Foo v-slot><template #header>x</template></Foo>"#,
        support::ExpectedRefusal::Diagnostics,
    ),
    (
        "slot_template_extra_attr",
        r#"<Foo><template #header id="x">x</template></Foo>"#,
        support::ExpectedRefusal::Unsupported(Reason::SlotDefaultShape),
    ),
];

#[test]
fn s2_named_slots_match_the_shipped_dom_lane_byte_for_byte() {
    support::assert_s2_matches_shipped(BATTERY);
}

#[test]
fn s2_slot_forms_that_stay_unsupported_are_pinned_negative_cases() {
    support::assert_s2_refuses(UNSUPPORTED_BATTERY);
}
