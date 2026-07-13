use tower_lsp::lsp_types::{Location, Range, Url};
use vize_carton::cstr;

use crate::ide::IdeContext;
use crate::ide::diagnostics::VirtualTsResult;

use super::canonical::{CanonicalVirtualDocument, map_lsp_range_to_source};

pub(super) fn map_location(
    ctx: &IdeContext<'_>,
    location: &vize_canon::LspLocation,
) -> Option<Location> {
    let parsed = Url::parse(&location.uri).ok()?;
    let path = parsed.to_file_path().ok()?;
    let file_name = path.file_name()?.to_str()?;
    let vue_file_name = file_name
        .strip_suffix(".tsx")
        .or_else(|| file_name.strip_suffix(".ts"))?;
    if !vue_file_name.ends_with(".vue") {
        return None;
    }

    let source_path = path.with_file_name(vue_file_name);
    if !source_path.is_file() {
        return None;
    }

    let uri = Url::from_file_path(source_path).ok()?;
    if let Some(range) = map_range(ctx, &uri, &location.range) {
        return Some(Location { uri, range });
    }

    Some(Location {
        uri,
        range: Range {
            start: tower_lsp::lsp_types::Position {
                line: 0,
                character: 0,
            },
            end: tower_lsp::lsp_types::Position {
                line: 0,
                character: 0,
            },
        },
    })
}

fn map_range(
    ctx: &IdeContext<'_>,
    source_uri: &Url,
    range: &vize_canon::LspRange,
) -> Option<Range> {
    let source_path = source_uri.to_file_path().ok()?;
    let source = ctx
        .state
        .documents
        .get(source_uri)
        .map(|document| document.text())
        .or_else(|| std::fs::read_to_string(&source_path).ok())?;
    let generated = ctx.state.canon_vue_document_for(
        source_uri,
        &source,
        vize_canon::CorsaVueVirtualDocumentOptions {
            options_api: ctx.state.options_api_enabled(),
            legacy_vue2: ctx.state.legacy_vue2_enabled(),
        },
    )?;

    let mirror_doc = CanonicalVirtualDocument {
        request_uri: cstr!("{}{}", source_uri.path(), generated.virtual_suffix),
        virtual_result: VirtualTsResult {
            code: generated.code.to_string(),
            source_mappings: generated.mappings.clone(),
            import_source_map: generated.import_source_map.clone(),
            user_code_start_line: 0,
            sfc_script_start_line: 0,
            template_scope_start_line: 0,
            line_mappings: Vec::new(),
            skipped_import_lines: 0,
        },
    };
    map_lsp_range_to_source(&source, &mirror_doc, range)
}

#[cfg(test)]
mod tests {
    use tower_lsp::lsp_types::Url;

    use super::map_location;
    use crate::ide::IdeContext;
    use crate::server::ServerState;

    #[test]
    fn repeated_cross_file_mapping_reuses_the_persistent_canon_frontend() {
        let project = tempfile::TempDir::new().expect("temp project");
        let host_path = project.path().join("Host.vue");
        let dependency_path = project.path().join("Dependency.vue");
        let host = r#"<script setup lang="ts">
import Dependency from "./Dependency.vue"
</script>
<template><Dependency /></template>"#;
        let dependency = r#"<script setup lang="ts">
const target = 1
</script>
<template>{{ target }}</template>"#;
        std::fs::write(&host_path, host).expect("host");
        std::fs::write(&dependency_path, dependency).expect("dependency");

        let state = ServerState::new();
        let host_uri = Url::from_file_path(&host_path).expect("host uri");
        let dependency_uri = Url::from_file_path(&dependency_path).expect("dependency uri");
        state
            .documents
            .open(host_uri.clone(), host.to_string(), 1, "vue".to_string());
        let options = vize_canon::CorsaVueVirtualDocumentOptions {
            options_api: state.options_api_enabled(),
            legacy_vue2: state.legacy_vue2_enabled(),
        };
        let host_document = state
            .canon_vue_document_for(&host_uri, host, options)
            .expect("host document");
        let overlays = state.canon_vue_overlays(&host_uri, &host_document, options);
        assert!(
            overlays.generated.iter().any(|(path, _)| {
                vize_carton::path::canonicalize_non_verbatim(path)
                    == vize_carton::path::canonicalize_non_verbatim(&dependency_path)
            }),
            "dependency must already be part of the persistent Canon graph",
        );
        let dependency_document = state
            .canon_vue_document_for(&dependency_uri, dependency, options)
            .expect("dependency document");
        let mapping = dependency_document
            .mappings
            .iter()
            .find(|mapping| mapping.gen_range.start < mapping.gen_range.end)
            .expect("dependency source mapping");
        let (line, character) =
            crate::ide::offset_to_position(&dependency_document.code, mapping.gen_range.start);
        let virtual_path = dependency_path.with_file_name(format!(
            "{}{}",
            dependency_path
                .file_name()
                .and_then(|name| name.to_str())
                .expect("dependency filename"),
            dependency_document.virtual_suffix,
        ));
        let location = vize_canon::LspLocation {
            uri: Url::from_file_path(virtual_path)
                .expect("virtual uri")
                .to_string(),
            range: vize_canon::LspRange {
                start: vize_canon::LspPosition { line, character },
                end: vize_canon::LspPosition { line, character },
            },
        };
        let ctx = IdeContext::new(&state, &host_uri, 0).expect("IDE context");
        let executions = || {
            [
                state.artifact_product_executions::<vize_atelier_sfc::SfcDescriptorProduct>(),
                state.artifact_product_executions::<vize_atelier_sfc::SfcScriptSyntaxProduct>(),
                state.artifact_product_executions::<vize_module::ModuleSyntaxProduct>(),
                state.artifact_product_executions::<vize_croquis::CroquisDocumentProduct>(),
                state.artifact_product_executions::<vize_relief::ReliefProduct>(),
                state.artifact_product_executions::<vize_canon::batch::CanonVueDocumentProduct>(),
            ]
        };
        let before = executions();
        vize_atelier_sfc::reset_authored_script_parse_invocations();
        vize_canon::virtual_ts::reset_authored_script_fallback_parse_invocations();

        for _ in 0..2 {
            let mapped = map_location(&ctx, &location).expect("mapped dependency location");
            assert_eq!(mapped.uri, dependency_uri);
        }

        assert_eq!(executions(), before);
        assert_eq!(vize_atelier_sfc::authored_script_parse_invocations(), 0);
        assert_eq!(
            vize_canon::virtual_ts::authored_script_fallback_parse_invocations(),
            0,
        );
    }
}
