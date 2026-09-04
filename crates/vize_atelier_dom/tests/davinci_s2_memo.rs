//! P2-11 `v-memo` witness: cache wrappers and `v-for` cached-item
//! guards, compared **byte-for-byte** including helper usage.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

mod support;

use vize_s0::Allocator;
use vize_s1_to_s2::emit_dom_source;

const BATTERY: &[(&str, &str)] = &[
    ("native_text", r#"<div v-memo="[id]">x</div>"#),
    (
        "native_interpolation",
        r#"<div v-memo="[id]">{{ msg }}</div>"#,
    ),
    (
        "native_dynamic_prop",
        r#"<div v-memo="[id]" :id="id">{{ msg }}</div>"#,
    ),
    (
        "nested_native",
        r#"<div><span v-memo="[id]">{{ msg }}</span></div>"#,
    ),
    ("component_root", r#"<Foo v-memo="[prop]" :prop="prop" />"#),
    (
        "nested_component",
        r#"<div><Foo v-memo="[prop]" :prop="prop" /></div>"#,
    ),
    ("native_v_if", r#"<div v-if="ok" v-memo="[id]">x</div>"#),
    (
        "component_v_if",
        r#"<Foo v-if="ok" v-memo="[prop]" :prop="prop" />"#,
    ),
    (
        "native_v_for",
        r#"<div v-for="item in items" :key="item.id" v-memo="[item.selected]">{{ item.name }}</div>"#,
    ),
    (
        "native_v_for_static_key",
        r#"<div v-for="item in items" key="row" v-memo="[item.selected]">{{ item.name }}</div>"#,
    ),
    (
        "component_v_for",
        r#"<Foo v-for="item in items" :key="item.id" v-memo="[item.selected]" :prop="item.prop" />"#,
    ),
    (
        "unkeyed_v_for",
        r#"<div v-for="item in items" v-memo="[item.selected]">{{ item.name }}</div>"#,
    ),
    (
        "numeric_v_for",
        r#"<div v-for="n in 3" v-memo="[n]">{{ n }}</div>"#,
    ),
];

struct PatchCase {
    name: &'static str,
    src: &'static str,
    sites: &'static [&'static str],
}

const PATCH_CASES: &[PatchCase] = &[
    PatchCase {
        name: "native_text",
        src: r#"<div v-memo="[id]">x</div>"#,
        sites: &[],
    },
    PatchCase {
        name: "native_interpolation",
        src: r#"<div v-memo="[id]">{{ msg }}</div>"#,
        sites: &["1 /* TEXT */"],
    },
    PatchCase {
        name: "native_dynamic_prop_keeps_array_only",
        src: r#"<div v-memo="[id]" :id="id">{{ msg }}</div>"#,
        sites: &["1 /* TEXT */"],
    },
    PatchCase {
        name: "component_root",
        src: r#"<Foo v-memo="[prop]" :prop="prop" />"#,
        sites: &["8 /* PROPS */, [\"prop\"]"],
    },
    PatchCase {
        name: "native_v_for",
        src: r#"<div v-for="item in items" :key="item.id" v-memo="[item.selected]">{{ item.name }}</div>"#,
        sites: &["1 /* TEXT */", "128 /* KEYED_FRAGMENT */"],
    },
    PatchCase {
        name: "native_v_for_static_key",
        src: r#"<div v-for="item in items" key="row" v-memo="[item.selected]">{{ item.name }}</div>"#,
        sites: &["1 /* TEXT */", "256 /* UNKEYED_FRAGMENT */"],
    },
    PatchCase {
        name: "component_v_for",
        src: r#"<Foo v-for="item in items" :key="item.id" v-memo="[item.selected]" :prop="item.prop" />"#,
        sites: &["8 /* PROPS */, [\"prop\"]", "128 /* KEYED_FRAGMENT */"],
    },
    PatchCase {
        name: "unkeyed_v_for",
        src: r#"<div v-for="item in items" v-memo="[item.selected]">{{ item.name }}</div>"#,
        sites: &["1 /* TEXT */", "256 /* UNKEYED_FRAGMENT */"],
    },
];

#[test]
fn s2_v_memo_matches_the_shipped_dom_lane_byte_for_byte() {
    support::assert_s2_matches_shipped(BATTERY);
}

#[test]
fn s2_v_memo_patch_flags_match_the_shipped_dom_lane_per_node() {
    let battery: Vec<_> = PATCH_CASES
        .iter()
        .map(|case| (case.name, case.src))
        .collect();
    support::assert_s2_matches_shipped(&battery);

    let mut mismatches = Vec::new();
    for case in PATCH_CASES {
        let expected: Vec<_> = case.sites.iter().map(|site| site.to_string()).collect();
        let old = support::shipped(case.src);
        let allocator = Allocator::new();
        let new = emit_dom_source(&allocator, case.src)
            .unwrap_or_else(|error| panic!("{}: S2 emit refused: {error:?}", case.name))
            .assembled();
        let old_sites = support::patch_sites(&old);
        let new_sites = support::patch_sites(&new);

        if old_sites != expected || new_sites != expected {
            mismatches.push(format!(
                "{}: expected={expected:?} old={old_sites:?} new={new_sites:?}",
                case.name
            ));
        }
    }
    assert!(
        mismatches.is_empty(),
        "v-memo patch flag mismatches:\n{}",
        mismatches.join("\n")
    );
}
