//! An SFC type parameter may share a name with anything the generator writes
//! inside that parameter's scope (#11485). Each child names its parameter
//! after one such name; consumers must infer it exactly as they infer `T`.

use super::*;
use vize_carton::cstr;

/// Generator-declared aliases, a shared helper and lib types it applies
/// inside the component's generic signatures.
const PARAMETER_NAMES: &[&str] = &[
    "T",
    "Props",
    "Emits",
    "Exposed",
    "Slots",
    "$Slots",
    "$GenericContext",
    "$StaticEventMap",
    "$AuthoredEventMap",
    "$PublicEmit",
    "Record",
    "Partial",
    "Pick",
    "Awaited",
    "ReturnType",
];

fn child(name: &str) -> std::string::String {
    format!(
        r#"<script setup lang="ts" generic="{name} extends string | number">
const props = defineProps<{{ v: {name}; list?: {name}[] }}>()
const emit = defineEmits<{{ pick: [value: {name}] }}>()
defineSlots<{{ default(p: {{ v: {name} }}): any }}>()
defineExpose({{ get: (): {name} => props.v }})
</script>
<template><div @click="emit('pick', props.v)"><slot :v="props.v" /></div></template>
"#
    )
}

fn component(index: usize) -> std::string::String {
    format!("N{index}")
}

fn consumer() -> std::string::String {
    let mut imports = std::string::String::new();
    let mut refs = std::string::String::new();
    let mut template = std::string::String::new();
    for index in 0..PARAMETER_NAMES.len() {
        let name = component(index);
        imports.push_str(&format!("import {name} from './{name}.vue'\n"));
        refs.push_str(&format!(
            "const r{index} = useTemplateRef('r{index}')\nonMounted(() => takesString(r{index}.value!.get()))\n"
        ));
        template.push_str(&format!(
            r#"  <{name} ref="r{index}" v="a" @pick="(x) => takesString(x)" v-slot="{{ v }}">{{{{ takesString(v) }}}}</{name}>
  <{name} v="a" @pick="(x) => takesNumber(x)" />
  <{name} :v="1" :list="['b']" />
"#
        ));
    }
    format!(
        r#"<script setup lang="ts">
import {{ onMounted, useTemplateRef }} from 'vue'
{imports}declare function takesNumber(n: number): void
declare function takesString(s: string): void
{refs}</script>
<template>
{template}</template>
"#
    )
}

#[test]
fn generic_parameters_named_like_generated_types_keep_consumer_inference() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let children: Vec<_> = PARAMETER_NAMES
        .iter()
        .enumerate()
        .map(|(index, name)| (format!("src/{}.vue", component(index)), child(name)))
        .collect();
    let consumer = consumer();
    let mut files: Vec<(&str, &str)> = children
        .iter()
        .map(|(path, source)| (path.as_str(), source.as_str()))
        .collect();
    files.push(("src/Use.vue", consumer.as_str()));
    let project_root = create_project_case("generic-parameter-generated-names", &files);

    let Some(snapshot) = snapshot_project_diagnostics(&project_root) else {
        let _ = std::fs::remove_dir_all(&project_root);
        return;
    };

    // Children are clean; each consumer row reports exactly its two negative
    // controls: the listener payload (string to number) and the list item.
    let template = consumer
        .lines()
        .position(|line| line == "<template>")
        .expect("consumer template");
    let mut expected = Vec::new();
    for index in 0..PARAMETER_NAMES.len() {
        let first_row = template + 2 + index * 3;
        expected.push(cstr!("src/Use.vue:{}:TS2345", first_row + 1));
        expected.push(cstr!("src/Use.vue:{}:TS2322", first_row + 2));
    }
    let mut actual: Vec<_> = snapshot
        .iter()
        .map(|(file, code, message)| {
            let line = message.split(':').next().unwrap_or_default();
            cstr!("{file}:{line}:TS{}", code.unwrap_or_default())
        })
        .collect();
    actual.sort();
    expected.sort();
    assert_eq!(
        actual, expected,
        "every parameter name must infer like `T`: {snapshot:#?}"
    );
    let _ = std::fs::remove_dir_all(&project_root);
}
