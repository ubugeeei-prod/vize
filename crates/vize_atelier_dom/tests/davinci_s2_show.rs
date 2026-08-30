//! P2-11 `v-show` witness: `vue.show` lowers through S2 and emits the
//! shipped `_withDirectives(..., [[_vShow, expr]])` DOM shape, including
//! the `NEED_PATCH` patch flag.

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
        name: "native_text",
        src: r#"<div v-show="visible">content</div>"#,
        sites: &["512 /* NEED_PATCH */"],
    },
    Case {
        name: "native_interp",
        src: r#"<div v-show="visible">{{ msg }}</div>"#,
        sites: &["513 /* TEXT, NEED_PATCH */"],
    },
    Case {
        name: "static_child",
        src: r#"<div v-show="visible"><span id="">content</span></div>"#,
        sites: &["512 /* NEED_PATCH */"],
    },
    Case {
        name: "bind_id",
        src: r#"<div v-show="visible" :id="id">content</div>"#,
        sites: &["8 /* PROPS */, [\"id\"]"],
    },
    Case {
        name: "v_if",
        src: r#"<div v-if="ok" v-show="visible">content</div>"#,
        sites: &["512 /* NEED_PATCH */"],
    },
    Case {
        name: "v_for",
        src: r#"<div v-for="item in items" v-show="item.visible">{{ item.label }}</div>"#,
        sites: &["1 /* TEXT */", "256 /* UNKEYED_FRAGMENT */"],
    },
    Case {
        name: "root_component",
        src: r#"<MyComponent v-show="visible" />"#,
        sites: &["512 /* NEED_PATCH */"],
    },
    Case {
        name: "child_component",
        src: r#"<div><MyComponent v-show="visible" /></div>"#,
        sites: &["512 /* NEED_PATCH */"],
    },
    Case {
        name: "custom_and_show",
        src: r#"<div v-show="visible" v-example></div>"#,
        sites: &["512 /* NEED_PATCH */"],
    },
    Case {
        name: "native_model_and_show",
        src: r#"<input v-model="text" v-show="visible">"#,
        sites: &["8 /* PROPS */, [\"onUpdate:modelValue\"]"],
    },
];

#[test]
fn s2_v_show_matches_the_shipped_dom_lane_byte_for_byte() {
    let battery: Vec<_> = CASES.iter().map(|case| (case.name, case.src)).collect();
    support::assert_s2_matches_shipped(&battery);
}

#[test]
fn s2_v_show_patch_flags_match_the_shipped_dom_lane_per_node() {
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
        "v-show patch flag mismatches:\n{}",
        mismatches.join("\n")
    );
}
