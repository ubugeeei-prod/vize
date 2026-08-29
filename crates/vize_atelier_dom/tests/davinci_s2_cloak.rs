//! P2-11 `v-cloak` witness: `vue.cloak` lowers through S2 and emits the
//! shipped no-op DOM shape, including prop and slot elision.

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
        name: "native_empty",
        src: r#"<div v-cloak></div>"#,
        sites: &[],
    },
    Case {
        name: "native_static_child",
        src: r#"<div v-cloak>fallback</div>"#,
        sites: &[],
    },
    Case {
        name: "native_interp_child",
        src: r#"<div v-cloak>{{ msg }}</div>"#,
        sites: &["1 /* TEXT */"],
    },
    Case {
        name: "bind_id",
        src: r#"<div v-cloak :id="id"></div>"#,
        sites: &["8 /* PROPS */, [\"id\"]"],
    },
    Case {
        name: "valued",
        src: r#"<div v-cloak="raw"></div>"#,
        sites: &[],
    },
    Case {
        name: "argument",
        src: r#"<div v-cloak:foo></div>"#,
        sites: &[],
    },
    Case {
        name: "modifier",
        src: r#"<div v-cloak.foo></div>"#,
        sites: &[],
    },
    Case {
        name: "dynamic_argument_value",
        src: r#"<div v-cloak:[foo]="raw"></div>"#,
        sites: &[],
    },
    Case {
        name: "style_bind",
        src: r#"<div v-cloak :style="style"></div>"#,
        sites: &["4 /* STYLE */"],
    },
    Case {
        name: "object_bind",
        src: r#"<div v-cloak v-bind="attrs"></div>"#,
        sites: &["16 /* FULL_PROPS */"],
    },
    Case {
        name: "v_if",
        src: r#"<div v-if="ok" v-cloak></div>"#,
        sites: &[],
    },
    Case {
        name: "v_for",
        src: r#"<div v-for="item in items" v-cloak>{{ item.label }}</div>"#,
        sites: &["1 /* TEXT */", "256 /* UNKEYED_FRAGMENT */"],
    },
    Case {
        name: "root_component",
        src: r#"<MyComponent v-cloak />"#,
        sites: &[],
    },
    Case {
        name: "custom_and_cloak",
        src: r#"<div v-cloak v-example></div>"#,
        sites: &["512 /* NEED_PATCH */"],
    },
    Case {
        name: "native_model_and_cloak",
        src: r#"<input v-model="text" v-cloak>"#,
        sites: &["8 /* PROPS */, [\"onUpdate:modelValue\"]"],
    },
    Case {
        name: "slot_outlet",
        src: r#"<slot v-cloak></slot>"#,
        sites: &[],
    },
    Case {
        name: "slot_outlet_with_fallback",
        src: r#"<slot v-cloak>fallback</slot>"#,
        sites: &[],
    },
];

#[test]
fn s2_v_cloak_matches_the_shipped_dom_lane_byte_for_byte() {
    let battery: Vec<_> = CASES.iter().map(|case| (case.name, case.src)).collect();
    support::assert_s2_matches_shipped(&battery);
}

#[test]
fn s2_v_cloak_patch_flags_match_the_shipped_dom_lane_per_node() {
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
        "v-cloak patch flag mismatches:\n{}",
        mismatches.join("\n")
    );
}
