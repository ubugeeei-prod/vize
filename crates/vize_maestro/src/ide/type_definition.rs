//! Type-aware `textDocument/typeDefinition` for authored Vue and JSX files.

#![cfg_attr(
    feature = "native",
    expect(
        clippy::disallowed_types,
        reason = "the Corsa bridge is shared across LSP requests as `Arc<CorsaBridge>`"
    )
)]

#[cfg(feature = "native")]
use std::sync::Arc;

#[cfg(feature = "native")]
use tower_lsp::lsp_types::{GotoDefinitionResponse, Location};
#[cfg(feature = "native")]
use vize_canon::CorsaBridge;

#[cfg(feature = "native")]
use super::IdeContext;
#[cfg(feature = "native")]
use super::corsa_support;
#[cfg(feature = "native")]
use crate::virtual_code::{ArtCursorPosition, ArtVariantInfo, BlockType, VirtualDocument};

/// Checker-backed type-definition service.
pub struct TypeDefinitionService;

#[cfg(feature = "native")]
impl TypeDefinitionService {
    /// Resolve type definitions through Corsa's standard LSP surface and map
    /// every returned location back onto authored source.
    pub async fn type_definition_with_corsa(
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
                Self::type_definition_in_canonical_sfc(ctx, &bridge).await
            }
            BlockType::Script => Self::type_definition_in_split_script(ctx, &bridge, false).await,
            BlockType::ScriptSetup => {
                Self::type_definition_in_split_script(ctx, &bridge, true).await
            }
            BlockType::Art(ArtCursorPosition::VariantTemplate(ref info)) => {
                Self::type_definition_in_art_variant(ctx, &bridge, info).await
            }
            BlockType::Template | BlockType::Style(_) | BlockType::Art(_) => None,
        }
    }

    async fn type_definition_in_canonical_sfc(
        ctx: &IdeContext<'_>,
        bridge: &CorsaBridge,
    ) -> Option<GotoDefinitionResponse> {
        let document = corsa_support::open_canonical_virtual_project_document(ctx, bridge).await?;
        let (line, character) =
            corsa_support::canonical_source_offset_to_position(&document, ctx.offset)?;
        let locations = bridge
            .type_definition(&document.request_uri, line, character)
            .await
            .ok()?;
        let locations = corsa_support::map_canonical_corsa_locations(ctx, &document, locations);
        Self::convert_locations(locations)
    }

    async fn type_definition_in_split_script(
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
        Self::type_definition_in_virtual_document(
            ctx,
            bridge,
            document,
            generated_offset,
            corsa_support::script_request_path(ctx.uri, is_setup),
        )
        .await
    }

    async fn type_definition_in_art_variant(
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
        Self::type_definition_in_virtual_document(
            ctx,
            bridge,
            document,
            generated_offset,
            corsa_support::art_template_request_path(ctx.uri, info.variant_index),
        )
        .await
    }

    async fn type_definition_in_virtual_document(
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
        let locations = bridge.type_definition(&uri, line, character).await.ok()?;
        let locations = corsa_support::map_corsa_locations(ctx, locations);
        Self::convert_locations(locations)
    }

    pub(in crate::ide) fn convert_locations(
        locations: Vec<Location>,
    ) -> Option<GotoDefinitionResponse> {
        match locations.as_slice() {
            [] => None,
            [location] => Some(GotoDefinitionResponse::Scalar(location.clone())),
            _ => Some(GotoDefinitionResponse::Array(locations)),
        }
    }
}

#[cfg(test)]
#[cfg(feature = "native")]
#[expect(clippy::string_slice, reason = "tests assert by panicking")]
mod tests {
    use std::{fs, sync::Arc};

    use tower_lsp::lsp_types::{GotoDefinitionResponse, Location, Url};
    use vize_canon::{CorsaBridge, CorsaBridgeConfig};

    use super::TypeDefinitionService;
    use crate::{ide::IdeContext, server::ServerState};

    #[test]
    fn canonical_type_definition_maps_template_binding_to_authored_interface() {
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

            let response =
                TypeDefinitionService::type_definition_with_corsa(&ctx, Some(bridge.clone()))
                    .await
                    .expect("type definition");
            bridge.shutdown().await.expect("shutdown");

            let location = scalar_location(response);
            assert_eq!(location.uri, uri);
            assert_eq!(authored_text(source, &location), "Product");
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
        if std::env::var_os("VIZE_TEST_DISABLE_TSGO").is_some() {
            return None;
        }
        let workspace_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(std::path::Path::parent)?;
        vize_carton::corsa_resolver::resolve_corsa_executable(
            vize_carton::corsa_resolver::CorsaResolveRequest {
                project_root: Some(workspace_root),
                ..Default::default()
            },
        )
        .ok()
    }
}
