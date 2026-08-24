//! P2-11 Vue-builtin witness: Teleport / KeepAlive array children,
//! Transition / Suspense slot objects, nested `createBlock`, unused
//! static-props hoists, and helper import order, compared
//! **byte-for-byte** including helper usage.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

use vize_atelier_dom::compile_template;
use vize_carton::Allocator;
use vize_ricalco::{DOM_LANE_FLAG, emit_dom_source};

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

fn shipped(src: &str) -> String {
    let allocator = Allocator::new();
    let (_, errors, result) = compile_template(&allocator, src);
    assert!(errors.is_empty(), "shipped lane errors: {errors:?}");
    format!("{}\n{}", result.preamble, result.code)
}

#[test]
fn s2_builtins_match_the_shipped_dom_lane_byte_for_byte() {
    let mut compared = 0u64;
    let mut skipped_legacy_flag = 0u64;
    if std::env::var(DOM_LANE_FLAG).is_ok_and(|value| value == "legacy") {
        skipped_legacy_flag += 1;
    } else {
        let allocator = Allocator::new();
        for (name, src) in BATTERY {
            let old = shipped(src);
            let new = emit_dom_source(&allocator, src)
                .unwrap_or_else(|error| panic!("{name}: S2 emit refused: {error:?}"))
                .assembled();
            assert_eq!(
                old.as_str(),
                new.as_str(),
                "{name}: S2 DOM emit diverged from the shipped lane"
            );
            compared += 1;
        }
    }
    assert_eq!(
        (compared, skipped_legacy_flag),
        (BATTERY.len() as u64, 0),
        "a cfg or {DOM_LANE_FLAG}=legacy regression disarmed the dual-run"
    );
}
