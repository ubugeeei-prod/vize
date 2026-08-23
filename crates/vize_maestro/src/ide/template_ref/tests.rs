use super::*;
use tower_lsp::lsp_types::Url;

use crate::server::ServerState;

#[test]
fn static_ref_value_finds_authored_value_only() {
    let source = r#"<template><button data-ref="skip" :ref="dynamic" ref="button" /></template>"#;
    let offset = source.find("button\"").unwrap() + "button".len();
    let value = static_ref_value_at_offset(source, offset).unwrap();

    assert_eq!(value.ref_name, "button");
    assert_eq!(&source[value.start..value.end], "button");
    assert!(static_ref_value_at_offset(source, source.find("dynamic").unwrap()).is_none());
    assert!(static_ref_value_at_offset(source, source.find("skip").unwrap()).is_none());
}

#[test]
fn target_maps_static_ref_to_use_template_ref_binding() {
    let source = r#"<script setup lang="ts">
import { useTemplateRef } from 'vue'
const el = useTemplateRef<HTMLButtonElement>('button')
</script>

<template><button ref="button" /></template>
"#;
    let state = ServerState::new();
    let uri = Url::parse("file:///workspace/App.vue").unwrap();
    state
        .documents
        .open(uri.clone(), source.to_string(), 1, "vue".to_string());
    state.update_virtual_docs(&uri, source);
    let offset = source.rfind("button\"").unwrap() + "button".len();
    let ctx = IdeContext::new(&state, &uri, offset).unwrap();
    let target = target_at_offset(&ctx).unwrap();

    assert_eq!(target.ref_name, "button");
    assert_eq!(target.binding_name, "el");
    assert_eq!(&source[target.binding_start..target.binding_end], "el");
    assert_eq!(&source[target.value_start..target.value_end], "button");
}

#[test]
fn target_rejects_missing_use_template_ref_binding() {
    let source = r#"<script setup lang="ts">
const el = 1
</script>
<template><button ref="button" /></template>
"#;
    let state = ServerState::new();
    let uri = Url::parse("file:///workspace/App.vue").unwrap();
    state
        .documents
        .open(uri.clone(), source.to_string(), 1, "vue".to_string());
    state.update_virtual_docs(&uri, source);
    let offset = source.rfind("button\"").unwrap() + "button".len();
    let ctx = IdeContext::new(&state, &uri, offset).unwrap();

    assert!(target_at_offset(&ctx).is_none());
}
