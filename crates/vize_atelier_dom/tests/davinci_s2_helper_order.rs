//! P2-11 helper preamble ordering witnesses reduced from the real DOM corpus.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

mod support;

const BATTERY: &[(&str, &str)] = &[
    (
        "style_before_class_still_destructures_class_before_style",
        r#"<div :style="{ opacity: disabled ? 0.5 : 1 }"></div><div :class="{ active: ok }"></div>"#,
    ),
    (
        "directive_and_key_event_modifiers_keep_vue_helper_order",
        r#"<input v-focus @keyup.enter.stop="handler">"#,
    ),
    (
        "class_before_block_helpers_in_conditional_list",
        r#"<div><button :class="buttonClass" v-if="ok">Play</button><span v-for="item in items">{{ item }}</span></div>"#,
    ),
    (
        "array_template_literal_class_still_normalizes",
        r##"<span :class="[`text-${msg[0]}`, 'text-monospace', 'small', 'd-block']" style="white-space: pre-wrap;">{{ msg[1] }}</span>"##,
    ),
    (
        "transition_group_component_list_keeps_dynamic_class_helper",
        r##"<transition-group tag="ul" name="flip-list"><b-list-group-item v-for="msg in messages" :key="`console-${msg[2]}`"><span :class="[`text-${msg[0]}`, 'text-monospace', 'small', 'd-block']" style="white-space: pre-wrap;">{{ msg[1] }}</span></b-list-group-item></transition-group>"##,
    ),
    (
        "hoisted_style_helper_precedes_body_class_helper",
        r#"<div><Foo :style="{ color: tone }" /><span :class="classes" /></div>"#,
    ),
    (
        "alias_shaped_expression_text_does_not_reorder_helpers",
        r#"<div :class="classes" :style="value === '_normalizeStyle()' ? styles : fallback" />"#,
    ),
];

#[test]
fn s2_helper_order_matches_the_shipped_dom_lane_byte_for_byte() {
    support::assert_s2_matches_shipped(BATTERY);
}
