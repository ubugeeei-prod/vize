//! The Elk NavUser nested template branch is a Fragment, not an attrs owner.
#![cfg(feature = "legacy-differential")]

use vize_atelier_ssr::{SsrCompilerExperimentalOptions, SsrCompilerOptions};

const CASES: &[(&str, &str, bool)] = &[
    (
        "nested-root",
        r#"<template v-if="outer"><button v-if="inner">a</button><button v-else>b</button></template>"#,
        true,
    ),
    (
        "nested-else",
        r#"<i v-if="outer">outer</i><template v-else><button v-if="inner">a</button><button v-else>b</button></template>"#,
        true,
    ),
    (
        "single-template-root",
        r#"<template v-if="outer"><button>b</button></template>"#,
        false,
    ),
    (
        "plain-root-chain",
        r#"<i v-if="outer">outer</i><button v-else>b</button>"#,
        false,
    ),
    (
        "loop-owns-fragment",
        r#"<template v-if="outer"><button v-for="n in 2">{{ n }}</button></template>"#,
        false,
    ),
    (
        "loop-conditional-boundary",
        r#"<template v-for="n in 2"><template v-if="outer"><button v-if="inner">a</button><button v-else>b</button></template></template>"#,
        false,
    ),
    (
        "loop-single-conditional",
        r#"<template v-for="n in 2"><button v-if="inner">a</button><button v-else>b</button></template>"#,
        false,
    ),
    (
        "loop-nested-loop",
        r#"<template v-for="n in 2"><button v-for="m in 2">{{ n }}:{{ m }}</button></template>"#,
        false,
    ),
    (
        "loop-single-element",
        r#"<button v-for="n in 2">{{ n }}</button>"#,
        false,
    ),
    (
        "loop-single-text",
        r#"<template v-for="n in 2">text</template>"#,
        false,
    ),
    (
        "loop-empty",
        r#"<template v-for="n in 2"></template>"#,
        false,
    ),
    (
        "transition-group-loop-conditional",
        r#"<TransitionGroup tag="section"><template v-for="n in 2"><template v-if="outer"><button v-if="inner">a</button><button v-else>b</button></template></template></TransitionGroup>"#,
        false,
    ),
    (
        "loop-keyed-element",
        r#"<template v-for="n in 2" :key="n"><button>{{ n }}</button></template>"#,
        false,
    ),
    (
        "loop-keyed-conditional",
        r#"<template v-for="n in 2" :key="n"><button v-if="inner">a</button><button v-else>b</button></template>"#,
        false,
    ),
    (
        "loop-keyed-component",
        r#"<template v-for="n in 2" :key="n"><RuntimeChild :n="n" /></template>"#,
        false,
    ),
    (
        "loop-keyed-slot",
        r#"<template v-for="n in 2" :key="n"><slot :n="n"><button>fallback</button></slot></template>"#,
        false,
    ),
    (
        "fallback-keyed-component",
        r#"<RuntimeSlotHost><template v-for="n in 2" :key="n"><RuntimeChild :n="n" /></template></RuntimeSlotHost>"#,
        false,
    ),
    (
        "fallback-keyed-slot",
        r#"<RuntimeSlotHost><template v-for="n in 2" :key="n"><slot :n="n"><button>fallback</button></slot></template></RuntimeSlotHost>"#,
        false,
    ),
    (
        "fallback-conditional-boundary",
        r#"<RuntimeSlotHost><template v-for="n in 2"><template v-if="outer"><button v-if="inner">a</button><button v-else>b</button></template></template></RuntimeSlotHost>"#,
        false,
    ),
    (
        "fallback-nested-loop",
        r#"<RuntimeSlotHost><template v-for="n in 2"><button v-for="m in 2">{{ n }}:{{ m }}</button></template></RuntimeSlotHost>"#,
        false,
    ),
];

#[test]
fn nested_template_conditionals_keep_fragments_and_both_emission_routes() {
    let mut captures = Vec::new();
    for vapor in [false, true] {
        for &(name, source, nested) in CASES {
            let pair = vize_atelier_ssr::differential::compare_ssr_lanes(
                source,
                &SsrCompilerOptions {
                    vapor,
                    ..Default::default()
                },
                &SsrCompilerExperimentalOptions {
                    source_map: true,
                    source_map_filename: Some("NestedBranch.vue".into()),
                    ..Default::default()
                },
            );
            assert_eq!(pair.lane, "s4", "native plan must own {name}");
            assert!(pair.selected.errors.is_empty(), "{name}");
            assert!(pair.legacy.errors.is_empty(), "{name}");
            assert_eq!(pair.selected.result.code, pair.legacy.result.code, "{name}");
            assert_eq!(
                pair.selected.result.preamble, pair.legacy.result.preamble,
                "{name}"
            );
            assert_eq!(pair.selected.result.map, pair.legacy.result.map, "{name}");
            if nested {
                assert!(
                    pair.selected.result.code.contains("_push(`<!--[-->`)"),
                    "nested conditional needs its own fragment: {name}"
                );
                assert!(
                    !pair.selected.result.code.contains("<button${"),
                    "parent attrs cannot cross the fragment into its button: {name}"
                );
            }
            captures.push(serde_json::json!({
                "name": name,
                "source": source,
                "vapor": vapor,
                "lane": pair.lane,
                "selected": {
                    "code": pair.selected.result.code,
                    "preamble": pair.selected.result.preamble,
                    "map": pair.selected.result.map,
                },
                "legacy": {
                    "code": pair.legacy.result.code,
                    "preamble": pair.legacy.result.preamble,
                    "map": pair.legacy.result.map,
                },
            }));
        }
    }
    if let Some(destination) = std::env::var_os("VIZE_NESTED_BRANCH_CAPTURE") {
        std::fs::write(
            destination,
            serde_json::to_vec_pretty(&captures).expect("serialize complete native modules"),
        )
        .expect("retain native modules for actual Vue execution");
    }
}
