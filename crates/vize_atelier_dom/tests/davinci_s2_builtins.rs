//! P2-11 Vue-builtin witness: Teleport / KeepAlive array children,
//! Transition / Suspense slot objects, nested `createBlock`, unused
//! static-props hoists, and helper import order, compared
//! **byte-for-byte** including helper usage.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

mod support;

const BATTERY: &[(&str, &str)] = &[
    ("bare_teleport", "<Teleport />"),
    ("teleport_to", r##"<Teleport to="#app" />"##),
    ("teleport_empty", r##"<Teleport to="#app"></Teleport>"##),
    ("kebab_teleport", r##"<teleport to="#app" />"##),
    (
        "teleport_span",
        r##"<Teleport to="#app"><span></span></Teleport>"##,
    ),
    ("teleport_text", r##"<Teleport to="#app">hello</Teleport>"##),
    (
        "teleport_interp",
        r##"<Teleport to="#app">hello {{ msg }}</Teleport>"##,
    ),
    (
        "teleport_bind",
        r#"<Teleport :to="dest"><span></span></Teleport>"#,
    ),
    (
        "nested_teleport",
        r##"<div><Teleport to="#app"><span></span></Teleport></div>"##,
    ),
    ("bare_keepalive", "<KeepAlive />"),
    ("keepalive_foo", "<KeepAlive><Foo /></KeepAlive>"),
    ("kebab_keepalive", "<keep-alive><Foo /></keep-alive>"),
    (
        "nested_keepalive",
        "<div><KeepAlive><Foo /></KeepAlive></div>",
    ),
    (
        "keepalive_in_slot",
        "<Foo><KeepAlive><Bar /></KeepAlive></Foo>",
    ),
    (
        "keepalive_include",
        r#"<KeepAlive include="Foo"><Foo /></KeepAlive>"#,
    ),
    ("bare_transition", "<Transition />"),
    ("transition_name", r#"<Transition name="fade" />"#),
    ("transition_div", "<Transition><div></div></Transition>"),
    (
        "kebab_transition",
        r#"<transition name="fade"><div></div></transition>"#,
    ),
    (
        "nested_transition",
        "<div><Transition><span></span></Transition></div>",
    ),
    ("bare_suspense", "<Suspense />"),
    ("suspense_foo", "<Suspense><Foo /></Suspense>"),
    ("nested_suspense", "<div><Suspense><Foo /></Suspense></div>"),
    (
        "transition_group",
        r#"<TransitionGroup name="list"><div v-for="i in n" :key="i"></div></TransitionGroup>"#,
    ),
    (
        "base_transition",
        "<BaseTransition><div></div></BaseTransition>",
    ),
    (
        "teleport_vif",
        r##"<Teleport v-if="ok" to="#app"><span></span></Teleport>"##,
    ),
    (
        "keepalive_vfor",
        r#"<KeepAlive v-for="i in n"><Foo /></KeepAlive>"#,
    ),
    (
        "transition_vif",
        r#"<Transition v-if="ok"><div></div></Transition>"#,
    ),
    ("teleport_ws", r##"<Teleport to="#app">  </Teleport>"##),
    ("keepalive_ws", "<KeepAlive>  </KeepAlive>"),
];

#[test]
fn s2_builtins_match_the_shipped_dom_lane_byte_for_byte() {
    support::assert_s2_matches_shipped(BATTERY);
}
