//! A reassignable setup declaration stays one variable for script and
//! template in every output that returns setup state to a separate render.
//!
//! The returned setup-state object is read and written by the render through
//! `$setup` / `_ctx`. A `let` or `var` returned by value would be a snapshot:
//! template writes (`:ref="(e) => (el = e)"`, `@click="n++"`, `v-model`) would
//! never reach the script variable, and script reassignments would never reach
//! the template. Both are returned as an accessor pair instead.

#![expect(
    clippy::expect_used,
    clippy::disallowed_types,
    reason = "tests assert by panicking; entries are compared as std strings"
)]

use vize_atelier_core::{CodegenOptions, TemplateSyntaxMode, options::CustomElementMatcher};
use vize_atelier_sfc::{
    ScriptCompileOptions, SfcCompileOptions, SfcParseOptions, SfcScriptOutputMode,
    TemplateCompileOptions, compile_sfc, compile_sfc_for_adapter, parse_sfc,
};
use vize_l0::String;

const SOURCE: &str = r#"<script setup>
let el = null
var v = 0
const c = 1
</script>

<template>
  <div :ref="(e) => (el = e)" @click="v++">{{ c }} {{ v }}</div>
</template>
"#;

const EL_ACCESSORS: &str = "get el() { return el }, set el(v) { el = v }";
/// The setter parameter is renamed so it cannot shadow the binding `v`.
const V_ACCESSORS: &str = "get v() { return v }, set v(_v) { v = _v }";

/// Client DOM output keeps the setup state separate from the render only in
/// the module form Vite development and other bundler adapters request.
fn compile_dom_module() -> String {
    let descriptor = parse_sfc(SOURCE, SfcParseOptions::default()).expect("parse SFC");
    let result = compile_sfc_for_adapter(
        &descriptor,
        SfcCompileOptions::default(),
        TemplateSyntaxMode::Standard,
        CustomElementMatcher::default(),
        CodegenOptions::default(),
        SfcScriptOutputMode::SeparateTemplate,
    )
    .expect("compile SFC");
    assert!(result.errors.is_empty(), "{:?}", result.errors);
    result.code
}

fn compile(vapor: bool, ssr: bool, is_prod: bool) -> String {
    let descriptor = parse_sfc(SOURCE, SfcParseOptions::default()).expect("parse SFC");
    let options = SfcCompileOptions {
        vapor,
        script: ScriptCompileOptions::default(),
        template: TemplateCompileOptions {
            ssr,
            is_prod,
            ..Default::default()
        },
        ..Default::default()
    };
    let result = compile_sfc(&descriptor, options).expect("compile SFC");
    assert!(result.errors.is_empty(), "{:?}", result.errors);
    result.code
}

/// The entries of the returned setup-state object, accessor pairs folded into
/// one `@name` entry, in a stable order.
fn returned_entries(code: &str) -> std::vec::Vec<std::string::String> {
    let lines: std::vec::Vec<&str> = code
        .lines()
        .map(str::trim)
        .filter_map(|line| line.strip_prefix("const __returned__ = { "))
        .collect();
    assert_eq!(lines.len(), 1, "{code}");
    let body = lines
        .first()
        .and_then(|line| line.strip_suffix(" }"))
        .expect("one-line returned object");
    let folded = body.replace(EL_ACCESSORS, "@el").replace(V_ACCESSORS, "@v");
    let mut entries: std::vec::Vec<std::string::String> =
        folded.split(", ").map(std::string::String::from).collect();
    entries.sort();
    entries
}

#[test]
fn dom_setup_state_returns_let_and_var_as_accessors() {
    let code = compile_dom_module();
    assert_eq!(returned_entries(&code), ["@el", "@v", "c"], "{code}");
}

#[test]
fn ssr_setup_state_returns_let_and_var_as_accessors() {
    let code = compile(false, true, false);
    assert_eq!(returned_entries(&code), ["@el", "@v", "c"], "{code}");
}

#[test]
fn vapor_setup_state_returns_let_and_var_as_accessors() {
    for is_prod in [false, true] {
        let code = compile(true, false, is_prod);
        assert_eq!(
            returned_entries(&code),
            ["@el", "@v", "c", "vaporTemplateRefSetter"],
            "{code}"
        );
    }
}

/// The pinned differential row: a template destructuring write to a `var`.
#[test]
fn template_write_to_setup_var_has_a_pinned_ssr_module() {
    let parse = || SfcParseOptions {
        filename: "Layout.vue".into(),
        ..Default::default()
    };
    let source = include_str!(
        "../../../tests/_fixtures/differential/compiler/setup-var-template-write/Layout.vue.txt"
    );
    let descriptor = parse_sfc(source, parse()).expect("parse SFC");
    let options = SfcCompileOptions {
        parse: parse(),
        template: TemplateCompileOptions {
            ssr: true,
            ..Default::default()
        },
        ..Default::default()
    };
    let result = compile_sfc(&descriptor, options).expect("compile SFC");
    assert_eq!(
        result.code.as_str(),
        include_str!(
            "../../../tests/_fixtures/differential/compiler/setup-var-template-write/module.expected.txt"
        )
    );
}
