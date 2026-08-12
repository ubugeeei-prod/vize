//! Art variants whose cursor offset has no generated counterpart must still be
//! answered from the authored SFC instead of returning nothing.

use std::{fs, sync::Arc};

use tower_lsp::lsp_types::{GotoDefinitionResponse, Hover, HoverContents, Url};
use vize_canon::CorsaBridge;

use crate::{
    ide::{DefinitionService, HoverService, IdeContext},
    server::ServerState,
    virtual_code::{ArtCursorPosition, ArtVariantInfo, BlockType},
};

const ART_SOURCE: &str = r#"<script setup lang="ts">
import { ref } from 'vue'

const primaryLabel = ref('primary')
const secondaryLabel = ref('secondary')
</script>

<art title="Button" component="./Child.vue">
  <variant name="Primary" default>
    <Child>{{ primaryLabel }}</Child>
  </variant>
  <variant name="Secondary">
    <Child>{{ secondaryLabel }}</Child>
  </variant>
</art>
"#;

/// Open the art fixture and place the cursor on the non-default variant's
/// expression, which the typed art template does not map to generated code.
fn open_secondary_variant(state: &ServerState, dir: &std::path::Path) -> (Url, usize) {
    let src = dir.join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(
        src.join("Child.vue"),
        "<script setup lang=\"ts\"></script>\n<template><button /></template>\n",
    )
    .unwrap();
    let art_path = src.join("Button.art.vue");
    fs::write(&art_path, ART_SOURCE).unwrap();
    let uri = Url::from_file_path(&art_path).unwrap();
    state.set_workspace_root(dir.to_path_buf());
    state.documents.open(
        uri.clone(),
        ART_SOURCE.to_string(),
        1,
        "art-vue".to_string(),
    );
    state.update_virtual_docs(&uri, ART_SOURCE);
    let offset = ART_SOURCE.rfind("secondaryLabel").unwrap() + "secondaryLabel".len();
    (uri, offset)
}

fn variant_info(ctx: &IdeContext<'_>) -> ArtVariantInfo {
    match ctx.block_type {
        Some(BlockType::Art(ArtCursorPosition::VariantTemplate(ref info))) => *info,
        ref other => panic!("expected an art variant context, got {other:?}"),
    }
}

fn hover_text(hover: Hover) -> String {
    match hover.contents {
        HoverContents::Markup(markup) => markup.value,
        other => panic!("expected markdown hover contents, got {other:?}"),
    }
}

#[test]
fn art_variant_hover_answers_from_the_authored_template_without_a_generated_offset() {
    let dir = tempfile::tempdir().unwrap();
    let state = ServerState::new();
    let (uri, offset) = open_secondary_variant(&state, dir.path());
    let ctx = IdeContext::new(&state, &uri, offset).unwrap();
    let info = variant_info(&ctx);

    let template = ctx
        .virtual_docs
        .as_ref()
        .and_then(|documents| documents.art_template(info.variant_index))
        .expect("typed art template");
    assert!(
        template.source_map.to_generated(offset as u32).is_none(),
        "fixture must exercise the unmapped variant offset",
    );

    let hover = crate::runtime::block_on(HoverService::hover_with_corsa(
        &ctx,
        Some(Arc::new(CorsaBridge::new())),
    ))
    .expect("authored hover fallback");
    let text = hover_text(hover);
    assert!(text.contains("secondaryLabel"), "{text}");
    assert!(text.contains("template scope"), "{text}");
}

#[test]
fn art_variant_definition_answers_from_the_authored_template_without_a_generated_offset() {
    let dir = tempfile::tempdir().unwrap();
    let state = ServerState::new();
    let (uri, offset) = open_secondary_variant(&state, dir.path());
    let ctx = IdeContext::new(&state, &uri, offset).unwrap();

    let definition = crate::runtime::block_on(DefinitionService::definition_with_corsa(
        &ctx,
        Some(Arc::new(CorsaBridge::new())),
    ))
    .expect("authored definition fallback");
    let location = match definition {
        GotoDefinitionResponse::Scalar(location) => location,
        GotoDefinitionResponse::Array(mut locations) if !locations.is_empty() => {
            locations.remove(0)
        }
        other => panic!("expected an authored location, got {other:?}"),
    };
    assert_eq!(location.uri, uri);
    assert_eq!(location.range.start.line, 4);
}
