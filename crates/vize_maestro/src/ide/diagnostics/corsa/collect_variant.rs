//! Synchronize authored art variants through the canonical native project.

use super::super::VirtualTsResult;
use super::collect_virtual::collect_synced_virtual_result_diagnostics;
use tower_lsp::lsp_types::{Diagnostic, Url};

/// A generated art variant shares the native project and dependency graph with
/// regular Vue and script documents. Relative-only in-memory mirrors cannot
/// participate in native TypeScript module resolution.
pub(super) struct VariantProjectContext<'a> {
    pub options: vize_canon::CorsaVueVirtualDocumentOptions,
    pub overlays: &'a [(std::path::PathBuf, &'a str)],
    pub virtual_ts_options: &'a vize_canon::virtual_ts::VirtualTsOptions,
}

pub(super) async fn collect_virtual_result_diagnostics(
    bridge: &std::sync::Arc<vize_canon::CorsaBridge>,
    host_uri: &Url,
    content: &str,
    virtual_name: String,
    mut virtual_result: VirtualTsResult,
    project: VariantProjectContext<'_>,
) -> Result<(Vec<Diagnostic>, Vec<std::path::PathBuf>), vize_canon::CorsaBridgeError> {
    let opened = bridge
        .open_script_virtual_document_with_vue_dependencies(
            vize_canon::CorsaScriptVirtualDocumentRequest {
                source_path: std::path::Path::new(&virtual_name),
                request_path: &virtual_name,
                code: &virtual_result.code,
                source_type: oxc_span::SourceType::ts(),
                options: project.options,
                overlays: project.overlays,
                virtual_ts_options: project.virtual_ts_options,
            },
        )
        .await?;
    virtual_result.semantic_links = super::semantic_links_after_import_rewrite(
        virtual_result.semantic_links,
        &opened.import_source_map,
    );
    virtual_result.code = opened.code.to_string();
    virtual_result.import_source_map = opened.import_source_map;
    let diagnostics = collect_synced_virtual_result_diagnostics(
        bridge,
        host_uri,
        content,
        opened.request_uri.to_string(),
        virtual_result,
    )
    .await?;
    Ok((diagnostics, opened.resolved_dependencies))
}
