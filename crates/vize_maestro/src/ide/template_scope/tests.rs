use tower_lsp::lsp_types::Url;

use super::{TemplateScopeBindingKind, v_for_binding_at};
use crate::{ide::IdeContext, server::ServerState};

#[test]
fn finds_v_for_value_key_and_index_aliases_from_body_usage() {
    let source = r#"<script setup>
const rows = []
</script>
<template>
  <li v-for="(row, key, index) in rows" :key="row.id">
    {{ row.name }} {{ key }} {{ index }}
  </li>
</template>
"#;
    let uri = Url::parse("file:///ForAliases.vue").unwrap();
    let state = ServerState::new();
    state
        .documents
        .open(uri.clone(), source.to_string(), 1, "vue".to_string());
    state.update_virtual_docs(&uri, source);

    for (usage, declaration, kind) in [
        ("row.name", "(row", TemplateScopeBindingKind::Value),
        ("{{ key }}", " key", TemplateScopeBindingKind::Key),
        ("{{ index }}", " index", TemplateScopeBindingKind::Index),
    ] {
        let usage_offset =
            source.rfind(usage).unwrap() + usage.find(|c: char| c.is_alphanumeric()).unwrap();
        let word = usage
            .trim_matches(|c: char| !c.is_alphanumeric())
            .split('.')
            .next()
            .unwrap();
        let ctx = IdeContext::new(&state, &uri, usage_offset).unwrap();
        let binding = v_for_binding_at(&ctx, word).unwrap();
        assert_eq!(binding.kind, kind);
        assert_eq!(&source[binding.start..binding.end], word);
        assert_eq!(binding.start, source.find(declaration).unwrap() + 1);
    }
}

#[test]
fn prefers_the_nearest_nested_v_for_alias() {
    let source = r#"<script setup>
const outer = []
const inner = []
</script>
<template>
  <div v-for="item in outer">
    <span v-for="item in inner">{{ item }}</span>
  </div>
</template>
"#;
    let uri = Url::parse("file:///NestedFor.vue").unwrap();
    let state = ServerState::new();
    state
        .documents
        .open(uri.clone(), source.to_string(), 1, "vue".to_string());
    state.update_virtual_docs(&uri, source);

    let usage_offset = source.rfind("{{ item }}").unwrap() + "{{ ".len();
    let ctx = IdeContext::new(&state, &uri, usage_offset).unwrap();
    let binding = v_for_binding_at(&ctx, "item").unwrap();

    assert_eq!(
        binding.start,
        source.rfind("item in inner").unwrap(),
        "nested v-for aliases must shadow outer aliases",
    );
}
