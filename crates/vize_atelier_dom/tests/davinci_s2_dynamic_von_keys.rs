//! P2-11 dynamic `v-on` key witness: modifier-free computed event keys
//! compare byte-for-byte against the shipped DOM lane, while modifiers,
//! non-JS event names and slot-outlet named events stay typed refusals.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

mod support;

use vize_s1_to_s2::UnsupportedReason as Reason;

const BATTERY: &[(&str, &str)] = &[
    ("native_dynamic", r#"<button @[event]="handler"></button>"#),
    (
        "native_dynamic_member",
        r#"<button @[item.event]="handler"></button>"#,
    ),
    (
        "native_dynamic_call",
        r#"<button @[eventOf()]="handler"></button>"#,
    ),
    (
        "native_dynamic_inline",
        r#"<button @[event]="handler($event)"></button>"#,
    ),
    ("component_dynamic", r#"<Foo @[event]="handler" />"#),
    (
        "static_then_dynamic",
        r#"<button @click="onClick" @[event]="handler"></button>"#,
    ),
    (
        "dynamic_then_static",
        r#"<button @[event]="handler" @click="onClick"></button>"#,
    ),
    (
        "merge_props_dynamic",
        r#"<button v-bind="bag" @[event]="handler"></button>"#,
    ),
    (
        "object_on_then_dynamic",
        r#"<button v-on="bag" @[event]="handler"></button>"#,
    ),
    (
        "v_if_dynamic",
        r#"<button v-if="ok" @[event]="handler">x</button>"#,
    ),
    (
        "v_for_dynamic_local_name",
        r#"<button v-for="item in items" @[item.event]="item.handler"></button>"#,
    ),
    (
        "text_and_dynamic_event",
        r#"<button @[event]="handler">{{ msg }}</button>"#,
    ),
    (
        "dynamic_prop_and_dynamic_event",
        r#"<button :[key]="value" @[event]="handler"></button>"#,
    ),
    (
        "dynamic_slot_and_dynamic_event",
        r#"<Foo @[event]="handler"><template #[name]>x</template></Foo>"#,
    ),
];

const REFUSALS: &[(&str, &str, support::ExpectedRefusal)] = &[
    (
        "dynamic_event_with_modifier",
        r#"<button @[event].stop="handler"></button>"#,
        support::ExpectedRefusal::Unsupported(Reason::DynamicOnHasModifiers),
    ),
    (
        "dynamic_event_name_not_js",
        r#"<button @[event.]="handler"></button>"#,
        support::ExpectedRefusal::Unsupported(Reason::OnNameNotJs),
    ),
    (
        "dynamic_event_handler_not_js",
        r#"<button @[event]="handler."></button>"#,
        support::ExpectedRefusal::Unsupported(Reason::OnHandlerNotJs),
    ),
    (
        "slot_outlet_dynamic_event",
        r#"<slot @[event]="handler"></slot>"#,
        support::ExpectedRefusal::Unsupported(Reason::SlotOutletPropKind),
    ),
];

#[test]
fn s2_dynamic_v_on_keys_match_the_shipped_dom_lane_byte_for_byte() {
    support::assert_s2_matches_shipped(BATTERY);
}

#[test]
fn s2_dynamic_v_on_refusal_boundary_is_typed() {
    support::assert_s2_refuses(REFUSALS);
}
