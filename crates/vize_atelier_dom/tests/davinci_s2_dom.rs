//! P2-11 installment 5 witness: static native HTML, interpolations,
//! mixed text siblings, static-name binds, static-name events including
//! event/key/option modifiers, native v-if, native v-for,
//! object-spread v-bind, static-name components, object v-on, and
//! implicit text / native / component default slots, compared
//! **byte-for-byte** including helper usage.
//!
//! `vize_atelier_dom` is published; the Davinci crates are not. The
//! comparator therefore rides stripped-on-publish dev-deps (the same
//! carve-out P2-9 used). The shipped `compile_template` path is
//! unchanged. `VIZE_DAVINCI_DOM=legacy` disarms the dual-run; the
//! pinned comparison count makes a silent disarm a loud failure.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

use vize_atelier_dom::compile_template;
use vize_carton::Allocator;
use vize_ricalco::{DOM_LANE_FLAG, emit_dom_source};

const BATTERY: &[(&str, &str)] = &[
    ("empty_div", "<div></div>"),
    ("div_with_text", "<div>hello</div>"),
    ("nested_elements", "<div><span>hello</span></div>"),
    ("paragraph", "<p>hi</p>"),
    ("sibling_spans", "<div><span>a</span><span>b</span></div>"),
    ("class_attr", r#"<div class="x"></div>"#),
    (
        "id_and_class",
        r#"<div id="app" class="container">static</div>"#,
    ),
    ("data_attr", r#"<div data-id="1"></div>"#),
    ("boolean_attr", "<div disabled></div>"),
    ("nested_class", r#"<div><span class="x">hello</span></div>"#),
    ("simple_interpolation", "{{ msg }}"),
    ("interpolation_in_element", "<div>{{ msg }}</div>"),
    ("mixed_text_interp", "<div>hello {{ msg }}</div>"),
    ("hoisted_class_interp", r#"<div class="x">{{ msg }}</div>"#),
    ("nested_interp", "<div><span>{{ msg }}</span></div>"),
    ("compound_two_dyn", "<p>Hi {{ name }}!</p>"),
    ("multiline_root_compound", "<div>Hi {{ name }}</div>\n"),
    ("interp_then_span", "<div>{{ msg }}<span></span></div>"),
    ("text_then_span", "<div>hello<span></span></div>"),
    (
        "space_between_spans",
        "<div><span></span> <span></span></div>",
    ),
    ("dynamic_class", r#"<div :class="cls"></div>"#),
    ("dynamic_id", r#"<div :id="foo"></div>"#),
    ("dynamic_style", r#"<div :style="s"></div>"#),
    ("class_and_interp", r#"<div :class="cls">{{ msg }}</div>"#),
    (
        "static_and_dynamic_class",
        r#"<div class="base" :class="cls"></div>"#,
    ),
    ("hyphenated_bind", r#"<div :data-id="x"></div>"#),
    ("click_handler", r#"<div @click="handler"></div>"#),
    ("keyup_handler", r#"<div @keyup="handler"></div>"#),
    ("hyphenated_event", r#"<div @foo-bar="x"></div>"#),
    (
        "click_and_interp",
        r#"<div @click="handler">{{ msg }}</div>"#,
    ),
    ("inline_click", r#"<div @click="count++"></div>"#),
    ("click_stop", r#"<div @click.stop="handler"></div>"#),
    (
        "click_prevent_stop",
        r#"<div @click.prevent.stop="handler"></div>"#,
    ),
    ("click_capture", r#"<div @click.capture="handler"></div>"#),
    ("click_once", r#"<div @click.once="handler"></div>"#),
    ("click_passive", r#"<div @click.passive="handler"></div>"#),
    ("click_right", r#"<div @click.right="handler"></div>"#),
    ("click_middle", r#"<div @click.middle="handler"></div>"#),
    ("click_left", r#"<div @click.left="handler"></div>"#),
    ("keyup_enter", r#"<div @keyup.enter="handler"></div>"#),
    ("keyup_left", r#"<div @keyup.left="handler"></div>"#),
    (
        "keyup_enter_stop",
        r#"<div @keyup.enter.stop="handler"></div>"#,
    ),
    ("inline_click_stop", r#"<div @click.stop="count++"></div>"#),
    ("bare_submit_prevent", r#"<div @submit.prevent></div>"#),
    (
        "click_once_capture",
        r#"<div @click.once.capture="handler"></div>"#,
    ),
    ("unknown_click_key", r#"<div @click.foo="handler"></div>"#),
    ("simple_v_if", r#"<div v-if="ok">hello</div>"#),
    (
        "v_if_else",
        r#"<div v-if="ok">yes</div><div v-else>no</div>"#,
    ),
    ("nested_v_if", r#"<div><p v-if="ok">x</p></div>"#),
    ("v_if_class", r#"<div v-if="ok" class="x"></div>"#),
    ("v_if_static_key", r#"<div v-if="ok" key="k"></div>"#),
    ("v_if_dyn_key", r#"<div v-if="ok" :key="k"></div>"#),
    (
        "v_if_dyn_key_expr",
        r#"<div v-if="ok" :key="item.id">x</div>"#,
    ),
    ("v_if_dyn_key_same", r#"<div v-if="ok" :key></div>"#),
    ("component_v_if_dyn_key", r#"<Foo v-if="ok" :key="k" />"#),
    (
        "v_if_dyn_key_chain",
        r#"<div v-if="a" :key="ka">a</div><div v-else-if="b" :key="kb">b</div><div v-else :key="kc">c</div>"#,
    ),
    (
        "tpl_v_if_dyn_key",
        r#"<template v-if="ok" :key="k"><span>x</span></template>"#,
    ),
    (
        "sibling_v_if",
        r#"<div><p v-if="a">1</p><span v-if="b">2</span></div>"#,
    ),
    (
        "v_if_elseif",
        r#"<div v-if="a">1</div><div v-else-if="b">2</div><div v-else>3</div>"#,
    ),
    (
        "keyed_v_for",
        r#"<div v-for="item in list" :key="item">{{ item }}</div>"#,
    ),
    (
        "unkeyed_v_for",
        r#"<div v-for="item in list">{{ item }}</div>"#,
    ),
    ("numeric_v_for", r#"<div v-for="n in 3">{{ n }}</div>"#),
    (
        "static_v_for_item_hoists",
        r#"<div><span v-for="i in n">x</span></div>"#,
    ),
    (
        "v_for_index",
        r#"<div v-for="(item, i) in list" :key="i">{{ item }}</div>"#,
    ),
    ("object_bind", r#"<div v-bind="obj"></div>"#),
    (
        "attr_then_object_bind",
        r#"<div id="x" v-bind="obj"></div>"#,
    ),
    (
        "object_bind_then_attr",
        r#"<div v-bind="obj" id="x"></div>"#,
    ),
    (
        "named_bind_then_object",
        r#"<div :id="foo" v-bind="obj"></div>"#,
    ),
    (
        "class_then_object_bind",
        r#"<div class="a" v-bind="obj"></div>"#,
    ),
    (
        "static_dynamic_class_then_object",
        r#"<div class="a" :class="cls" v-bind="obj"></div>"#,
    ),
    (
        "click_then_object_bind",
        r#"<div @click="h" v-bind="obj"></div>"#,
    ),
    (
        "keyup_then_object_bind",
        r#"<div @keyup="h" v-bind="obj"></div>"#,
    ),
    ("v_if_object_bind", r#"<div v-if="ok" v-bind="obj">x</div>"#),
    (
        "class_object_then_dynamic_class",
        r#"<div class="a" v-bind="obj" :class="cls"></div>"#,
    ),
    ("empty_component", "<Foo />"),
    ("nested_component", "<div><Foo /></div>"),
    ("kebab_component", "<foo-bar />"),
    ("component_static_class", r#"<Foo class="x" />"#),
    ("component_bind_id", r#"<Foo :id="x" />"#),
    ("component_bind_class", r#"<Foo :class="cls" />"#),
    ("component_click", r#"<Foo @click="h" />"#),
    ("component_keyup", r#"<Foo @keyup="h" />"#),
    ("component_v_if", r#"<Foo v-if="ok" />"#),
    (
        "component_v_for",
        r#"<Foo v-for="item in list" :key="item" />"#,
    ),
    ("component_v_for_unkeyed", r#"<Foo v-for="i in n" />"#),
    ("component_siblings", "<div><Foo /><Bar /></div>"),
    ("component_duplicate", "<div><Foo /><Foo /></div>"),
    ("component_then_span", "<div><Foo /><span></span></div>"),
    ("nested_component_v_if", r#"<div><Foo v-if="ok" /></div>"#),
    ("component_class_and_id", r#"<Foo class="x" :id="y" />"#),
    ("component_click_stop", r#"<Foo @click.stop="h" />"#),
    ("component_object_bind", r#"<Foo v-bind="obj" />"#),
    (
        "component_attr_then_object",
        r#"<Foo id="x" v-bind="obj" />"#,
    ),
    ("object_on", r#"<div v-on="handlers"></div>"#),
    (
        "attr_then_object_on",
        r#"<div id="x" v-on="handlers"></div>"#,
    ),
    (
        "object_on_then_attr",
        r#"<div v-on="handlers" id="x"></div>"#,
    ),
    (
        "click_then_object_on",
        r#"<div @click="h" v-on="handlers"></div>"#,
    ),
    (
        "object_bind_then_object_on",
        r#"<div v-bind="obj" v-on="handlers"></div>"#,
    ),
    (
        "v_if_object_on",
        r#"<div v-if="ok" v-on="handlers">x</div>"#,
    ),
    ("component_object_on", r#"<Foo v-on="handlers" />"#),
    (
        "component_attr_then_object_on",
        r#"<Foo id="x" v-on="handlers" />"#,
    ),
    ("component_text_slot", "<Foo>hello</Foo>"),
    ("component_interp_slot", "<Foo>{{ msg }}</Foo>"),
    ("component_mixed_text_slot", "<Foo>hello {{ msg }}</Foo>"),
    (
        "component_three_text_parts",
        "<Foo>hello{{ msg }}world</Foo>",
    ),
    ("nested_component_text_slot", "<div><Foo>hello</Foo></div>"),
    ("component_text_slot_v_if", r#"<Foo v-if="ok">hello</Foo>"#),
    (
        "component_text_slot_v_for",
        r#"<Foo v-for="item in list">hello</Foo>"#,
    ),
    (
        "component_text_slot_keyed_v_for",
        r#"<Foo v-for="item in list" :key="item">hello</Foo>"#,
    ),
    (
        "nested_v_for_component_text_slot",
        r#"<div v-for="i in n"><Foo>hello</Foo></div>"#,
    ),
    ("component_bind_text_slot", r#"<Foo :id="x">hello</Foo>"#),
    (
        "component_static_id_text_slot",
        r#"<Foo id="x">hello</Foo>"#,
    ),
    (
        "component_static_class_text_slot",
        r#"<Foo class="x">hello</Foo>"#,
    ),
    (
        "component_static_two_attrs_text_slot",
        r#"<Foo class="x" id="y">hello</Foo>"#,
    ),
    (
        "component_mixed_static_bind_text_slot",
        r#"<Foo class="x" :id="z">hello</Foo>"#,
    ),
    ("component_ws_only_children", "<Foo>  </Foo>"),
    ("component_padded_text_slot", "<Foo> hello </Foo>"),
    ("component_empty_span_slot", "<Foo><span></span></Foo>"),
    ("component_span_text_slot", "<Foo><span>hi</span></Foo>"),
    (
        "component_span_class_slot",
        r#"<Foo><span class="x"></span></Foo>"#,
    ),
    (
        "component_span_class_text_slot",
        r#"<Foo><span class="x">hi</span></Foo>"#,
    ),
    (
        "component_nested_static_slot",
        "<Foo><div><span></span></div></Foo>",
    ),
    (
        "component_two_static_spans",
        "<Foo><span></span><span></span></Foo>",
    ),
    (
        "component_text_then_span_slot",
        "<Foo>hello<span></span></Foo>",
    ),
    (
        "component_interp_in_span_slot",
        "<Foo><span>{{ msg }}</span></Foo>",
    ),
    (
        "component_dynamic_span_slot",
        r#"<Foo><span :id="x"></span></Foo>"#,
    ),
    (
        "component_class_then_span_slot",
        r#"<Foo class="x"><span></span></Foo>"#,
    ),
    ("component_nested_bar_slot", "<Foo><Bar /></Foo>"),
    ("component_nested_bar_text_slot", "<Foo><Bar>x</Bar></Foo>"),
    (
        "component_span_then_bar_slot",
        "<Foo><span></span><Bar /></Foo>",
    ),
    (
        "component_vif_span_slot",
        r#"<Foo><span v-if="ok">x</span></Foo>"#,
    ),
    (
        "component_compound_p_slot",
        r#"<Foo><p>Hi {{ name }}!</p></Foo>"#,
    ),
    (
        "nested_div_component_span_slot",
        "<div><Foo><span></span></Foo></div>",
    ),
    (
        "component_static_tree_with_text",
        r#"<Foo><div class="x"><span>hi</span></div></Foo>"#,
    ),
    (
        "component_mixed_text_element_hoist",
        "<Foo><div>hello<span></span></div></Foo>",
    ),
    (
        "component_vfor_span_slot",
        r#"<Foo><span v-for="i in n">x</span></Foo>"#,
    ),
    (
        "component_vfor_item_span_slot",
        r#"<Foo v-for="i in n"><span></span></Foo>"#,
    ),
    ("component_ws_then_span_slot", "<Foo> <span></span></Foo>"),
    (
        "component_nested_bar_vfor_slot",
        r#"<Foo><Bar v-for="i in n" /></Foo>"#,
    ),
];

fn shipped(src: &str) -> String {
    let allocator = Allocator::new();
    let (_, errors, result) = compile_template(&allocator, src);
    assert!(errors.is_empty(), "shipped lane errors: {errors:?}");
    format!("{}\n{}", result.preamble, result.code)
}

#[test]
fn s2_native_html_and_interpolations_match_the_shipped_dom_lane_byte_for_byte() {
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

#[test]
fn the_dom_lane_flag_has_its_recorded_name() {
    assert_eq!(DOM_LANE_FLAG, "VIZE_DAVINCI_DOM");
}
