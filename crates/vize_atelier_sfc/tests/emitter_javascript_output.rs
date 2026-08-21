//! The emitter's module output must be plain JavaScript.
//!
//! The Vite plugin used to re-run Vite's TypeScript strip over every module the
//! Rust compiler emitted. It no longer does: the napi boundary now guarantees
//! JavaScript via `ensure_javascript_output`. These tests pin that guarantee so
//! a regression in the script pipeline cannot leak TypeScript into the bundler.

#![cfg(feature = "compile")]

use vize_atelier_sfc::{
    SfcCompileOptions, SfcParseOptions,
    compile_script::typescript::{
        ensure_javascript_output, is_plain_javascript, transform_typescript_to_js,
    },
    compile_sfc, parse_sfc,
};

/// Compile an SFC exactly the way the Vite plugin does: `is_ts` is left at its
/// default `false`, which is what makes the script pipeline strip TypeScript
/// itself instead of deferring it to the bundler.
fn compile_like_the_vite_plugin(source: &str) -> vize_carton::String {
    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("SFC should parse");
    compile_sfc(&descriptor, SfcCompileOptions::default())
        .expect("SFC should compile")
        .code
}

fn assert_emits_plain_javascript(label: &str, source: &str) {
    let code = compile_like_the_vite_plugin(source);
    assert!(
        is_plain_javascript(&code),
        "{label}: emitter output still contains TypeScript syntax, so removing the JS-side \
         strip would ship it to the bundler:\n{code}"
    );
}

#[test]
fn type_annotations_are_stripped_by_the_emitter() {
    assert_emits_plain_javascript(
        "typed bindings and parameters",
        r#"<script setup lang="ts">
const count: number = 1
const label: string | null = null
function greet(who: string, times?: number): string {
  return who.repeat(times ?? 1)
}
const shout = (who: string): string => greet(who) + "!"
</script>
<template><div>{{ count }}{{ label }}{{ shout('a') }}</div></template>"#,
    );
}

#[test]
fn generics_are_stripped_by_the_emitter() {
    assert_emits_plain_javascript(
        "generic functions and generic call sites",
        r#"<script setup lang="ts">
import { ref } from "vue"
function identity<T>(value: T): T {
  return value
}
const box = ref<Array<string>>([])
const first = identity<string>("a")
</script>
<template><div>{{ box }}{{ first }}</div></template>"#,
    );
}

#[test]
fn interfaces_type_aliases_and_enums_are_stripped_by_the_emitter() {
    assert_emits_plain_javascript(
        "declarations",
        r#"<script setup lang="ts">
interface User { id: number; name: string }
type Maybe<T> = T | null
enum Level { Low, High }
declare const injected: string
const user: Maybe<User> = null
const level = Level.High
</script>
<template><div>{{ user }}{{ level }}{{ injected }}</div></template>"#,
    );
}

#[test]
fn assertions_and_satisfies_are_stripped_by_the_emitter() {
    assert_emits_plain_javascript(
        "as / satisfies / non-null",
        r#"<script setup lang="ts">
const raw = { a: 1 } as Record<string, number>
const frozen = { b: 2 } as const
const checked = { c: 3 } satisfies Record<string, number>
const forced = (raw.a as unknown) as string
const present = raw!.a
</script>
<template><div>{{ raw }}{{ frozen }}{{ checked }}{{ forced }}{{ present }}</div></template>"#,
    );
}

#[test]
fn type_only_imports_are_stripped_by_the_emitter() {
    assert_emits_plain_javascript(
        "type-only imports and inline type specifiers",
        r#"<script setup lang="ts">
import type { Ref } from "vue"
import { ref, type ComputedRef } from "vue"
const a: Ref<number> = ref(0)
const b: ComputedRef<number> | null = null
</script>
<template><div>{{ a }}{{ b }}</div></template>"#,
    );
}

#[test]
fn decorators_and_access_modifiers_are_stripped_by_the_emitter() {
    assert_emits_plain_javascript(
        "class members",
        r#"<script lang="ts">
function logged(_target: unknown, _key: string): void {}
export default class Widget {
  private readonly id: number = 1
  protected label?: string
  @logged
  render(): null {
    return null
  }
}
</script>"#,
    );
}

/// `<script lang="uts">` — uni-app's TypeScript dialect — is the one case the
/// script pipeline does *not* strip, because `is_ts_lang` only matches `ts` and
/// `tsx`. Before this change the Vite plugin's oxc pass happened to clean it up
/// as a side effect; now `ensure_javascript_output` is what does it, so pin
/// both halves: the raw emitter output is still TypeScript, and the napi
/// boundary's guarantee turns it into JavaScript.
#[test]
fn uts_script_output_is_rescued_by_ensure_javascript_output() {
    let code = compile_like_the_vite_plugin(
        r#"<script setup lang="uts">
const count: number = 1
function greet(who: string): string {
  return who
}
</script>
<template><div>{{ count }}{{ greet('a') }}</div></template>"#,
    );

    assert!(
        !is_plain_javascript(&code),
        "`lang=\"uts\"` is expected to bypass the script pipeline's TypeScript strip; if this \
         starts passing the emitter learned `uts` and this test should become a plain \
         `assert_emits_plain_javascript` case:\n{code}"
    );

    let guaranteed = ensure_javascript_output(code);
    assert!(
        is_plain_javascript(&guaranteed),
        "the napi boundary must strip TypeScript the script pipeline left behind:\n{guaranteed}"
    );
    assert!(
        !guaranteed.contains(": number"),
        "`uts` type annotations must not survive:\n{guaranteed}"
    );
}

#[test]
fn ensure_javascript_output_passes_javascript_through_untouched() {
    let js: vize_carton::String = "import { ref } from \"vue\";\nexport default { ref };".into();
    let out = ensure_javascript_output(js.clone());
    assert_eq!(
        out, js,
        "code that already parses as JavaScript must not be re-printed"
    );
}

/// A semantic diagnostic must not abort the emitter's strip.
///
/// `transform_typescript_to_js` bails on any `SemanticBuilder` error and hands
/// back the untouched TypeScript — correct while compiling a script, but fatal
/// for an output guarantee, since the JavaScript-side pass this replaces never
/// ran a semantic check at all.
#[test]
fn semantic_diagnostics_do_not_abort_the_emitter_strip() {
    let redeclared: vize_carton::String =
        "let a: number = 1;\nlet a: number = 2;\nexport { a };".into();

    assert!(
        !is_plain_javascript(&transform_typescript_to_js(&redeclared)),
        "precondition: the script-pipeline strip is expected to bail on a semantic diagnostic"
    );

    let guaranteed = ensure_javascript_output(redeclared);
    assert!(
        is_plain_javascript(&guaranteed),
        "the emitter strip must ignore semantic diagnostics and still emit JavaScript:\n\
         {guaranteed}"
    );
}

#[test]
fn is_plain_javascript_rejects_typescript() {
    assert!(is_plain_javascript("export const a = 1;"));
    assert!(!is_plain_javascript("export const a: number = 1;"));
    assert!(!is_plain_javascript("interface A { b: number }"));
    assert!(!is_plain_javascript("export default {"));
}
