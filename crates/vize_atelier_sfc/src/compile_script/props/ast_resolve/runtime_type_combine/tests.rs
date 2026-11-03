//! Runtime prop types for unions with an unresolvable member, such as a
//! component generic. Vue keeps `Boolean`/`Function` (casting, function
//! defaults) and skips the runtime check; anything else is `null`.
use crate::compile_sfc;
use crate::types::SfcCompileOptions;
use crate::{SfcParseOptions, parse_sfc};

/// The whole `props: …` option of the compiled component, one entry per prop.
fn props_option(source: &str, is_prod: bool) -> Vec<String> {
    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("parse");
    let mut options = SfcCompileOptions::default();
    options.template.is_prod = is_prod;
    let code = compile_sfc(&descriptor, options).expect("compile").code;
    let mut entries: Vec<String> = Vec::new();
    let mut depth = 0_usize;
    for line in code
        .lines()
        .map(str::trim)
        .skip_while(|line| !line.starts_with("props: "))
        .skip(1)
    {
        if depth == 0 && line.starts_with('}') {
            break;
        }
        match entries.last_mut() {
            Some(entry) if depth > 0 => {
                entry.push(' ');
                entry.push_str(line);
            }
            _ => entries.push(line.to_owned()),
        }
        depth += line.matches('{').count();
        depth -= line.matches('}').count();
    }
    entries
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
    assert_eq!(
        props_option(PROPS, false),
        [
            "flag: { type: Boolean, required: true, skipCheck: true },",
            "nullableFlag: { type: [Boolean, null], required: false, skipCheck: true },",
            // A function default is the value, not a factory, only for `Function`.
            "fn: { type: Function, required: false, skipCheck: true, default: () => {} },",
            "ctor: { type: Function, required: false, skipCheck: true },",
            "ext: { type: Function, required: true, skipCheck: true },",
            "mixed: { type: [String, Boolean], required: true, skipCheck: true },",
            "either: { type: [Function, String], required: true, skipCheck: true },",
            "plain: { type: null, required: false },",
            // Keys of a generic may be numbers or symbols.
            "key: { type: null, required: false },",
            "local: { type: String, required: false },",
            "both: { type: Boolean, required: true }",
        ]
    );
}

#[test]
fn production_keeps_casting_types_without_skip_check() {
    assert_eq!(
        props_option(PROPS, true),
        [
            "flag: { type: Boolean },",
            "nullableFlag: { type: [Boolean, null] },",
            "fn: { type: Function, default: () => {} },",
            "ctor: { type: Function },",
            "ext: { type: Function },",
            "mixed: { type: [String, Boolean] },",
            "either: { type: [Function, String] },",
            "plain: {},",
            "key: {},",
            "local: {},",
            "both: { type: Boolean }",
        ]
    );
}

const MODELS: &str = r#"<script setup lang="ts" generic="T">
const open = defineModel<boolean | T>('open')
const run = defineModel<(() => void) | T>('run', { default: () => {} })
const value = defineModel<T | string>()
</script>
<template><div /></template>"#;

#[test]
fn define_model_keeps_boolean_casting_for_generic_unions() {
    assert_eq!(
        props_option(MODELS, false),
        [
            r#""open": { type: Boolean, skipCheck: true },"#,
            r#""openModifiers": {},"#,
            r#""run": { type: Function, skipCheck: true, ...{ default: () => {} } },"#,
            r#""runModifiers": {},"#,
            r#""modelValue": { type: null },"#,
            r#""modelModifiers": {}"#,
        ]
    );
    assert_eq!(
        props_option(MODELS, true),
        [
            r#""open": { type: Boolean },"#,
            r#""openModifiers": {},"#,
            r#""run": { type: Function, ...{ default: () => {} } },"#,
            r#""runModifiers": {},"#,
            r#""modelValue": { type: null },"#,
            r#""modelModifiers": {}"#,
        ]
    );
}
