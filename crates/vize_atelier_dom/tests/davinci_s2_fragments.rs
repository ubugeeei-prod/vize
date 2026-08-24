//! P2-11 template-fragment witness: empty roots, unique text,
//! multi-root `_Fragment`, compound interpolations, compared
//! **byte-for-byte** including helper usage.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

use vize_atelier_dom::compile_template;
use vize_carton::Allocator;
use vize_ricalco::{emit_dom_source, DOM_LANE_FLAG};

const BATTERY: &[(&str, &str)] = &[
    ("empty", ""),
    ("ws_only", "   \n"),
    ("plain_text", "hello"),
    ("plain_text_space", "hello world"),
    ("two_divs", "<div></div><span></span>"),
    ("two_divs_nl", "<div></div>\n<span></span>"),
    ("two_divs_space", "<div></div> <span></span>"),
    ("class_then_span", r#"<div class="x"></div><span></span>"#),
    ("two_class", r#"<div class="x"></div><span id="y"></span>"#),
    (
        "three_class",
        r#"<div class="a"></div><span class="b"></span><p class="c"></p>"#,
    ),
    ("two_static_text", "<div>a</div><span>b</span>"),
    ("text_then_span", "hello<span></span>"),
    ("span_then_text", "<span></span>hello"),
    ("interp_then_div", "{{ msg }}<div></div>"),
    ("div_then_interp", "<div></div>{{ msg }}"),
    ("two_interps", "{{ a }}{{ b }}"),
    ("text_interp", "hello {{ msg }}"),
    ("hello_interp_tight", "hello{{ msg }}"),
    ("text_nl_span", "hello\n<span></span>"),
    ("two_comps", "<Foo /><Bar />"),
    ("comp_then_span", "<Foo /><span></span>"),
    ("comp_id_text", r#"<Foo id="x">hello</Foo><span></span>"#),
    ("comp_empty_id", r#"<Foo id="x" /><span></span>"#),
    ("div_then_vif", r#"<div></div><p v-if="ok">x</p>"#),
    ("vif_then_div", r#"<p v-if="ok">x</p><div></div>"#),
    ("two_vif", r#"<p v-if="a">1</p><span v-if="b">2</span>"#),
    (
        "vif_else_then_div",
        r#"<div v-if="ok">a</div><div v-else>b</div><span></span>"#,
    ),
    (
        "div_then_vfor",
        r#"<div></div><p v-for="i in n">{{ i }}</p>"#,
    ),
    ("slot_then_div", "<slot></slot><div></div>"),
    (
        "teleport_then_div",
        r#"<Teleport to="body"><span></span></Teleport><div></div>"#,
    ),
    ("dynamic_then_div", r#"<component :is="x" /><div></div>"#),
    ("three_roots", "<div></div><span></span><p></p>"),
    ("nested_in_first", "<div><i></i></div><span></span>"),
    ("bind_then_span", r#"<div :id="x"></div><span></span>"#),
    ("comment_then_div", "<!-- hi --><div></div>"),
    ("div_then_comment", "<div></div><!-- hi -->"),
    ("only_comment", "<!-- hi -->"),
    (
        "class_interp_then_span",
        r#"<div class="x">{{ msg }}</div><span></span>"#,
    ),
    ("two_interp_elements", "<h1>{{ t }}</h1><p>{{ b }}</p>"),
    ("dup_comp", "<Foo /><Foo />"),
    ("named_slot_then_div", r#"<slot name="x"></slot><div></div>"#),
    (
        "keepalive_then_div",
        "<KeepAlive><Foo /></KeepAlive><div></div>",
    ),
    (
        "transition_then_div",
        r#"<Transition><div v-if="ok"></div></Transition><span></span>"#,
    ),
];

fn shipped(src: &str) -> String {
    let allocator = Allocator::new();
    let (_, errors, result) = compile_template(&allocator, src);
    assert!(errors.is_empty(), "shipped lane errors: {errors:?}");
    format!("{}\n{}", result.preamble, result.code)
}

#[test]
fn s2_template_fragments_match_the_shipped_dom_lane_byte_for_byte() {
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
