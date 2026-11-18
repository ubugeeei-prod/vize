use super::ordinary;
use crate::types::{BindingType, SfcCompileOptions};
use crate::{SfcParseOptions, compile_sfc, parse_sfc};

/// The binding table of a normal script, sorted by name.
fn table(content: &str, lang: Option<&str>) -> Vec<(String, BindingType)> {
    let mut rows: Vec<_> = ordinary(content, lang)
        .bindings
        .into_iter()
        .map(|(name, binding)| (name.to_string(), binding))
        .collect();
    rows.sort_by(|left, right| left.0.cmp(&right.0));
    rows
}

fn rows(expected: &[(&str, BindingType)]) -> Vec<(String, BindingType)> {
    expected
        .iter()
        .map(|(name, binding)| ((*name).to_owned(), *binding))
        .collect()
}

#[test]
fn bindings_are_classified_as_vue_compile_script_classifies_them() {
    // Vue 3.5 `compileScript` bindings for this normal script (the paired
    // setup's own `local` aside). `declare` statements bind nothing, and an
    // array or object literal can never be a ref.
    let script = "import { ref, reactive } from 'vue'
import Vue from 'vue'
import Child from './Child.vue'
import { helper } from './helper'
import * as ns from './ns'
import type { T } from './t'
export const list = ['a', 'b'] as const
export const options = { a: 1 }
export const label = 'x'
const count = ref(0)
const state = reactive({})
const derived = helper()
let mutable = 1
export function run() {}
export class Model {}
export enum Kind { A = 1 }
enum Dynamic { A = helper() }
declare const ambient: number
declare function ambientFn(): void
declare enum AmbientKind { A }
const { a, b: [c] } = helper()
";
    use BindingType::{
        LiteralConst, SetupConst, SetupLet, SetupMaybeRef, SetupReactiveConst, SetupRef,
    };
    assert_eq!(
        table(script, Some("ts")),
        rows(&[
            ("Child", SetupConst),
            ("Dynamic", SetupConst),
            ("Kind", LiteralConst),
            ("Model", SetupConst),
            ("Vue", SetupConst),
            ("a", SetupMaybeRef),
            ("c", SetupMaybeRef),
            ("count", SetupRef),
            ("derived", SetupMaybeRef),
            ("helper", SetupMaybeRef),
            ("label", LiteralConst),
            ("list", SetupConst),
            ("mutable", SetupLet),
            ("ns", SetupConst),
            ("options", SetupConst),
            ("reactive", SetupConst),
            ("ref", SetupConst),
            ("run", SetupConst),
            ("state", SetupReactiveConst),
        ])
    );
}

#[test]
fn the_block_lang_selects_the_parser() {
    // A `tsx` script is read with JSX; it used to be read as plain TypeScript,
    // which refused it and forwarded nothing.
    let script = "export const icon = <i />\nexport const wrapped = (<b />)\n";
    assert_eq!(
        table(script, Some("tsx")),
        rows(&[
            ("icon", BindingType::SetupMaybeRef),
            ("wrapped", BindingType::SetupMaybeRef),
        ])
    );
    assert_eq!(table(script, Some("ts")), rows(&[]));
}

#[test]
fn a_script_the_levels_refuse_forwards_nothing() {
    assert_eq!(table("export const = 1\n", Some("ts")), rows(&[]));
}

#[test]
fn ambient_names_resolve_as_vue_resolves_them() {
    // Vue 3.5 renders this template with `_resolveComponent("C3")` and
    // `_ctx.func(count)`: an ambient declaration is not a module binding.
    let source = "<script lang=\"ts\">\nexport const levels = ['a', 'b'] as const\nexport const COLORS = { red: 1 }\ndeclare const C3: any\ndeclare function func(a: number): number\nexport const count = 1\n</script>\n<script setup lang=\"ts\">\nconst local = 1\n</script>\n<template><C3 :x=\"levels\" :y=\"COLORS\" :z=\"func(count)\" :w=\"local\"/></template>\n";
    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("parse");
    let result = compile_sfc(&descriptor, SfcCompileOptions::default()).expect("compile");
    assert_eq!(
        result.code.as_str(),
        "import { resolveComponent as _resolveComponent, openBlock as _openBlock, createBlock as _createBlock } from \"vue\"


export const levels = [\"a\", \"b\"];
export const COLORS = { red: 1 };
export const count = 1;

export default {
  __name: 'anonymous',
  setup(__props) {

const local = 1;

return (_ctx, _cache) => {
  const _component_C3 = _resolveComponent(\"C3\")

  return (_openBlock(), _createBlock(_component_C3, {
    x: levels,
    y: COLORS,
    z: _ctx.func(count),
    w: local
  }, null, 8 /* PROPS */, [\"x\", \"y\", \"z\", \"w\"]))
}
}

}"
    );
}

#[test]
fn a_default_named_import_from_a_vue_file_renders_as_its_component() {
    // Vue 3.5 renders `_createBlock(Child)`: `{ default as Child }` from a
    // `.vue` file is a setup constant in either script, as a default import is.
    for script in [
        "<script lang=\"ts\">\nimport { default as Child } from './Child.vue'\n</script>\n<script setup lang=\"ts\">\n</script>\n",
        "<script setup lang=\"ts\">\nimport { default as Child } from './Child.vue'\n</script>\n",
    ] {
        let source = format!("{script}<template><Child /></template>\n");
        let descriptor = parse_sfc(&source, SfcParseOptions::default()).expect("parse");
        let result = compile_sfc(&descriptor, SfcCompileOptions::default()).expect("compile");
        let render = result
            .code
            .lines()
            .find(|line| line.trim_start().starts_with("return (_openBlock()"))
            .expect("render return");
        assert_eq!(
            render.trim(),
            "return (_openBlock(), _createBlock(Child))",
            "{source}"
        );
    }
}
