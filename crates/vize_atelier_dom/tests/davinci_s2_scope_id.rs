//! P2-11 installment 91 witness: **`scope_id`**. `<style scoped>` gives the
//! SFC an attribute name that every element's props object carries as a
//! trailing `"data-v-abc123": ""` pair. Compared byte-for-byte with the
//! shipped lane.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

mod support;

use vize_atelier_core::options::CodegenOptions;
use vize_atelier_dom::DomCompilerOptions;
use vize_s1_to_s2::DomEmitOptions;

const SCOPE_ID: &str = "data-v-abc123";

const BATTERY: &[(&str, &str)] = &[
    ("empty_element", "<div></div>"),
    ("text_only", "<div>hello</div>"),
    ("static_attr", r#"<div class="a"></div>"#),
    ("two_static_attrs", r#"<div id="x" class="a"></div>"#),
    ("dynamic_bind", r#"<div :id="x"></div>"#),
    ("static_and_dynamic", r#"<div class="a" :id="x"></div>"#),
    ("interpolation", "<div>{{ msg }}</div>"),
    ("handler", r#"<div @click="go"></div>"#),
    ("nested", "<div><span>hi</span></div>"),
    ("dynamic_class", r#"<div :class="cls"></div>"#),
    (
        "static_class_and_style",
        r#"<div class="a" style="color:red"></div>"#,
    ),
    ("dynamic_style", r#"<div :style="s"></div>"#),
    ("multi_root", "<div></div><span></span>"),
    ("deep_static", "<div><p><em>x</em></p></div>"),
    ("static_siblings", "<div><i>a</i><b>b</b></div>"),
    ("v_if", r#"<div v-if="ok">x</div>"#),
    ("v_if_else", r#"<div v-if="ok">x</div><div v-else>y</div>"#),
    ("v_for", r#"<li v-for="i in items" :key="i">{{ i }}</li>"#),
    (
        "v_for_static_child",
        r#"<li v-for="i in items" :key="i"><b>x</b></li>"#,
    ),
    ("component", "<MyComp />"),
    ("component_with_prop", r#"<MyComp :a="x" />"#),
    ("component_static_prop", r#"<MyComp a="1" />"#),
    ("component_with_slot", "<MyComp><span>x</span></MyComp>"),
    ("slot_outlet", "<slot />"),
    ("template_root", "<template><div>x</div></template>"),
    ("svg", r#"<svg><path d="M0 0"/></svg>"#),
    ("v_once", "<div v-once>x</div>"),
    ("v_html", r#"<div v-html="raw"></div>"#),
    ("v_show", r#"<div v-show="ok"></div>"#),
    ("v_model", r#"<input v-model="v">"#),
    ("ref_attr", r#"<div ref="el"></div>"#),
    ("spread_bind", r#"<div v-bind="obj"></div>"#),
    ("spread_and_static", r#"<div class="a" v-bind="obj"></div>"#),
    ("spread_on", r#"<div v-on="handlers"></div>"#),
    ("dynamic_bind_key", r#"<div :[k]="v"></div>"#),
    ("teleport", r##"<Teleport to="#a"><div>x</div></Teleport>"##),
    ("keepalive", "<KeepAlive><MyComp /></KeepAlive>"),
];

fn dual_run(battery: &[(&str, &str)]) {
    support::assert_s2_matches_shipped_with_options(
        battery,
        &DomCompilerOptions {
            scope_id: Some(SCOPE_ID.into()),
            ..Default::default()
        },
        &CodegenOptions::default(),
        &DomEmitOptions {
            scope_id: Some(SCOPE_ID),
            ..DomEmitOptions::DEFAULT
        },
    );
}

#[test]
fn scope_id_matches_the_shipped_dom_lane() {
    dual_run(BATTERY);
}

/// The option must actually do something: a lane that ignored it would
/// still pass the dual runs above. Full outputs, not substrings.
#[test]
fn the_option_is_what_produces_the_scope_attribute() {
    let allocator = vize_s0::Allocator::new();
    let body = |src: &str, scope_id: Option<&str>| {
        vize_s1_to_s2::emit_dom_source_with_options(
            &allocator,
            src,
            vize_s1_to_s2::LegacyCaps::VUE3,
            &DomEmitOptions {
                scope_id,
                ..DomEmitOptions::DEFAULT
            },
        )
        .expect("scope-id witness must emit")
        .assembled()
    };
    const PREAMBLE: &str = "const { mergeProps: _mergeProps, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue\n\nfunction render(_ctx, _cache, $props, $setup, $data, $options) {\n";
    let render = |scope_id: Option<&str>| {
        let src = r#"<div class="a" v-bind="obj"></div>"#;
        let out = body(src, scope_id);
        out.strip_prefix(PREAMBLE)
            .unwrap_or_else(|| panic!("unexpected preamble in:\n{out}"))
            .to_string()
    };
    assert_eq!(
        (render(None), render(Some(SCOPE_ID))),
        (
            // Off: the spread merges only the authored props.
            "  return (_openBlock(), _createElementBlock(\"div\", _mergeProps({ class: \"a\" }, obj), null, 16 /* FULL_PROPS */))\n}"
                .to_string(),
            // On: the scope pair rides one trailing `mergeProps` argument,
            // never each segment.
            "  return (_openBlock(), _createElementBlock(\"div\", _mergeProps({ class: \"a\" }, obj, { \"data-v-abc123\": \"\" }), null, 16 /* FULL_PROPS */))\n}"
                .to_string(),
        )
    );
}
