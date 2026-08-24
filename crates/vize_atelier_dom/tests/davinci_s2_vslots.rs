//! P2-11 `v-slots` witness: the forwarded object as the children
//! argument, and `...expr` after authored slots, compared
//! **byte-for-byte** including helper usage and the missing `_` flag.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

use vize_atelier_dom::compile_template;
use vize_carton::Allocator;
use vize_ricalco::{emit_dom_source, DOM_LANE_FLAG};

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
];

fn shipped(src: &str) -> String {
    let allocator = Allocator::new();
    let (_, errors, result) = compile_template(&allocator, src);
    assert!(errors.is_empty(), "shipped lane errors: {errors:?}");
    format!("{}\n{}", result.preamble, result.code)
}

#[test]
fn s2_v_slots_match_the_shipped_dom_lane_byte_for_byte() {
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
