//! Virtual TypeScript generation from Vue SFCs and `.art.vue` files.

#[cfg(test)]
use tower_lsp::lsp_types::Url;

use super::super::{DiagnosticService, VirtualTsResult};
#[cfg(test)]
use vize_canon::ImportRewriter;
use vize_canon::{CorsaVueVirtualDocument, ImportSourceMap};

impl DiagnosticService {
    pub(in crate::ide) fn virtual_ts_result_from_corsa_vue_document(
        opened: CorsaVueVirtualDocument,
    ) -> (std::string::String, VirtualTsResult) {
        (
            opened.request_uri.to_string(),
            VirtualTsResult::from_projection(
                opened.code.to_string(),
                opened.mapping,
                opened.import_source_map,
            ),
        )
    }

    /// Generate virtual TypeScript for a Vue SFC.
    #[cfg(test)]
    pub(in crate::ide) fn generate_virtual_ts(
        uri: &Url,
        content: &str,
        options_api: bool,
        legacy_vue2: bool,
    ) -> Option<VirtualTsResult> {
        use std::path::Path;
        use vize_canon::{
            batch::{VueDocumentVirtualTsOptions, generate_vue_document_virtual_ts_with_options},
            virtual_ts::VirtualTsOptions,
        };

        let virtual_ts_options = VirtualTsOptions::default();
        let rewriter = ImportRewriter::new();
        let generated = generate_vue_document_virtual_ts_with_options(
            Path::new(uri.path()),
            content,
            &virtual_ts_options,
            &rewriter,
            false,
            VueDocumentVirtualTsOptions {
                options_api,
                legacy_vue2,
                preserve_event_navigation: true,
                dialect: Default::default(),
                preserve_missing_vue_diagnostics: false,
                experimental_patterned_template: false,
            },
        )
        .ok()?;

        // The generated code is the same rewritten `.vue.ts` document that
        // CorsaBridge syncs for editor sessions; this helper keeps the mapping
        // metadata available to tests without owning dependency synchronization.
        Some(VirtualTsResult::from_projection(
            generated.code.to_string(),
            generated.mapping,
            generated.import_source_map,
        ))
    }
}

impl VirtualTsResult {
    /// A synced document from Canon's projection: span links stay in
    /// pre-rewrite coordinates, semantic links move into the rewritten code.
    pub(in crate::ide) fn from_projection(
        code: std::string::String,
        mapping: vize_canon::virtual_ts::ProjectionMapping,
        import_source_map: ImportSourceMap,
    ) -> Self {
        let (source_mappings, semantic_links) = mapping.into_parts();
        Self {
            code,
            source_mappings,
            semantic_links: semantic_links_after_import_rewrite(semantic_links, &import_source_map),
            import_source_map,
        }
    }
}

pub(in crate::ide) fn semantic_links_after_import_rewrite(
    links: Vec<vize_canon::virtual_ts::VizeSemanticLink>,
    import_source_map: &ImportSourceMap,
) -> Vec<vize_canon::virtual_ts::VizeSemanticLink> {
    links
        .into_iter()
        .map(|mut link| {
            link.source_range = rewrite_range(link.source_range, import_source_map);
            link.target_range = rewrite_range(link.target_range, import_source_map);
            link
        })
        .collect()
}

/// Map a pre-rewrite generated TypeScript range into the post-rewrite code
/// held by [`VirtualTsResult::code`].
fn rewrite_range(
    range: std::ops::Range<usize>,
    import_source_map: &ImportSourceMap,
) -> std::ops::Range<usize> {
    import_source_map.get_virtual_offset(range.start as u32) as usize
        ..import_source_map.get_virtual_offset(range.end as u32) as usize
}
