//! P2-11 helper ordering and component-slot composition witnesses.
//!
//! These are reduced from the hydrated DOM corpus divergences where
//! component slots combine text helpers, conditional comment helpers,
//! and slot wrappers in one helper preamble.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

mod support;

const BATTERY: &[(&str, &str)] = &[
    (
        "component_text_then_if_span",
        r#"<Foo>hello<span v-if="ok">x</span></Foo>"#,
    ),
    (
        "component_if_span_then_text",
        r#"<Foo><span v-if="ok">x</span>hello</Foo>"#,
    ),
    (
        "component_template_if_text_then_text",
        r#"<Foo><template v-if="ok">x</template>hello</Foo>"#,
    ),
    (
        "component_text_then_template_if_text",
        r#"<Foo>hello<template v-if="ok">x</template></Foo>"#,
    ),
    (
        "component_text_then_conditional_named_slot",
        r#"<Foo>hello<template #header v-if="ok">x</template></Foo>"#,
    ),
    (
        "component_conditional_named_slot_then_text",
        r#"<Foo><template #header v-if="ok">x</template>hello</Foo>"#,
    ),
    (
        "component_nested_text_then_if_component",
        r#"<Foo>hello<Bar v-if="ok">x</Bar></Foo>"#,
    ),
    (
        "component_if_component_then_text",
        r#"<Foo><Bar v-if="ok">x</Bar>hello</Foo>"#,
    ),
    (
        "text_slot_before_v_if_fallback",
        r#"<Foo>hello<template v-if="ok"><span>x</span></template></Foo>"#,
    ),
    (
        "v_show_sibling_before_v_for",
        r#"<section><div v-show="visible"></div><p v-for="item in items">{{ item }}</p></section>"#,
    ),
    (
        "v_show_component_slot",
        r#"<Dialog v-show="open"><span>Body</span></Dialog>"#,
    ),
    (
        "nested_trigger_icon_resolves_in_child_before_parent_order",
        r#"<DropdownMenu><DropdownMenuTrigger><Icon /></DropdownMenuTrigger></DropdownMenu>"#,
    ),
];

#[test]
fn s2_helper_composition_matches_the_shipped_dom_lane_byte_for_byte() {
    support::assert_s2_matches_shipped(BATTERY);
}
