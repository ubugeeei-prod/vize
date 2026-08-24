//! P2-11 `v-slots` witness: the forwarded object as the children
//! argument, `...expr` after authored slots, and the spread on a
//! `createSlots` base, compared **byte-for-byte**.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

mod support;

const BATTERY: &[(&str, &str)] = &[
    ("only_forwarded", r#"<Comp v-slots="slots" />"#),
    ("self_close_no_space", r#"<Comp v-slots="slots"/>"#),
    (
        "authored_default",
        r#"<Comp v-slots="slots"><span></span></Comp>"#,
    ),
    ("authored_text", r#"<Comp v-slots="slots">hello</Comp>"#),
    (
        "named_then_spread",
        r#"<Comp v-slots="slots"><template #header>x</template></Comp>"#,
    ),
    ("with_static_prop", r#"<Comp id="x" v-slots="slots" />"#),
    (
        "create_if",
        r#"<Foo v-slots="slots"><template #header v-if="ok">x</template></Foo>"#,
    ),
    (
        "create_default_if",
        r#"<Foo v-slots="slots">hello<template #header v-if="ok">x</template></Foo>"#,
    ),
    (
        "create_for",
        r#"<Foo v-slots="slots"><template v-for="i in n" #header>x</template></Foo>"#,
    ),
];

#[test]
fn s2_v_slots_match_the_shipped_dom_lane_byte_for_byte() {
    support::assert_s2_matches_shipped(BATTERY);
}
