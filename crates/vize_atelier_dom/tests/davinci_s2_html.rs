//! P2-11 `v-html` witness: `vue.html` lowers through S2 and emits the
//! shipped `innerHTML` DOM-prop shape, including patch-flag composition.

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
        src: r#"<div v-html="raw"></div>"#,
        sites: &["8 /* PROPS */, [\"innerHTML\"]"],
    },
    Case {
        name: "native_static_child",
        src: r#"<div v-html="raw">fallback</div>"#,
        sites: &["8 /* PROPS */, [\"innerHTML\"]"],
    },
    Case {
        name: "native_interp_child",
        src: r#"<div v-html="raw">{{ msg }}</div>"#,
        sites: &["9 /* TEXT, PROPS */, [\"innerHTML\"]"],
    },
    Case {
        name: "multiline_value_keeps_authored_padding",
        src: r#"<div v-html="
  formatHtml(
    value,
  )
"></div>"#,
        sites: &["8 /* PROPS */, [\"innerHTML\"]"],
    },
    Case {
        name: "value_decodes_attribute_entities",
        src: r#"<em class="weui-form-preview__value" v-html="headerValue || '&nbsp;'"></em>"#,
        sites: &["8 /* PROPS */, [\"innerHTML\"]"],
    },
    Case {
        name: "value_less_bare",
        src: r#"<div v-html></div>"#,
        sites: &["8 /* PROPS */, [\"innerHTML\"]"],
    },
    Case {
        name: "value_less_empty",
        src: r#"<div v-html=""></div>"#,
        sites: &["8 /* PROPS */, [\"innerHTML\"]"],
    },
    Case {
        name: "bind_id",
        src: r#"<div v-html="raw" :id="id"></div>"#,
        sites: &["8 /* PROPS */, [\"innerHTML\", \"id\"]"],
    },
    Case {
        name: "style_bind",
        src: r#"<div v-html="raw" :style="style"></div>"#,
        sites: &["12 /* STYLE, PROPS */, [\"innerHTML\"]"],
    },
    Case {
        name: "object_bind",
        src: r#"<div v-html="raw" v-bind="attrs"></div>"#,
        sites: &["16 /* FULL_PROPS */, [\"innerHTML\"]"],
    },
    Case {
        name: "v_if",
        src: r#"<div v-if="ok" v-html="raw"></div>"#,
        sites: &["8 /* PROPS */, [\"innerHTML\"]"],
    },
    Case {
        name: "v_for",
        src: r#"<div v-for="item in items" v-html="item.html"></div>"#,
        sites: &[
            "8 /* PROPS */, [\"innerHTML\"]",
            "256 /* UNKEYED_FRAGMENT */",
        ],
    },
    Case {
        name: "root_component",
        src: r#"<MyComponent v-html="raw" />"#,
        sites: &["8 /* PROPS */, [\"innerHTML\"]"],
    },
    Case {
        name: "custom_and_html",
        src: r#"<div v-html="raw" v-example></div>"#,
        sites: &["8 /* PROPS */, [\"innerHTML\"]"],
    },
    Case {
        name: "native_model_and_html",
        src: r#"<input v-model="text" v-html="raw">"#,
        sites: &["8 /* PROPS */, [\"onUpdate:modelValue\", \"innerHTML\"]"],
    },
    Case {
        name: "slot_outlet",
        src: r#"<slot v-html="raw"></slot>"#,
        sites: &[],
    },
];

#[test]
fn s2_v_html_matches_the_shipped_dom_lane_byte_for_byte() {
    let battery: Vec<_> = CASES.iter().map(|case| (case.name, case.src)).collect();
    support::assert_s2_matches_shipped(&battery);
}

#[test]
fn s2_v_html_patch_flags_match_the_shipped_dom_lane_per_node() {
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
        "v-html patch flag mismatches:\n{}",
        mismatches.join("\n")
    );
}

#[test]
fn s2_pug_example_text_decodes_double_escaped_parentheses_like_shipped_lane() {
    support::assert_s2_matches_shipped(&[
        (
            "pug_example_text_double_escaped_parens",
            r#"example
  template(#pug).
    w-button(@click="accordion = Array&amp;#40;3&amp;#41;.fill&amp;#40;true&amp;#41;" sm) Expand all"#,
        ),
        (
            "pug_example_text_zero_padded_double_escaped_close_paren",
            r#"example
  template(#html).
    &lt;w-button @click="overlayColor = 'rgba(35, 71, 129, 0.5&amp;#041;'"&gt;Color&lt;/w-button&gt;"#,
        ),
    ]);
}
