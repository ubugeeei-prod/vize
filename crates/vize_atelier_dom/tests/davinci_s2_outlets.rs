//! P2-11 slot-outlet witness: `_renderSlot`, fallback, camelized props,
//! `v-if` / `v-for` outlets, and `_: 3 /* FORWARDED */`, compared
//! **byte-for-byte** including helper usage.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

mod support;

const BATTERY: &[(&str, &str)] = &[
    ("bare", "<slot></slot>"),
    ("self_close", "<slot/>"),
    ("bare_name_attr", r#"<slot name></slot>"#),
    ("named", r#"<slot name="header"></slot>"#),
    ("bracket_literal_name", r#"<slot name="[header]"></slot>"#),
    ("dynamic_name", r#"<slot :name="n"></slot>"#),
    (
        "dynamic_member_name",
        r#"<slot :name="tabs[index]"></slot>"#,
    ),
    ("same_name_shorthand", r#"<slot :name></slot>"#),
    ("same_name_longhand", r#"<slot v-bind:name></slot>"#),
    ("blank_dynamic_name", r#"<slot :name=""></slot>"#),
    ("fallback_text", "<slot>fallback</slot>"),
    ("fallback_interp", "<slot>hello {{ msg }}</slot>"),
    ("fallback_span", "<slot><span></span></slot>"),
    ("static_prop", r#"<slot foo="bar"></slot>"#),
    ("hyphen_prop", r#"<slot foo-bar="x"></slot>"#),
    ("bind_prop", r#"<slot :foo="bar"></slot>"#),
    ("bind_hyphen", r#"<slot :foo-bar="x"></slot>"#),
    ("prop_and_fallback", r#"<slot foo="bar">fb</slot>"#),
    ("object_bind", r#"<slot v-bind="obj"></slot>"#),
    ("in_div", "<div><slot></slot></div>"),
    ("forwarded", "<Foo><slot></slot></Foo>"),
    ("forwarded_nested", "<Foo><div><slot></slot></div></Foo>"),
    ("vif", r#"<slot v-if="ok"></slot>"#),
    ("vif_fallback", r#"<slot v-if="ok">x</slot>"#),
    ("vif_else", r#"<slot v-if="a"></slot><slot v-else></slot>"#),
    ("vfor", r#"<slot v-for="i in n"></slot>"#),
    ("vfor_fallback", r#"<slot v-for="i in n">x</slot>"#),
    (
        "scoped_forwarded",
        r#"<Bar v-slot="p"><Foo><slot></slot></Foo></Bar>"#,
    ),
    (
        "named_mixed_props",
        r#"<slot name="header" foo="1" :bar="b"></slot>"#,
    ),
];

#[test]
fn s2_slot_outlets_match_the_shipped_dom_lane_byte_for_byte() {
    support::assert_s2_matches_shipped(BATTERY);
}
