//! Type-aware `textDocument/declaration` for authored Vue files.

#![cfg_attr(
    feature = "native",
    expect(
        clippy::disallowed_types,
        reason = "tower-lsp lsp_types take std String/HashMap values, built with to_string/format!"
    )
)]

#[cfg(feature = "native")]
use std::sync::Arc;

#[cfg(feature = "native")]
use tower_lsp::lsp_types::GotoDefinitionResponse;
#[cfg(feature = "native")]
use vize_canon::CorsaBridge;

#[cfg(feature = "native")]
use super::{IdeContext, TypeDefinitionService, corsa_support};
#[cfg(feature = "native")]
use crate::virtual_code::{ArtCursorPosition, ArtVariantInfo, BlockType, VirtualDocument};

/// Checker-backed declaration navigation service.
pub struct DeclarationService;

#[cfg(feature = "native")]
impl DeclarationService {
    /// Resolve declaration locations through Corsa's standard LSP surface and
    /// map every returned location back onto authored source.
    pub async fn declaration_with_corsa(
        ctx: &IdeContext<'_>,
        corsa_bridge: Option<Arc<CorsaBridge>>,
    ) -> Option<GotoDefinitionResponse> {
        let bridge = corsa_bridge?;
        if !bridge.is_initialized() {
            return None;
        }

        match ctx.block_type? {
            BlockType::Template | BlockType::Script | BlockType::ScriptSetup
                if !ctx.uri.path().ends_with(".art.vue") =>
            {
                Self::declaration_in_canonical_sfc(ctx, &bridge).await
            }
            BlockType::Script => Self::declaration_in_split_script(ctx, &bridge, false).await,
            BlockType::ScriptSetup => Self::declaration_in_split_script(ctx, &bridge, true).await,
            BlockType::Art(ArtCursorPosition::VariantTemplate(ref info)) => {
                Self::declaration_in_art_variant(ctx, &bridge, info).await
            }
            BlockType::Style(_) if !ctx.uri.path().ends_with(".art.vue") => {
                TypeDefinitionService::convert_locations(
                    corsa_support::style_bind_locations(
                        ctx,
                        &bridge,
                        corsa_support::StyleBindQuery::Declaration,
                    )
                    .await,
                )
            }
            BlockType::Template | BlockType::Style(_) | BlockType::Art(_) => None,
        }
    }

    async fn declaration_in_canonical_sfc(
        ctx: &IdeContext<'_>,
        bridge: &CorsaBridge,
    ) -> Option<GotoDefinitionResponse> {
        let document = corsa_support::open_canonical_virtual_project_document(ctx, bridge).await?;
        let (line, character) =
            corsa_support::canonical_source_offset_to_position(&document, ctx.offset)?;
        let locations = Self::declaration_or_definition_locations(
            bridge,
            &document.request_uri,
            line,
            character,
        )
        .await?;
        let mapped =
            corsa_support::map_canonical_corsa_locations(ctx, &document, locations.clone());
        if !mapped.is_empty() {
            return TypeDefinitionService::convert_locations(mapped);
        }
        // A template ref-unwrap shadow (`var name: __U<__R_name>`) has no
        // authored span; its semantic link leads to the setup binding capture,
        // whose declaration is the authored one.
        let mut followed = Vec::new();
        for location in &locations {
            for position in
                corsa_support::linked_semantic_positions(&document, &location.uri, &location.range)
            {
                if let Some(found) = Self::declaration_or_definition_locations(
                    bridge,
                    &position.request_uri,
                    position.line,
                    position.character,
                )
                .await
                {
                    followed.extend(found);
                }
            }
        }
        TypeDefinitionService::convert_locations(corsa_support::map_canonical_corsa_locations(
            ctx, &document, followed,
        ))
    }

    async fn declaration_in_split_script(
        ctx: &IdeContext<'_>,
        bridge: &CorsaBridge,
        is_setup: bool,
    ) -> Option<GotoDefinitionResponse> {
        let virtual_docs = ctx.virtual_docs.as_ref()?;
        let document = if is_setup {
            virtual_docs.script_setup.as_ref()
        } else {
            virtual_docs.script.as_ref()
        }?;
        let generated_offset =
            super::hover::HoverService::sfc_to_virtual_ts_script_offset(ctx, ctx.offset)?;
        Self::declaration_in_virtual_document(
            ctx,
            bridge,
            document,
            generated_offset,
            corsa_support::script_request_path(ctx.uri, is_setup),
        )
        .await
    }

    async fn declaration_in_art_variant(
        ctx: &IdeContext<'_>,
        bridge: &CorsaBridge,
        info: &ArtVariantInfo,
    ) -> Option<GotoDefinitionResponse> {
        let document = ctx
            .virtual_docs
            .as_ref()?
            .art_template(info.variant_index)?;
        let generated_offset = document.source_map.to_generated_for(
            ctx.offset,
            crate::virtual_code::ProjectionFeatures::DEFINITION,
        )?;
        Self::declaration_in_virtual_document(
            ctx,
            bridge,
            document,
            generated_offset,
            corsa_support::art_template_request_path(ctx.uri, info.variant_index),
        )
        .await
    }

    async fn declaration_in_virtual_document(
        ctx: &IdeContext<'_>,
        bridge: &CorsaBridge,
        document: &VirtualDocument,
        generated_offset: usize,
        request_path: vize_l0::String,
    ) -> Option<GotoDefinitionResponse> {
        let (line, character) = super::offset_to_position(&document.content, generated_offset);
        let uri = bridge
            .open_or_update_virtual_document(&request_path, &document.content)
            .await
            .ok()?;
        let locations =
            Self::declaration_or_definition_locations(bridge, &uri, line, character).await?;
        let locations = corsa_support::map_corsa_locations(ctx, locations);
        TypeDefinitionService::convert_locations(locations)
    }

    async fn declaration_or_definition_locations(
        bridge: &CorsaBridge,
        uri: &str,
        line: u32,
        character: u32,
    ) -> Option<Vec<vize_canon::LspLocation>> {
        match bridge.declaration(uri, line, character).await {
            Ok(locations) if !locations.is_empty() => Some(locations),
            Ok(_) | Err(_) => bridge.definition(uri, line, character).await.ok(),
        }
    }
}

#[cfg(all(test, feature = "native"))]
mod tests {
    #![expect(
        clippy::expect_used,
        clippy::panic,
        clippy::string_slice,
        reason = "tests assert by panicking"
    )]
    use std::{fs, sync::Arc};

    use tower_lsp::lsp_types::{GotoDefinitionResponse, Location, Url};
    use vize_canon::{CorsaBridge, CorsaBridgeConfig};

    use super::DeclarationService;
    use crate::{ide::IdeContext, server::ServerState};

    #[test]
    fn canonical_declaration_maps_template_binding_to_authored_const() {
        crate::runtime::block_on(async {
            let Some(tsgo_path) = resolve_tsgo_binary() else {
                return;
            };
            let project = tempfile::TempDir::new().expect("temp project");
            let src = project.path().join("src");
            fs::create_dir_all(&src).expect("src");
            fs::write(
                project.path().join("tsconfig.json"),
                r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "allowImportingTsExtensions": true,
    "noEmit": true
  },
  "include": ["src/**/*"]
}"#,
            )
            .expect("tsconfig");

            let source = r#"<script setup lang="ts">
interface Product {
  name: string
}
const product = {} as Product
</script>
<template>{{ product.name }}</template>
"#;
            let path = src.join("Card.vue");
            fs::write(&path, source).expect("source");
            let uri = Url::from_file_path(&path).expect("source uri");
            let state = ServerState::new();
            state.set_workspace_root(project.path().to_path_buf());
            state
                .documents
                .open(uri.clone(), source.to_string(), 1, "vue".to_string());
            state.update_virtual_docs(&uri, source);
            let offset = source
                .rfind("product.name")
                .expect("template product binding")
                + "pro".len();
            let ctx = IdeContext::new(&state, &uri, offset).expect("context");
            let bridge = Arc::new(CorsaBridge::with_config(CorsaBridgeConfig {
                corsa_path: Some(tsgo_path),
                working_dir: Some(project.path().to_path_buf()),
                timeout_ms: 30_000,
                ..Default::default()
            }));
            bridge.spawn().await.expect("tsgo session");

            let response = DeclarationService::declaration_with_corsa(&ctx, Some(bridge.clone()))
                .await
                .expect("declaration");
            bridge.shutdown().await.expect("shutdown");

            let location = scalar_location(response);
            assert_eq!(location.uri, uri);
            assert_eq!(authored_text(source, &location), "product");
            assert_eq!(
                location.range.start.line, 4,
                "declaration should point at the script const, not the template use: {location:#?}",
            );
            assert!(
                !location.uri.path().ends_with(".vue.ts"),
                "generated URI leaked: {location:#?}",
            );
        });
    }

    fn scalar_location(response: GotoDefinitionResponse) -> Location {
        match response {
            GotoDefinitionResponse::Scalar(location) => location,
            GotoDefinitionResponse::Array(mut locations) => {
                assert_eq!(locations.len(), 1, "{locations:#?}");
                locations.remove(0)
            }
            GotoDefinitionResponse::Link(_) => panic!("expected location result"),
        }
    }

    fn authored_text<'a>(source: &'a str, location: &Location) -> &'a str {
        let start = crate::ide::position_to_offset(
            source,
            location.range.start.line,
            location.range.start.character,
        )
        .expect("source start");
        let end = crate::ide::position_to_offset(
            source,
            location.range.end.line,
            location.range.end.character,
        )
        .expect("source end");
        &source[start..end]
    }

    fn resolve_tsgo_binary() -> Option<std::path::PathBuf> {
        crate::test_support::tsgo()
    }
}
