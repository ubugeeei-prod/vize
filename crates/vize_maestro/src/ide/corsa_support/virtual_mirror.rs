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
    let source = std::fs::read_to_string(&source_path).ok()?;
    let rewriter = vize_canon::ImportRewriter::new();
    let generated = vize_canon::batch::generate_vue_document_virtual_ts_with_options(
        &source_path,
        &source,
        &vize_canon::virtual_ts::VirtualTsOptions::default(),
        &rewriter,
        false,
        vize_canon::batch::VueDocumentVirtualTsOptions {
            options_api: ctx.state.options_api_enabled(),
            legacy_vue2: ctx.state.legacy_vue2_enabled(),
        },
    )
    .ok()?;

    let mirror_doc = CanonicalVirtualDocument {
        request_uri: cstr!("{}{}", source_uri.path(), generated.virtual_suffix),
        virtual_result: VirtualTsResult {
            code: generated.code.to_string(),
            source_mappings: generated.mappings,
            import_source_map: generated.import_source_map,
            user_code_start_line: 0,
            sfc_script_start_line: 0,
            template_scope_start_line: 0,
            line_mappings: Vec::new(),
            skipped_import_lines: 0,
        },
    };
    map_lsp_range_to_source(&source, &mirror_doc, range)
}
