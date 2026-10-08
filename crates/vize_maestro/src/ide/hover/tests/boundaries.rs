//! Existing hover cursor and petite-vue boundary controls.

use super::*;

#[test]
fn test_hover_petite_vue_v_scope_binding_in_expression() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("index.html");
    let source = r#"<script src="https://unpkg.com/petite-vue" defer init></script>
<div v-scope="{ count: 0, msg: 'x' }">{{ count }}</div>
"#;
    fs::write(&source_path, source).unwrap();

    let uri = Url::from_file_path(&source_path).unwrap();
    let state = ServerState::new();
    state
        .documents
        .open(uri.clone(), source.to_string(), 1, "html".to_string());
    state.update_virtual_docs(&uri, source);

    // Cursor inside the `{{ count }}` interpolation expression.
    let offset = source.find("{{ count").unwrap() + "{{ co".len();
    let ctx = IdeContext::new(&state, &uri, offset).unwrap();
    let hover = HoverService::hover(&ctx).expect("v-scope binding should produce hover");
    let value = hover_markdown(hover);

    assert!(value.contains("count"), "got {value:?}");
    assert!(
        value.contains("petite-vue scope binding"),
        "hover should label the binding as a petite-vue scope binding; got {value:?}"
    );
}

#[test]
fn test_hover_petite_vue_v_scope_binding_does_not_leak_to_sibling() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("index.html");
    let source = r#"<script src="https://unpkg.com/petite-vue" defer init></script>
<span v-scope="{ count: 0 }">{{ count }}</span><p>{{ count }}</p>
"#;
    fs::write(&source_path, source).unwrap();

    let uri = Url::from_file_path(&source_path).unwrap();
    let state = ServerState::new();
    state
        .documents
        .open(uri.clone(), source.to_string(), 1, "html".to_string());
    state.update_virtual_docs(&uri, source);

    // Cursor inside the sibling `<p>` interpolation, outside the v-scope subtree.
    let p_start = source.find("<p>").unwrap();
    let offset = source.split_at(p_start).1.find("{{ count").unwrap() + p_start + "{{ co".len();
    let ctx = IdeContext::new(&state, &uri, offset).unwrap();
    let hover = HoverService::hover(&ctx);

    // Either no hover, or a generic template-expression hover — but never the
    // petite-vue scope binding hover, since the binding is out of scope here.
    if let Some(hover) = hover {
        let value = hover_markdown(hover);
        assert!(
            !value.contains("petite-vue scope binding"),
            "v-scope binding must not leak to a sibling subtree; got {value:?}"
        );
    }
}

#[test]
fn test_hover_supports_art_variant_binding_at_identifier_boundary() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("HoverButton.art.vue");
    let source = r#"<script setup lang="ts">
const primaryLabel = ref('primary')
const secondaryLabel = ref('secondary')
</script>

<art title="Button" component="./Button.vue">
  <variant name="Primary" default>
    <Button>{{ primaryLabel }}</Button>
  </variant>
  <variant name="Secondary">
    <Button>{{ secondaryLabel }}</Button>
  </variant>
</art>
"#;
    fs::write(&source_path, source).unwrap();

    let uri = Url::from_file_path(&source_path).unwrap();
    let state = ServerState::new();
    state
        .documents
        .open(uri.clone(), source.to_string(), 1, "art-vue".to_string());
    state.update_virtual_docs(&uri, source);

    let offset = source.rfind("secondaryLabel").unwrap() + "secondaryLabel".len();
    let ctx = IdeContext::new(&state, &uri, offset).unwrap();
    let hover = HoverService::hover(&ctx).unwrap();
    let value = hover_markdown(hover);

    assert!(value.contains("secondaryLabel"));
    assert!(value.contains("_Template expression_"));
}

#[cfg(feature = "native")]
#[test]
fn test_hover_with_corsa_fallback_supports_identifier_boundaries() {
    crate::runtime::block_on(async {
        let dir = tempfile::tempdir().unwrap();
        let source_path = dir.path().join("HoverBoundary.vue");
        let source = r#"<script setup lang="ts">
const count = ref(0)
</script>

<template>
  {{ count }}
</template>
"#;
        fs::write(&source_path, source).unwrap();

        let uri = Url::from_file_path(&source_path).unwrap();
        let state = ServerState::new();
        state
            .documents
            .open(uri.clone(), source.to_string(), 1, "vue".to_string());
        state.update_virtual_docs(&uri, source);

        let offset = source.rfind("count").unwrap() + "count".len();
        let ctx = IdeContext::new(&state, &uri, offset).unwrap();
        let hover = HoverService::hover_with_corsa(&ctx, None).await.unwrap();
        let value = hover_markdown(hover);

        assert!(value.contains("count"));
        assert!(value.contains("_Template binding_"));
    });
}

#[cfg(feature = "native")]
#[test]
fn test_hover_with_corsa_fallback_supports_directive_boundaries() {
    crate::runtime::block_on(async {
        let dir = tempfile::tempdir().unwrap();
        let source_path = dir.path().join("HoverDirective.vue");
        let source = r#"<template>
  <div v-if="visible" />
</template>
"#;
        fs::write(&source_path, source).unwrap();

        let uri = Url::from_file_path(&source_path).unwrap();
        let state = ServerState::new();
        state
            .documents
            .open(uri.clone(), source.to_string(), 1, "vue".to_string());
        state.update_virtual_docs(&uri, source);

        let offset = source.find("v-if").unwrap() + "v-if".len();
        let ctx = IdeContext::new(&state, &uri, offset).unwrap();
        let hover = HoverService::hover_with_corsa(&ctx, None).await.unwrap();
        let value = hover_markdown(hover);

        assert!(value.contains("**v-if**"));
        assert!(value.contains("Conditionally render"));
    });
}
