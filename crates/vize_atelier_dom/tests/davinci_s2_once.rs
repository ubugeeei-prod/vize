//! P2-11 `v-once` witness: native element cache wrappers from S2,
//! compared **byte-for-byte** against the shipped DOM lane.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

mod support;

use vize_s0::Allocator;
use vize_s1_to_s2::emit_dom_source;

struct Case {
    name: &'static str,
    src: &'static str,
    sites: &'static [&'static str],
}

const CASES: &[Case] = &[
    Case {
        name: "simple",
        src: r#"<div v-once>x</div>"#,
        sites: &[],
    },
    Case {
        name: "interpolation",
        src: r#"<div v-once>{{ msg }}</div>"#,
        sites: &["1 /* TEXT */"],
    },
    Case {
        name: "dynamic_class",
        src: r#"<div v-once :class="cls">content</div>"#,
        sites: &["2 /* CLASS */"],
    },
    Case {
        name: "dynamic_style",
        src: r#"<div v-once :style="style">content</div>"#,
        sites: &["4 /* STYLE */"],
    },
    Case {
        name: "nested_static",
        src: r#"<div v-once><span>x</span></div>"#,
        sites: &[],
    },
    Case {
        name: "nested_dynamic",
        src: r#"<div v-once><span :class="cls">{{ msg }}</span></div>"#,
        sites: &["3 /* TEXT, CLASS */"],
    },
    Case {
        name: "v_if_branch",
        src: r#"<div v-if="ok" v-once>x</div>"#,
        sites: &[],
    },
    Case {
        name: "component",
        src: r#"<Foo v-once />"#,
        sites: &[],
    },
    Case {
        name: "component_with_props_and_slot",
        src: r#"<Foo v-once :title="title"><span>x</span></Foo>"#,
        sites: &[],
    },
    Case {
        name: "inside_v_for",
        src: r#"<div v-for="item in items" :key="item.id"><span v-once>{{ item.static }}</span></div>"#,
        sites: &["1 /* TEXT */", "128 /* KEYED_FRAGMENT */"],
    },
];

#[test]
fn s2_v_once_matches_the_shipped_dom_lane_byte_for_byte() {
    let battery: Vec<_> = CASES.iter().map(|case| (case.name, case.src)).collect();
    support::assert_s2_matches_shipped(&battery);
}

#[test]
fn s2_v_once_patch_flags_match_the_shipped_dom_lane_per_node() {
    let mut mismatches = Vec::new();
    for case in CASES {
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
        "v-once patch flag mismatches:\n{}",
        mismatches.join("\n")
    );
}
