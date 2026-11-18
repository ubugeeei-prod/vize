//! `v-pre` freezes the element and everything under it.
//!
//! Vue's parser sets `inVPre` the moment it reads the spelling on an
//! opening tag and rewrites **every** prop on that element back to a
//! plain attribute under its raw authored name, for the element itself
//! and its whole subtree. So `:x="1"` stays the attribute `":x"` with the
//! string value `"1"`, `@click` stays `"@click"`, `v-if` never builds a
//! branch, `v-for` never builds a region, and `{{ x }}` is literal text.
//! The spelling itself is dropped from the output.
//!
//! The L2 lowering carried only the last of those: interpolations went
//! inert, and everything else compiled as usual. Since P2-11 made the L2
//! lane the shipped one, `v-pre` was effectively unimplemented in
//! production — `<div v-pre :x="1">` shipped `{ x: 1 }` where Vue emits
//! `{ ":x": "1" }`, and `<div v-pre><li v-for="i in l">` shipped a real
//! `renderList`.
//!
//! No corpus template caught it: `davinci_l2_transform_corpus`'s sweep
//! counts **zero** `v-pre` templates across the hydrated corpus, which is
//! why this needs its own witness. Expectations are `@vue/compiler-dom`
//! 3.6.0-beta.10's own output, cross-checked against this crate's legacy
//! parse/transform lane.

#![expect(
    clippy::disallowed_types,
    reason = "insta and fixtures use format!; fixtures use std strings"
)]

use vize_atelier_core::options::{
    CodegenExperimentalOptions, CodegenOptions, CustomElementMatcher, TemplateSyntaxMode,
};
use vize_atelier_dom::{
    DomCompilerOptions, compile_template, compile_template_legacy_with_options,
    compile_template_with_custom_elements_template_syntax_codegen_and_experimental_options_with_stage_capture,
    compile_template_with_template_syntax,
};
use vize_l0::Allocator;
use vize_l0::dump::capture::{CaptureOutcome, StageCapture};

/// `(name, template)` — every directive kind, on the `v-pre` element
/// itself and one level under it.
const CASES: &[(&str, &str)] = &[
    ("self-bind", r#"<div><div v-pre :x="1">c</div></div>"#),
    (
        "self-bind-full",
        r#"<div><div v-pre v-bind:x="1">c</div></div>"#,
    ),
    ("self-on", r#"<div><div v-pre @click="go">c</div></div>"#),
    (
        "self-on-modified",
        r#"<div><div v-pre @click.stop="go">c</div></div>"#,
    ),
    ("self-show", r#"<div><div v-pre v-show="s">c</div></div>"#),
    ("self-html", r#"<div><div v-pre v-html="h">c</div></div>"#),
    ("self-model", r#"<div><input v-pre v-model="m"></div>"#),
    (
        "self-custom",
        r#"<div><div v-pre v-mine:arg.mod="v">c</div></div>"#,
    ),
    ("self-static", r#"<div><div v-pre id="a">c</div></div>"#),
    (
        "self-mixed",
        r#"<div><div v-pre id="a" :x="1" @y="z">c</div></div>"#,
    ),
    ("self-if", r#"<div><div v-pre v-if="a">c</div></div>"#),
    (
        "self-for",
        r#"<div><div v-pre v-for="i in l">c</div></div>"#,
    ),
    (
        "child-bind",
        r#"<div><div v-pre><span :x="1"></span></div></div>"#,
    ),
    (
        "child-interp",
        r#"<div><div v-pre><span>{{ x }}</span></div></div>"#,
    ),
    (
        "child-if",
        r#"<div><div v-pre><span v-if="a">y</span></div></div>"#,
    ),
    (
        "child-if-else",
        r#"<div><div v-pre><i v-if="a">y</i><i v-else>n</i></div></div>"#,
    ),
    (
        "child-for",
        r#"<div><div v-pre><li v-for="i in l">{{ i }}</li></div></div>"#,
    ),
    (
        "child-component",
        r#"<div><div v-pre><MyComp :x="1"></MyComp></div></div>"#,
    ),
    (
        "grandchild",
        r#"<div><div v-pre><span><b :x="1">{{ y }}</b></span></div></div>"#,
    ),
    // A nested spelling is already frozen, so it is an ordinary
    // attribute — only the element that opens the subtree drops its own.
    (
        "nested-v-pre",
        r#"<div><div v-pre><span v-pre></span></div></div>"#,
    ),
    (
        "nested-v-pre-with-bind",
        r#"<div><div v-pre><span v-pre :x="1">c</span></div></div>"#,
    ),
    (
        "sibling-outside",
        r#"<div><div v-pre :x="1"></div><span :y="2"></span></div>"#,
    ),
];

fn render_body(code: &str) -> String {
    code.lines()
        .map(str::trim)
        .skip_while(|line| !line.starts_with("return "))
        .collect::<Vec<_>>()
        .join(" ")
}

#[test]
fn a_v_pre_subtree_compiles_exactly_as_the_legacy_lane_does() {
    for (name, source) in CASES {
        let l2_allocator = Allocator::new();
        let (_, errors, s2) = compile_template(&l2_allocator, source);
        assert!(
            errors.is_empty(),
            "{name}: {source:?} should compile cleanly"
        );

        let legacy_allocator = Allocator::new();
        let (_, _, legacy) = compile_template_legacy_with_options(
            &legacy_allocator,
            source,
            DomCompilerOptions::default(),
        );

        assert_eq!(
            render_body(&s2.code),
            render_body(&legacy.code),
            "{name}: {source:?}",
        );
        assert_eq!(s2.preamble, legacy.preamble, "{name} preamble: {source:?}");
    }
}

#[test]
fn implied_end_tags_keep_the_vue_v_pre_scope() {
    for (source, expected) in [
        (
            r#"<p v-pre><div><meta :role="x" /></div></p>"#,
            r#"return (_openBlock(), _createElementBlock("p", null, [ _createElementVNode("div", null, [ _createElementVNode("meta", { ":role": "x" }) ]) ])) }"#,
        ),
        (
            r#"<select><option v-pre><option><meta :role="x" /></option></option></select>"#,
            r#"return (_openBlock(), _createElementBlock("select", null, [ _createElementVNode("option", null, [ _createElementVNode("option", null, [ _createElementVNode("meta", { ":role": "x" }) ]) ]) ])) }"#,
        ),
        (
            r#"<select><optgroup v-pre><optgroup><meta :role="x" /></optgroup></optgroup></select>"#,
            r#"return (_openBlock(), _createElementBlock("select", null, [ _createElementVNode("optgroup", null, [ _createElementVNode("optgroup", null, [ _createElementVNode("meta", { ":role": "x" }) ]) ]) ])) }"#,
        ),
    ] {
        let allocator = Allocator::new();
        let (_, errors, selected) = compile_template(&allocator, source);
        assert_eq!(
            errors.len(),
            1,
            "standard diagnostics: {source}: {errors:?}"
        );
        assert_eq!(
            errors[0].code,
            vize_atelier_core::errors::ErrorCode::InvalidEndTag,
            "standard diagnostic: {source}"
        );
        assert!(selected.code.is_empty(), "standard must not emit: {source}");

        let quirks_allocator = Allocator::new();
        let (_, quirks_errors, quirks) = compile_template_with_template_syntax(
            &quirks_allocator,
            source,
            DomCompilerOptions::default(),
            TemplateSyntaxMode::Quirks,
        );
        assert!(
            quirks_errors.is_empty(),
            "quirks diagnostics: {source}: {quirks_errors:?}"
        );
        assert_eq!(render_body(&quirks.code), expected, "{source}");
    }
}

/// Compile on both lanes, require identical whole results, and return the
/// shared render body and `_hoisted_1` line.
fn both_lanes(source: &str) -> (String, String) {
    let allocator = Allocator::new();
    let (_, errors, s2) = compile_template(&allocator, source);
    assert!(errors.is_empty(), "{source:?}: {errors:?}");
    let legacy_allocator = Allocator::new();
    let (_, legacy_errors, legacy) = compile_template_legacy_with_options(
        &legacy_allocator,
        source,
        DomCompilerOptions::default(),
    );
    assert!(legacy_errors.is_empty(), "{source:?}: {legacy_errors:?}");
    assert_eq!(s2.code, legacy.code, "whole code: {source:?}");
    assert_eq!(s2.preamble, legacy.preamble, "whole preamble: {source:?}");
    let hoisted = s2
        .preamble
        .lines()
        .find(|line| line.starts_with("const _hoisted_1"))
        .unwrap_or("<no hoisted props>")
        .to_owned();
    (render_body(&s2.code), hoisted)
}

/// Vue refines a tag type only outside `v-pre` (#13314): the element that
/// opens the subtree and every descendant stay plain elements, a component,
/// `<component>`, `<slot>` or `<template>` tag included. Each body is
/// `@vue/compiler-sfc` 3.5.35 `compileTemplate` output without the static
/// caching this entry point leaves off, except two kept spellings: the
/// literal `<component>` keeps its `is` (Vue drops it here, SSR keeps it),
/// and a literal `<template>` passes its text as a string.
#[test]
fn a_v_pre_subtree_keeps_every_element_plain() {
    let cases: &[(&str, &str, &str)] = &[
        (
            "<div v-pre><div><Child>{{ x }}</Child></div></div>",
            r#"return (_openBlock(), _createElementBlock("div", null, [ _createElementVNode("div", null, [ _createElementVNode("Child", null, "{{ x }}") ]) ])) }"#,
            "<no hoisted props>",
        ),
        (
            "<Child v-pre>{{ x }}</Child>",
            r#"return (_openBlock(), _createElementBlock("Child", null, "{{ x }}")) }"#,
            "<no hoisted props>",
        ),
        (
            r#"<div v-pre><component is="x"/></div>"#,
            r#"return (_openBlock(), _createElementBlock("div", null, [ _createElementVNode("component", { is: "x" }) ])) }"#,
            "<no hoisted props>",
        ),
        (
            "<div v-pre><my-comp/></div>",
            r#"return (_openBlock(), _createElementBlock("div", null, [ _createElementVNode("my-comp") ])) }"#,
            "<no hoisted props>",
        ),
        (
            "<div v-pre><slot/></div>",
            r#"return (_openBlock(), _createElementBlock("div", null, [ _createElementVNode("slot") ])) }"#,
            "<no hoisted props>",
        ),
        (
            r#"<div v-pre><template v-if="a">x</template></div>"#,
            r#"return (_openBlock(), _createElementBlock("div", null, [ _createElementVNode("template", { "v-if": "a" }, "x") ])) }"#,
            "<no hoisted props>",
        ),
        (
            r#"<MyComp v-pre :x="1">c</MyComp>"#,
            r#"return (_openBlock(), _createElementBlock("MyComp", _hoisted_1, "c")) }"#,
            r#"const _hoisted_1 = { ":x": "1" }"#,
        ),
        (
            r#"<component v-pre is="x"/>"#,
            r#"return (_openBlock(), _createElementBlock("component", _hoisted_1)) }"#,
            r#"const _hoisted_1 = { is: "x" }"#,
        ),
        // The control outside `v-pre` still resolves its component.
        (
            "<div><Child/></div>",
            r#"return (_openBlock(), _createElementBlock("div", null, [ _createVNode(_component_Child) ])) }"#,
            "<no hoisted props>",
        ),
    ];
    for (source, body, hoisted) in cases {
        assert_eq!(
            both_lanes(source),
            ((*body).to_owned(), (*hoisted).to_owned()),
            "{source:?}"
        );
    }
}

/// A `<template v-pre #head>` under a component is a plain element in the
/// default slot, as Vue keeps it. Both lanes now agree on hoisting it.
#[test]
fn a_v_pre_template_under_a_component_stays_a_default_slot_element() {
    assert_eq!(
        both_lanes("<MyComp><template v-pre #head>c</template></MyComp>"),
        (
            "return (_openBlock(), _createBlock(_component_MyComp, null, { \
default: _withCtx(() => [ _hoisted_1 ]), _: 1 /* STABLE */ })) }"
                .to_owned(),
            r##"const _hoisted_1 = /*#__PURE__*/ _createElementVNode("template", { "#head": "" }, "c")"##
                .to_owned(),
        )
    );
}

/// The frozen names, pinned exactly. Every expectation is
/// `@vue/compiler-dom` 3.6.0-beta.10's own, and the two shapes on the
/// element that *opens* the subtree show the reassembly: the tokenizer
/// had already split `v-bind:x` when `v-pre` took effect, so the name
/// comes back without its separator.
#[test]
fn the_frozen_attributes_keep_their_authored_spelling() {
    let expectations: &[(&str, &str)] = &[
        (
            r#"<div v-pre :x="1"><span></span></div>"#,
            r#"const _hoisted_1 = { ":x": "1" }"#,
        ),
        (
            r#"<div v-pre @click="go"><span></span></div>"#,
            r#"const _hoisted_1 = { "@click": "go" }"#,
        ),
        (
            r#"<div v-pre v-show="s"><span></span></div>"#,
            r#"const _hoisted_1 = { "v-show": "s" }"#,
        ),
        (
            r#"<div v-pre v-bind:x="1"><span></span></div>"#,
            r#"const _hoisted_1 = { "v-bindx": "1" }"#,
        ),
        (
            r#"<div v-pre v-my:a.m="v"><span></span></div>"#,
            r#"const _hoisted_1 = { "v-mya": "v" }"#,
        ),
    ];
    for (source, expected) in expectations {
        let allocator = Allocator::new();
        let (_, _, result) = compile_template(&allocator, source);
        let hoisted = result
            .preamble
            .lines()
            .find(|line| line.starts_with("const _hoisted_1"))
            .unwrap_or("<no hoisted props>");
        assert_eq!(hoisted, *expected, "{source:?}");
    }
}

/// Inside the subtree the authored name survives whole — the tokenizer
/// never split it there.
#[test]
fn a_descendant_keeps_its_separators() {
    let expectations: &[(&str, &str)] = &[
        (
            r#"<div v-pre><span :x="1"></span></div>"#,
            r#"return (_openBlock(), _createElementBlock("div", null, [ _createElementVNode("span", { ":x": "1" }) ])) }"#,
        ),
        (
            r#"<div v-pre><span v-bind:x="1"></span></div>"#,
            r#"return (_openBlock(), _createElementBlock("div", null, [ _createElementVNode("span", { "v-bind:x": "1" }) ])) }"#,
        ),
        (
            r#"<div v-pre><span v-if="a">y</span></div>"#,
            r#"return (_openBlock(), _createElementBlock("div", null, [ _createElementVNode("span", { "v-if": "a" }, "y") ])) }"#,
        ),
        (
            r#"<div v-pre><span v-for="i in l">{{ i }}</span></div>"#,
            r#"return (_openBlock(), _createElementBlock("div", null, [ _createElementVNode("span", { "v-for": "i in l" }, "{{ i }}") ])) }"#,
        ),
    ];
    for (source, expected) in expectations {
        let allocator = Allocator::new();
        let (_, _, result) = compile_template(&allocator, source);
        assert_eq!(render_body(&result.code), *expected, "{source:?}");
    }
}

/// The spelling itself never reaches the output — Vue emits the frozen
/// attributes without it.
#[test]
fn the_v_pre_attribute_itself_is_dropped() {
    let allocator = Allocator::new();
    let (_, _, result) = compile_template(&allocator, r#"<div v-pre :x="1">c</div>"#);
    let hoisted = result
        .preamble
        .lines()
        .find(|line| line.starts_with("const _hoisted_1"))
        .unwrap_or("<no hoisted props>");
    assert_eq!(hoisted, r#"const _hoisted_1 = { ":x": "1" }"#);
    assert_eq!(
        render_body(&result.code),
        r#"return (_openBlock(), _createElementBlock("div", _hoisted_1, "c")) }"#,
    );
}

/// Compare whole outputs: trimming the render body would hide whitespace.
#[test]
fn v_pre_whitespace_matches_complete_legacy_dom_outputs() {
    let original = include_str!(
        "../../../tests/_fixtures/differential/compiler/v-pre-whitespace/format-v-pre-content.vue.txt"
    );
    assert_eq!(original.len(), 228);
    let template = original
        .strip_prefix("<template>")
        .unwrap()
        .split_once("</template>")
        .unwrap()
        .0;
    for source in [
        template,
        "<div v-pre><i></i> <b></b></div>",
        "<div v-pre>\n  {{ literal }}\n</div><p>{{ active }}</p>",
        "<pre v-pre>x\n  <i></i>\n  y</pre>",
        "<div v-pre><i></i>\u{a0}<b></b></div>",
        "<div v-pre>\n<!--keep--><i :id=\"raw\"></i>\n</div>",
        "<div v-pre>\n<i></i>\r\n<b></b>\n</div><p :id=\"active\"></p>",
    ] {
        let allocator = Allocator::new();
        let mut capture = StageCapture::new("dom");
        let (_, errors, selected) =
            compile_template_with_custom_elements_template_syntax_codegen_and_experimental_options_with_stage_capture(
                &allocator,
                source,
                DomCompilerOptions::default(),
                TemplateSyntaxMode::Standard,
                CustomElementMatcher::default(),
                CodegenOptions::default(),
                CodegenExperimentalOptions::default(),
                &mut capture,
            );
        assert_eq!(
            capture.outcome,
            CaptureOutcome::Accepted,
            "native DOM must execute: {source:?}"
        );
        let legacy_allocator = Allocator::new();
        let (_, legacy_errors, legacy) = compile_template_legacy_with_options(
            &legacy_allocator,
            source,
            DomCompilerOptions::default(),
        );
        assert!(errors.is_empty(), "native diagnostics: {errors:?}");
        assert!(
            legacy_errors.is_empty(),
            "legacy diagnostics: {legacy_errors:?}"
        );
        assert_eq!(selected.code, legacy.code, "whole code: {source:?}");
        assert_eq!(
            selected.preamble, legacy.preamble,
            "whole preamble: {source:?}"
        );
        assert_eq!(selected.map, legacy.map, "whole map: {source:?}");
    }
}
