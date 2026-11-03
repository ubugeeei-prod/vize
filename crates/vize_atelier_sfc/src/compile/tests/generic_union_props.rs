//! Runtime prop types for unions with an unresolvable member, such as a
//! component generic. Vue keeps `Boolean`/`Function` (casting, function
//! defaults) and skips the runtime check; anything else is `null`.
#![expect(clippy::disallowed_macros, reason = "tests assert by panicking")]
use super::super::compile_sfc;
use crate::types::SfcCompileOptions;
use crate::{SfcParseOptions, parse_sfc};

fn compile(source: &str, is_prod: bool) -> String {
    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("parse");
    let mut options = SfcCompileOptions::default();
    options.template.is_prod = is_prod;
    let code = compile_sfc(&descriptor, options).expect("compile").code;
    code.split_whitespace().collect::<Vec<_>>().join(" ")
}

const PROPS: &str = r#"<script setup lang="ts" generic="T extends object, K">
import type { Ext } from 'ext'
withDefaults(defineProps<{
  flag: boolean | T
  nullableFlag?: boolean | T | null
  fn?: (() => void) | T
  ctor?: Function | K
  ext: Ext | (() => void)
  mixed: string | boolean | Ext
  either: (() => void) | T | string
  plain?: T | number
  key?: keyof T
  local?: keyof Local
  both: T & boolean
}>(), { fn: () => {} })
interface Local { a: string }
</script>
<template><div /></template>"#;

#[test]
fn unknown_union_members_keep_boolean_and_function_with_skip_check() {
    let code = compile(PROPS, false);
    for expected in [
        "flag: { type: Boolean, required: true, skipCheck: true }",
        "nullableFlag: { type: [Boolean, null], required: false, skipCheck: true }",
        // A function default is the value, not a factory, only for `Function`.
        "fn: { type: Function, required: false, skipCheck: true, default: () => {} }",
        "ctor: { type: Function, required: false, skipCheck: true }",
        "ext: { type: Function, required: true, skipCheck: true }",
        "mixed: { type: [String, Boolean], required: true, skipCheck: true }",
        "either: { type: [Function, String], required: true, skipCheck: true }",
        "plain: { type: null, required: false }",
        // Keys of a generic may be numbers or symbols.
        "key: { type: null, required: false }",
        "local: { type: String, required: false }",
        "both: { type: Boolean, required: true }",
    ] {
        assert!(code.contains(expected), "missing `{expected}`:\n{code}");
    }
}

#[test]
fn production_keeps_casting_types_without_skip_check() {
    let code = compile(PROPS, true);
    assert!(code.contains("flag: { type: Boolean }"), "{code}");
    assert!(
        code.contains("fn: { type: Function, default: () => {} }"),
        "{code}"
    );
    assert!(!code.contains("skipCheck"), "{code}");
}

#[test]
fn define_model_keeps_boolean_casting_for_generic_unions() {
    let source = r#"<script setup lang="ts" generic="T">
const open = defineModel<boolean | T>('open')
const run = defineModel<(() => void) | T>('run', { default: () => {} })
const value = defineModel<T | string>()
</script>
<template><div /></template>"#;
    let code = compile(source, false);
    for expected in [
        r#""open": { type: Boolean, skipCheck: true }"#,
        r#""run": { type: Function, skipCheck: true, ...{ default: () => {} } }"#,
        r#""modelValue": { type: null }"#,
    ] {
        assert!(code.contains(expected), "missing `{expected}`:\n{code}");
    }
    let prod = compile(source, true);
    assert!(prod.contains(r#""open": { type: Boolean }"#), "{prod}");
    assert!(!prod.contains("skipCheck"), "{prod}");
}
