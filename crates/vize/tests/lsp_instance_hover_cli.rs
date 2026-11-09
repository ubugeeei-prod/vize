//! Hovers over a component tag, its template ref and `InstanceType` name the
//! public instance: the constructor's bound branch keeps `NoInfer` (#12431),
//! which these hovers must never print.
#![cfg(test)]

use crate::support::lsp_vue_project::Fixture;
use serde_json::{Map, Value, json};

const PARENT: &str = r#"<script setup lang="ts">
import { useTemplateRef } from 'vue'
import Child from './Child.vue'
const child = useTemplateRef<InstanceType<typeof Child>>('child')
type ChildInstance = InstanceType<typeof Child>
declare const instance: ChildInstance
console.log(child.value?.$props.title, instance.$props.title)
</script>

<template>
  <Child ref="child" title="t" />
</template>
"#;

const CHILD: &str = r#"<script setup lang="ts">
defineProps<{ title: string }>()
</script>

<template><span>{{ title }}</span></template>
"#;

#[test]
fn instance_hovers_name_the_public_instance() {
    let expected: Value =
        serde_json::from_str(include_str!("fixtures/instance_hover/hover.vize-lsp.json")).unwrap();
    let mut project =
        Fixture::new_with_vue_component_project(PARENT, "Parent.vue", &[("Child.vue", CHILD)]);
    project.open(PARENT);
    let uri = project.uri.clone();
    let mut actual = Map::new();
    for (case, needle) in [
        ("tag", "Child ref"),
        ("template ref", "child = useTemplateRef"),
        ("InstanceType", "instance: ChildInstance"),
    ] {
        let hover =
            project.request_file_with("textDocument/hover", &uri, PARENT, needle, json!({}));
        actual.insert(case.into(), hover);
    }
    project.shutdown();
    let actual = Value::Object(actual);
    if actual != expected {
        // The whole packet, so it can be reviewed and pinned at once.
        eprintln!("{}", serde_json::to_string_pretty(&actual).unwrap());
    }
    assert_eq!(actual, expected);
}
