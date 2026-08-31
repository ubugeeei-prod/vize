//! P2-11 static+dynamic `style` merge witness: static style attributes
//! beside `:style` compile to the same array/object shape as the shipped
//! DOM lane, including CSS semicolons inside function calls.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

mod support;

const BATTERY: &[(&str, &str)] = &[
    (
        "static_then_dynamic",
        r#"<div style="color: red" :style="s"></div>"#,
    ),
    (
        "dynamic_then_static",
        r#"<div :style="s" style="color: red"></div>"#,
    ),
    (
        "multiline_dynamic_then_static",
        r#"<div
  :style="
    open
      ? { height: '240px' }
      : undefined
  "
  style="height: 200px"
></div>"#,
    ),
    (
        "object_literal",
        r#"<div style="color: red" :style="{ fontSize: x }"></div>"#,
    ),
    (
        "bare_object_literal",
        r#"<div :style="{ color: textColor }"></div>"#,
    ),
    (
        "bare_object_static_value",
        r#"<div :style="{ color: 'red' }"></div>"#,
    ),
    (
        "bare_object_spread",
        r#"<div :style="{ ...styles }"></div>"#,
    ),
    (
        "bare_object_computed_key",
        r#"<div :style="{ [prop]: 'red' }"></div>"#,
    ),
    (
        "component_object_literal",
        r#"<WindowRoot :style="{ left: '50%', top: '72px' }" />"#,
    ),
    (
        "css_function_semicolon",
        r#"<div style="background: url(a;b); color: red" :style="s"></div>"#,
    ),
    (
        "v_if_style_merge",
        r#"<div v-if="ok" style="color: red" :style="s"></div>"#,
    ),
    (
        "v_for_style_merge",
        r#"<div v-for="item in items" :key="item.id" style="color: red" :style="item.style">{{ item.label }}</div>"#,
    ),
    (
        "spread_after_merge",
        r#"<input style="color: red" :style="dynamicStyle" v-bind="attrs" />"#,
    ),
    (
        "spread_before_merge",
        r#"<input v-bind="attrs" style="color: red" :style="dynamicStyle" />"#,
    ),
];

#[test]
fn s2_static_dynamic_style_merges_match_the_shipped_dom_lane_byte_for_byte() {
    support::assert_s2_matches_shipped(BATTERY);
}
