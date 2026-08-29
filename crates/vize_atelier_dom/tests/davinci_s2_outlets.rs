//! P2-11 slot-outlet witness: `_renderSlot`, fallback, camelized props,
//! named and dynamic event props, `v-if` / `v-for` outlets, and
//! `_: 3 /* FORWARDED */`, compared
//! **byte-for-byte** including helper usage.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

mod support;

use vize_s0::config::VueVersion;

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
    ("static_event", r#"<slot @pick="choose"></slot>"#),
    ("inline_event", r#"<slot @pick="choose(row)"></slot>"#),
    ("event_modifier", r#"<slot @pick.stop="choose"></slot>"#),
    (
        "event_key_modifier",
        r#"<slot @keyup.enter.stop="choose"></slot>"#,
    ),
    (
        "event_option_modifiers",
        r#"<slot @click.once.capture.passive="choose"></slot>"#,
    ),
    (
        "colon_and_custom_casing",
        r#"<slot @update:modelValue="sync" @customEvent="custom"></slot>"#,
    ),
    ("dynamic_event", r#"<slot @[event]="handler"></slot>"#),
    (
        "dynamic_event_modifier",
        r#"<slot @[event].enter.stop="handler"></slot>"#,
    ),
    (
        "dynamic_event_option_modifiers",
        r#"<slot @[event].once.capture.passive="handler"></slot>"#,
    ),
    (
        "dynamic_prop_and_dynamic_event",
        r#"<slot :[propKey]="value" @[event]="handler"></slot>"#,
    ),
    (
        "duplicate_events",
        r#"<slot @click="a" @click.stop="b"></slot>"#,
    ),
    (
        "event_with_name_and_props",
        r#"<slot name="cell" tone="brisk" :item="row" @pick="choose(row)"></slot>"#,
    ),
    ("prop_and_fallback", r#"<slot foo="bar">fb</slot>"#),
    ("event_and_fallback", r#"<slot @click="handler">fb</slot>"#),
    ("object_bind", r#"<slot v-bind="obj"></slot>"#),
    (
        "object_on_modifier",
        r#"<slot v-on.once="listeners"></slot>"#,
    ),
    (
        "event_then_bind_spread",
        r#"<slot @pick="choose" v-bind="slotProps"></slot>"#,
    ),
    (
        "bind_spread_then_event",
        r#"<slot v-bind="slotProps" @pick="choose"></slot>"#,
    ),
    (
        "event_with_object_on",
        r#"<slot @pick="choose" v-on="listeners"></slot>"#,
    ),
    ("in_div", "<div><slot></slot></div>"),
    ("forwarded", "<Foo><slot></slot></Foo>"),
    ("forwarded_nested", "<Foo><div><slot></slot></div></Foo>"),
    ("vif", r#"<slot v-if="ok"></slot>"#),
    ("vif_fallback", r#"<slot v-if="ok">x</slot>"#),
    ("vif_else", r#"<slot v-if="a"></slot><slot v-else></slot>"#),
    ("vfor", r#"<slot v-for="i in n"></slot>"#),
    ("vfor_fallback", r#"<slot v-for="i in n">x</slot>"#),
    (
        "vfor_dynamic_event_local_name",
        r#"<slot v-for="item in items" @[item.event]="item.handler"></slot>"#,
    ),
    (
        "scoped_forwarded_dynamic_event",
        r#"<Bar v-slot="{ row }"><Foo><slot @[row.event]="row.handler"></slot></Foo></Bar>"#,
    ),
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

const VUE2_BATTERY: &[(&str, &str)] = &[(
    "vue2_native_modifier",
    r#"<slot @click.native="handler"></slot>"#,
)];

#[test]
fn s2_vue2_slot_outlet_v_on_matches_the_shipped_dom_lane_byte_for_byte() {
    support::assert_s2_matches_shipped_with_dialect(VUE2_BATTERY, VueVersion::V2);
}
