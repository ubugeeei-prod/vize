//! Typed queries inside a standalone Art variant template.
//!
//! Diagnostics synchronize every variant through the canonical native project,
//! whose session root is the editor mirror. A variant opened as a bare overlay
//! afterwards lands outside that root, so its relative imports stop resolving
//! and imported bindings degrade to `any` (#13383). Opening the projection
//! through the same project keeps its imports and the opened document's
//! import-rewrite coordinates.

use oxc_span::SourceType;
use tower_lsp::lsp_types::{Position, Range};
use vize_canon::{CorsaBridge, CorsaScriptVirtualDocumentRequest, ImportSourceMap, LspRange};
use vize_l0::{String, cstr};

use crate::ide::{IdeContext, offset_to_position, position_to_offset};
use crate::virtual_code::VirtualDocument;

/// One Art variant projection opened in the native session at a projection offset.
pub(crate) struct ArtTemplateQuery<'a> {
    document: &'a VirtualDocument,
    pub(crate) request_uri: String,
    /// The opened text after import rewriting.
    code: String,
    import_source_map: ImportSourceMap,
    pub(crate) line: u32,
    pub(crate) character: u32,
}

impl ArtTemplateQuery<'_> {
    /// Map a range in the opened document back to the authored Art file.
    pub(crate) fn authored_range(&self, ctx: &IdeContext<'_>, range: &LspRange) -> Option<Range> {
        let projected = |line: u32, character: u32| {
            let opened = position_to_offset(&self.code, line, character)?;
            let offset = self.import_source_map.get_original_offset(opened as u32) as usize;
            let (line, character) = offset_to_position(&self.document.content, offset);
            Some(Position::new(line, character))
        };
        super::map_virtual_range(
            ctx,
            self.document,
            &Range {
                start: projected(range.start.line, range.start.character)?,
                end: projected(range.end.line, range.end.character)?,
            },
        )
    }
}

pub(crate) async fn open_art_template_query<'a>(
    ctx: &IdeContext<'_>,
    bridge: &CorsaBridge,
    variant_index: usize,
    document: &'a VirtualDocument,
    generated_offset: usize,
) -> Option<ArtTemplateQuery<'a>> {
    let request_path = super::art_template_request_path(ctx.uri, variant_index);
    let mut source_path = ctx.uri.to_file_path().ok()?;
    let source_suffix = cstr!(".art_variant_{variant_index}.template.ts");
    source_path.as_mut_os_string().push(source_suffix.as_str());
    let cached_overlays = ctx.state.corsa_overlays();
    let overlays = cached_overlays
        .iter()
        .map(|(path, content)| (path.clone(), content.as_str()))
        .collect::<Vec<_>>();
    let virtual_ts_options = ctx.state.virtual_ts_options();
    let opened = bridge
        .open_script_virtual_document_with_vue_dependencies(CorsaScriptVirtualDocumentRequest {
            source_path: &source_path,
            request_path: &request_path,
            code: &document.content,
            source_type: SourceType::ts(),
            options: ctx.state.canonical_vue_document_options(),
            overlays: &overlays,
            virtual_ts_options: &virtual_ts_options,
        })
        .await
        .ok()?;
    let offset = opened
        .import_source_map
        .get_virtual_offset(generated_offset as u32) as usize;
    let (line, character) = offset_to_position(&opened.code, offset);
    Some(ArtTemplateQuery {
        document,
        request_uri: opened.request_uri,
        code: opened.code,
        import_source_map: opened.import_source_map,
        line,
        character,
    })
}
