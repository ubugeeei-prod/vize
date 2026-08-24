//! P2-11 `<template v-if>` / `<template v-for>` fragment witness,
//! compared **byte-for-byte** including helper usage and hoists.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

use vize_atelier_dom::compile_template;
use vize_carton::Allocator;
use vize_ricalco::{DOM_LANE_FLAG, emit_dom_source};

const BATTERY: &[(&str, &str)] = &[
    (
        "vif_two_span",
        r#"<template v-if="ok"><span></span><span></span></template>"#,
    ),
    (
        "vif_one_span",
        r#"<template v-if="ok"><span></span></template>"#,
    ),
    ("vif_interp", r#"<template v-if="ok">{{ msg }}</template>"#),
    ("vif_text", r#"<template v-if="ok">hello</template>"#),
    (
        "vif_compound",
        r#"<template v-if="ok">hello {{ msg }}</template>"#,
    ),
    (
        "vif_key",
        r#"<template v-if="ok" key="k"><span></span><p></p></template>"#,
    ),
    (
        "vif_else",
        r#"<template v-if="ok"><span>a</span><span>b</span></template><div v-else>no</div>"#,
    ),
    (
        "vif_nested",
        r#"<div><template v-if="ok"><span></span><p></p></template></div>"#,
    ),
    (
        "vif_comp",
        r#"<template v-if="ok"><Foo /><Bar /></template>"#,
    ),
    (
        "vif_text_span",
        r#"<template v-if="ok">hello<span></span></template>"#,
    ),
    (
        "vif_unwrap_dyn",
        r#"<template v-if="ok"><span>{{ msg }}</span></template>"#,
    ),
    (
        "vif_unwrap_comp",
        r#"<template v-if="ok"><Foo /></template>"#,
    ),
    ("empty_tpl_vif", r#"<template v-if="ok"></template>"#),
    (
        "vif_for",
        r#"<template v-if="ok"><div v-for="i in n">{{ i }}</div></template>"#,
    ),
    (
        "vfor_two",
        r#"<template v-for="item in list"><span></span><span></span></template>"#,
    ),
    (
        "vfor_keyed",
        r#"<template v-for="item in list" :key="item"><span></span><span></span></template>"#,
    ),
    (
        "vfor_one",
        r#"<template v-for="item in list"><span></span></template>"#,
    ),
    (
        "vfor_one_keyed",
        r#"<template v-for="item in list" :key="item"><span>{{ item }}</span></template>"#,
    ),
    (
        "vfor_numeric",
        r#"<template v-for="n in 3"><span></span><span></span></template>"#,
    ),
    (
        "vfor_interp",
        r#"<template v-for="item in list">{{ item }}</template>"#,
    ),
    (
        "vfor_unwrap_dyn",
        r#"<template v-for="item in list"><span>{{ item }}</span></template>"#,
    ),
];

fn shipped(src: &str) -> String {
    let allocator = Allocator::new();
    let (_, errors, result) = compile_template(&allocator, src);
    assert!(errors.is_empty(), "shipped lane errors: {errors:?}");
    format!("{}\n{}", result.preamble, result.code)
}

#[test]
fn s2_template_wrapper_fragments_match_the_shipped_dom_lane_byte_for_byte() {
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
