//! `<script setup lang="tsx">` may hold JSX. Parsing it with the TypeScript
//! grammar failed, which dropped every macro and setup binding.
use crate::compile_sfc;
use crate::types::SfcCompileOptions;
use crate::{SfcParseOptions, parse_sfc};

fn compile(source: &str) -> vize_carton::String {
    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("parse");
    compile_sfc(&descriptor, SfcCompileOptions::default())
        .expect("compile")
        .code
}

#[test]
fn tsx_setup_keeps_macros_and_bindings_beside_jsx() {
    let source = r#"<script setup lang="tsx">
import { ref } from 'vue'
const count = ref(0)
const emit = defineEmits<{ inc: [n: number] }>()
const render = () => <button onClick={() => emit('inc', count.value)}>{count.value}</button>
</script>
<template><div @click="count++">{{ count }}<component :is="render" /></div></template>"#;
    assert_eq!(
        compile(source),
        r##"import { resolveDynamicComponent as _resolveDynamicComponent, toDisplayString as _toDisplayString, openBlock as _openBlock, createElementBlock as _createElementBlock, createBlock as _createBlock, createTextVNode as _createTextVNode } from "vue"

import { ref } from 'vue'


export default {
  __name: 'anonymous',
  emits: ["inc"],
  setup(__props, { emit: __emit }) {

const emit = __emit
const count = ref(0);
const render = () => <button onClick={() => emit("inc", count.value)}>{count.value}</button>;

return (_ctx, _cache) => {
  return (_openBlock(), _createElementBlock("div", {
    onClick: _cache[0] || (_cache[0] = $event => (count.value++))
  }, [
    _createTextVNode(_toDisplayString(count.value), 1 /* TEXT */),
    (_openBlock(), _createBlock(_resolveDynamicComponent(render)))
  ]))
}
}

}"##
    );
}

#[test]
fn tsx_setup_rewrites_destructured_props_inside_jsx() {
    let source = r#"<script setup lang="tsx">
const { title = 'x', items } = defineProps<{ title?: string; items: string[] }>()
const view = <ul title={title}>{items.map((item) => <li>{item}</li>)}{...items}</ul>
</script>
<template><component :is="() => view" /></template>"#;
    assert_eq!(
        compile(source),
        r##"import { resolveDynamicComponent as _resolveDynamicComponent, openBlock as _openBlock, createBlock as _createBlock, unref as _unref } from "vue"


export default {
  __name: 'anonymous',
  props: {
    title: { type: String, required: false, default: "x" },
    items: { type: Array, required: true }
  },
  setup(__props) {

const view = <ul title={__props.title}>{__props.items.map((item) => <li>{item}</li>)}{...__props.items}</ul>;

return (_ctx, _cache) => {
  return (_openBlock(), _createBlock(_resolveDynamicComponent(() => _unref(view))))
}
}

}"##
    );
}

#[test]
fn tsx_setup_strips_types_and_keeps_jsx_in_javascript_output() {
    let source = r#"<script setup lang="tsx">
import { ref } from 'vue'
const label = ref<string>('a')
const Item = (props: { text: string }): JSX.Element => <b>{props.text}</b>
</script>
<template><Item :text="label" /></template>"#;
    assert_eq!(
        compile(source),
        r##"import { openBlock as _openBlock, createBlock as _createBlock } from "vue"

import { ref } from 'vue'


export default {
  __name: 'anonymous',
  setup(__props) {

const label = ref("a");
const Item = (props) => <b>{props.text}</b>;

return (_ctx, _cache) => {
  return (_openBlock(), _createBlock(Item, { text: label.value }, null, 8 /* PROPS */, ["text"]))
}
}

}"##
    );
}

#[test]
fn jsx_setup_rewrites_props_but_not_shadowing_jsx_parameters() {
    let source = r#"<script setup lang="jsx">
const { title } = defineProps({ title: String })
const Row = ({ title: label }) => <b title={title}>{label}</b>
const Inner = (title) => <i>{title}</i>
</script>
<template><Row /><Inner /></template>"#;
    assert_eq!(
        compile(source),
        r##"import { createVNode as _createVNode, openBlock as _openBlock, createElementBlock as _createElementBlock, Fragment as _Fragment } from "vue"


export default {
  __name: 'anonymous',
  props: { title: String },
  setup(__props) {

const Row = ({ title: label }) => <b title={__props.title}>{label}</b>
const Inner = (title) => <i>{title}</i>

return (_ctx, _cache) => {
  return (_openBlock(), _createElementBlock(_Fragment, null, [
    _createVNode(Row),
    _createVNode(Inner)
  ], 64 /* STABLE_FRAGMENT */))
}
}

}"##
    );
}
