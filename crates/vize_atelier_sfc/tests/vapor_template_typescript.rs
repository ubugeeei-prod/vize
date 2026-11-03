//! A TypeScript SFC hands its template's TS-ness to the Vapor lane, which
//! emits JavaScript render code. TypeScript output (`is_ts`) keeps the
//! script's types, so no later module strip can hide what the lane emits.

use vize_atelier_sfc::{
    ScriptCompileOptions, SfcCompileOptions, SfcParseOptions, TemplateCompileOptions, compile_sfc,
    parse_sfc,
};
use vize_carton::{String, cstr};

fn render(lang: &str, template: &str, is_ts: bool) -> String {
    let source = cstr!(
        "<script setup{lang} vapor>\nconst pick = (value: unknown) => value\nconst a = 1, b = 2, c = 3\n</script>\n\n<template>\n  {template}\n</template>\n"
    );
    let descriptor = parse_sfc(&source, SfcParseOptions::default()).expect("parse");
    let result = compile_sfc(
        &descriptor,
        SfcCompileOptions {
            script: ScriptCompileOptions {
                is_ts,
                ..Default::default()
            },
            template: TemplateCompileOptions {
                is_ts,
                ..Default::default()
            },
            ..Default::default()
        },
    )
    .expect("compile");
    assert_eq!(result.errors.len(), 0, "{}", result.code);
    let mut out = String::default();
    let mut inside = false;
    for line in result.code.lines() {
        inside |= line == "function render(_ctx, $props, $emit, $attrs, $slots) {";
        if inside {
            out.push_str(line);
            out.push('\n');
        }
        if inside && line == "}" {
            break;
        }
    }
    out
}

#[test]
fn typescript_template_expressions_leave_the_vapor_lane_as_javascript() {
    assert_eq!(
        render(
            r#" lang="ts""#,
            r#"<div @click="(e: MouseEvent): void => pick(e)">{{ pick<string> }}</div>"#,
            true
        ),
        "function render(_ctx, $props, $emit, $attrs, $slots) {\n  const n0 = t0()\n  const x0 = _txt(n0)\n  _on(n0, \"click\", (e) => _ctx.pick(e))\n  _renderEffect(() => _setText(x0, _toDisplayString(_ctx.pick)))\n  return n0\n}\n"
    );
}

/// Negative control: a JavaScript SFC's template is not erased. `a < b > (c)`
/// is two comparisons in JavaScript and a generic call in TypeScript.
#[test]
fn javascript_template_expressions_keep_their_bytes() {
    let template = "<div>{{ a < b > (c) }}</div>";
    assert_eq!(
        render("", template, false),
        "function render(_ctx, $props, $emit, $attrs, $slots) {\n  const n0 = t0()\n  const x0 = _txt(n0)\n  _renderEffect(() => _setText(x0, _toDisplayString(_ctx.a < _ctx.b > (_ctx.c))))\n  return n0\n}\n"
    );
    assert_eq!(
        render(r#" lang="ts""#, template, true),
        "function render(_ctx, $props, $emit, $attrs, $slots) {\n  const n0 = t0()\n  const x0 = _txt(n0)\n  _renderEffect(() => _setText(x0, _toDisplayString(_ctx.a(_ctx.c))))\n  return n0\n}\n"
    );
}
