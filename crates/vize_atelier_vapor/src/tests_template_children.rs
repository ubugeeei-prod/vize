//! Complete-output regressions for transparent `<template>` children (#3595).
#![allow(clippy::disallowed_macros, clippy::disallowed_types)]

use super::{VaporCompilerOptions, compile_vapor};
use vize_atelier_dom::{DomCompilerOptions, compile_template_with_options};
use vize_carton::Bump;

const V_MATCH: &str = r#"<div><ul v-match="status"><li v-case="'ready'">Ready</li><li>Plain</li></ul><span>{{ x }}</span></div>"#;
const DYNAMIC_SIBLING: &str = r#"<div><ul v-match="status"><li v-case="'ready'">Ready</li><li :id="plainId"><span>{{ label }}</span></li></ul><footer :class="cls">Tail</footer></div>"#;
const NESTED: &str = r#"<div><section v-match="outer"><ul v-case="'open'" v-match="inner"><li v-case="'x'">X</li><li v-case.default>Inner fallback</li></ul><p v-case.default>Outer fallback</p></section><footer :class="cls">tail</footer></div>"#;
const CONTROL_FLOW: &str = r#"<div><aside v-if="aside">Aside</aside><ul v-match="status"><li v-case="'ready'" v-for="item in items" :key="item.id" :class="item.kind">{{ item.label }}</li><li v-case.default>Empty</li></ul><p>Last</p></div>"#;
const TEXT_RUN: &str = r#"<div><ul v-match="status"><li v-case="'ready'">Ready</li>Before {{ label }} after</ul><footer :class="cls">Tail</footer></div>"#;
const SEPARATED_TEXT_RUNS: &str = r#"<div><ul v-match="status"><li v-case="'ready'">Ready</li>Before {{ a }}<em>mid</em>After {{ b }}</ul></div>"#;

fn vapor_output(source: &str) -> std::string::String {
    let allocator = Bump::new();
    let result = compile_vapor(
        &allocator,
        source,
        VaporCompilerOptions {
            experimental_patterned_template: true,
            ..Default::default()
        },
    );
    format!(
        "errors={:?}\ntemplates={:?}\ncode=\n{}",
        result.error_messages, result.templates, result.code
    )
}

fn dom_output(source: &str) -> std::string::String {
    let allocator = Bump::new();
    let (root, errors, result) = compile_template_with_options(
        &allocator,
        source,
        DomCompilerOptions {
            experimental_patterned_template: true,
            ..Default::default()
        },
    );
    format!(
        "errors={:?}\ncomponents={:?}\npreamble=\n{}\ncode=\n{}",
        errors, root.components, result.preamble, result.code
    )
}

#[test]
fn patterned_template_child_vapor_output() {
    insta::assert_snapshot!(vapor_output(V_MATCH));
}

#[test]
fn patterned_template_child_dom_output() {
    insta::assert_snapshot!(dom_output(V_MATCH));
}

#[test]
fn dynamic_sibling_inside_patterned_template_vapor_output() {
    insta::assert_snapshot!(vapor_output(DYNAMIC_SIBLING));
}

#[test]
fn dynamic_sibling_inside_patterned_template_dom_output() {
    insta::assert_snapshot!(dom_output(DYNAMIC_SIBLING));
}

#[test]
fn nested_patterned_template_child_vapor_output() {
    insta::assert_snapshot!(vapor_output(NESTED));
}

#[test]
fn nested_patterned_template_child_dom_output() {
    insta::assert_snapshot!(dom_output(NESTED));
}

#[test]
fn adjacent_control_flow_and_keyed_pattern_vapor_output() {
    insta::assert_snapshot!(vapor_output(CONTROL_FLOW));
}

#[test]
fn adjacent_control_flow_and_keyed_pattern_dom_output() {
    insta::assert_snapshot!(dom_output(CONTROL_FLOW));
}

#[test]
fn nested_text_run_inside_patterned_template_vapor_output() {
    insta::assert_snapshot!(vapor_output(TEXT_RUN));
}

#[test]
fn nested_text_run_inside_patterned_template_dom_output() {
    insta::assert_snapshot!(dom_output(TEXT_RUN));
}

#[test]
fn separated_text_runs_inside_patterned_template_vapor_output() {
    insta::assert_snapshot!(vapor_output(SEPARATED_TEXT_RUNS));
}

#[test]
fn separated_text_runs_inside_patterned_template_dom_output() {
    insta::assert_snapshot!(dom_output(SEPARATED_TEXT_RUNS));
}

const DEEP_TEMPLATE_DEPTH: usize = 1100;
const SMALL_STACK: usize = 256 * 1024;

fn vapor_output_on_small_stack(source: std::string::String) -> std::string::String {
    std::thread::Builder::new()
        .stack_size(SMALL_STACK)
        .spawn(move || vapor_output(&source))
        .expect("spawn small-stack compiler thread")
        .join()
        .expect("small-stack compiler thread finished")
}

fn deeply_nested_template_source(wrapped: bool) -> std::string::String {
    let mut source = if wrapped {
        std::string::String::from("<div>")
    } else {
        std::string::String::new()
    };
    for _ in 0..DEEP_TEMPLATE_DEPTH {
        source.push_str("<template>");
    }
    source.push_str(r#"<ul v-match="status"><li v-case="'ready'">Ready</li>{{ label }}</ul>"#);
    for _ in 0..DEEP_TEMPLATE_DEPTH {
        source.push_str("</template>");
    }
    if wrapped {
        source.push_str("</div>");
    }
    source
}

#[test]
fn deeply_nested_transparent_templates_compile_on_a_small_stack() {
    // Match the depth/stack ratio used by the whole-pipeline regression in
    // vize_atelier_dom: an ordinary recursive walk exhausts this stack well
    // before reaching the innermost patterned child.
    let output = vapor_output_on_small_stack(deeply_nested_template_source(false));

    assert!(output.starts_with("errors=[]"), "{output}");
    assert!(
        !output.contains("resolveComponent(\"template\")"),
        "{output}"
    );
    assert!(output.contains("_toDisplayString(_ctx.label)"), "{output}");
}

#[test]
fn deeply_nested_deferred_template_chain_compiles_on_a_small_stack() {
    let output = vapor_output_on_small_stack(deeply_nested_template_source(true));
    assert!(output.starts_with("errors=[]"), "{output}");
    assert!(
        !output.contains("resolveComponent(\"template\")"),
        "{output}"
    );
    assert!(output.contains("_toDisplayString(_ctx.label)"), "{output}");
}
