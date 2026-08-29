//! P2-11 `v-text` witness: `vue.text` lowers through S2 and emits the
//! shipped `textContent` DOM-prop shape, including display-string coercion.

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
        src: r#"<div v-text="raw"></div>"#,
        sites: &["8 /* PROPS */, [\"textContent\"]"],
    },
    Case {
        name: "native_static_child",
        src: r#"<div v-text="raw">fallback</div>"#,
        sites: &["8 /* PROPS */, [\"textContent\"]"],
    },
    Case {
        name: "native_interp_child",
        src: r#"<div v-text="raw">{{ msg }}</div>"#,
        sites: &["9 /* TEXT, PROPS */, [\"textContent\"]"],
    },
    Case {
        name: "value_less_bare",
        src: r#"<div v-text></div>"#,
        sites: &["8 /* PROPS */, [\"textContent\"]"],
    },
    Case {
        name: "value_less_empty",
        src: r#"<div v-text=""></div>"#,
        sites: &["8 /* PROPS */, [\"textContent\"]"],
    },
    Case {
        name: "bind_id",
        src: r#"<div v-text="raw" :id="id"></div>"#,
        sites: &["8 /* PROPS */, [\"textContent\", \"id\"]"],
    },
    Case {
        name: "style_bind",
        src: r#"<div v-text="raw" :style="style"></div>"#,
        sites: &["12 /* STYLE, PROPS */, [\"textContent\"]"],
    },
    Case {
        name: "object_bind",
        src: r#"<div v-text="raw" v-bind="attrs"></div>"#,
        sites: &["16 /* FULL_PROPS */, [\"textContent\"]"],
    },
    Case {
        name: "v_if",
        src: r#"<div v-if="ok" v-text="raw"></div>"#,
        sites: &["8 /* PROPS */, [\"textContent\"]"],
    },
    Case {
        name: "v_for",
        src: r#"<div v-for="item in items" v-text="item.label"></div>"#,
        sites: &[
            "8 /* PROPS */, [\"textContent\"]",
            "256 /* UNKEYED_FRAGMENT */",
        ],
    },
    Case {
        name: "root_component",
        src: r#"<MyComponent v-text="raw" />"#,
        sites: &["8 /* PROPS */, [\"textContent\"]"],
    },
    Case {
        name: "custom_and_text",
        src: r#"<div v-text="raw" v-example></div>"#,
        sites: &["8 /* PROPS */, [\"textContent\"]"],
    },
    Case {
        name: "native_model_and_text",
        src: r#"<input v-model="text" v-text="raw">"#,
        sites: &["8 /* PROPS */, [\"onUpdate:modelValue\", \"textContent\"]"],
    },
    Case {
        name: "slot_outlet",
        src: r#"<slot v-text="raw"></slot>"#,
        sites: &[],
    },
];

#[test]
fn s2_v_text_matches_the_shipped_dom_lane_byte_for_byte() {
    let battery: Vec<_> = CASES.iter().map(|case| (case.name, case.src)).collect();
    support::assert_s2_matches_shipped(&battery);
}

#[test]
fn s2_v_text_patch_flags_match_the_shipped_dom_lane_per_node() {
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
        "v-text patch flag mismatches:\n{}",
        mismatches.join("\n")
    );
}
