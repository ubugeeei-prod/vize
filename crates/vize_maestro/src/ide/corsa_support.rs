//! Shared Corsa helpers for mapping virtual document responses back to Vue SFCs.
#![cfg(feature = "native")]
#![allow(clippy::disallowed_types, clippy::disallowed_methods)]

use tower_lsp::lsp_types::{Location, PrepareRenameResponse, Range, Url};
mod canonical;
#[cfg(test)]
mod canonical_tests;
mod html_attribute;
#[cfg(test)]
mod html_attribute_tests;
mod html_tag;
mod svg_attribute;
mod virtual_mirror;
mod workspace_edit;

pub(crate) use canonical::{
    canonical_source_offset_to_position, map_canonical_corsa_locations, map_canonical_lsp_range,
    open_canonical_virtual_document,
};
pub(crate) use html_attribute::{
    html_attribute_request_path, html_attribute_virtual_document, native_dom_attribute_info,
};
pub(crate) use html_tag::{html_tag_request_path, html_tag_virtual_document, native_dom_tag_info};
pub(crate) use workspace_edit::map_corsa_workspace_edit;

use vize_canon::LspLocation;
use vize_carton::{String, cstr};

use super::IdeContext;
use crate::virtual_code::{SourceRange, VirtualDocument};

enum CurrentVirtualDocument<'a> {
    Template(&'a VirtualDocument),
    Script(&'a VirtualDocument),
    ScriptSetup(&'a VirtualDocument),
}

impl<'a> CurrentVirtualDocument<'a> {
    fn document(&self) -> &'a VirtualDocument {
        match self {
            Self::Template(doc) | Self::Script(doc) | Self::ScriptSetup(doc) => doc,
        }
    }
}

pub(crate) fn template_request_path(uri: &Url) -> String {
    cstr!("{}.template.ts", uri.path())
}

pub(crate) fn art_template_request_path(uri: &Url, variant_index: usize) -> String {
    cstr!("{}.art_variant_{variant_index}.template.ts", uri.path())
}

pub(crate) fn script_request_path(uri: &Url, is_setup: bool) -> String {
    if is_setup {
        cstr!("{}.setup.ts", uri.path())
    } else {
        cstr!("{}.script.ts", uri.path())
    }
}

pub(crate) fn request_file_uri(path: &str) -> String {
    if path.starts_with("file://") {
        String::from(path)
    } else {
        cstr!("file://{path}")
    }
}

/// Map a batch of Corsa locations back onto the current Vue document.
pub(crate) fn map_corsa_locations(
    ctx: &IdeContext<'_>,
    locations: Vec<LspLocation>,
) -> Vec<Location> {
    locations
        .iter()
        .filter_map(|location| map_corsa_location(ctx, location))
        .collect()
}

/// Map a single Corsa location back to either the Vue SFC or a real file URI.
pub(crate) fn map_corsa_location(ctx: &IdeContext<'_>, location: &LspLocation) -> Option<Location> {
    if let Some(current_doc) = match_current_virtual_document(ctx, &location.uri) {
        let range = map_virtual_range(
            ctx,
            current_doc.document(),
            &Range {
                start: tower_lsp::lsp_types::Position {
                    line: location.range.start.line,
                    character: location.range.start.character,
                },
                end: tower_lsp::lsp_types::Position {
                    line: location.range.end.line,
                    character: location.range.end.character,
                },
            },
        )?;

        return Some(Location {
            uri: ctx.uri.clone(),
            range,
        });
    }

    let uri = accessible_external_uri(ctx, &location.uri)?;
    Some(Location {
        uri,
        range: Range {
            start: tower_lsp::lsp_types::Position {
                line: location.range.start.line,
                character: location.range.start.character,
            },
            end: tower_lsp::lsp_types::Position {
                line: location.range.end.line,
                character: location.range.end.character,
            },
        },
    })
}

/// Reject stale file locations returned from an older checker snapshot.
///
/// Open editor buffers are valid even before their first save. Closed file
/// targets, however, must still exist on disk before they are exposed to the
/// client; otherwise delete/rename races leave a convincing but dead jump.
fn accessible_external_uri(ctx: &IdeContext<'_>, raw_uri: &str) -> Option<Url> {
    let uri = Url::parse(raw_uri).ok()?;
    if uri.scheme() != "file" || ctx.state.documents.contains(&uri) {
        return Some(uri);
    }

    uri.to_file_path().ok()?.is_file().then_some(uri)
}

/// Translate a Corsa prepare-rename payload into SFC coordinates.
pub(crate) fn map_corsa_prepare_rename(
    ctx: &IdeContext<'_>,
    request_uri: &str,
    response: PrepareRenameResponse,
) -> Option<PrepareRenameResponse> {
    let current_doc = match_current_virtual_document(ctx, request_uri)?;

    match response {
        PrepareRenameResponse::Range(range) => {
            map_virtual_range(ctx, current_doc.document(), &range).map(PrepareRenameResponse::Range)
        }
        PrepareRenameResponse::RangeWithPlaceholder { range, placeholder } => {
            map_virtual_range(ctx, current_doc.document(), &range)
                .map(|range| PrepareRenameResponse::RangeWithPlaceholder { range, placeholder })
        }
        PrepareRenameResponse::DefaultBehavior { default_behavior } => {
            Some(PrepareRenameResponse::DefaultBehavior { default_behavior })
        }
    }
}

fn map_virtual_range(
    ctx: &IdeContext<'_>,
    document: &VirtualDocument,
    range: &Range,
) -> Option<Range> {
    let generated_start =
        super::position_to_offset(&document.content, range.start.line, range.start.character)?;
    let generated_end =
        super::position_to_offset(&document.content, range.end.line, range.end.character)?;

    let source_range = if generated_end > generated_start {
        document
            .source_map
            .generated_range_to_source(SourceRange::new(
                generated_start as u32,
                generated_end as u32,
            ))?
    } else {
        let source_offset = document.source_map.to_source(generated_start as u32)?;
        SourceRange::new(source_offset, source_offset)
    };

    let (start_line, start_character) =
        super::offset_to_position(&ctx.content, source_range.start as usize);
    let (end_line, end_character) =
        super::offset_to_position(&ctx.content, source_range.end as usize);

    Some(Range {
        start: tower_lsp::lsp_types::Position {
            line: start_line,
            character: start_character,
        },
        end: tower_lsp::lsp_types::Position {
            line: end_line,
            character: end_character,
        },
    })
}

fn match_current_virtual_document<'a>(
    ctx: &'a IdeContext<'_>,
    uri: &str,
) -> Option<CurrentVirtualDocument<'a>> {
    let path = virtual_document_path(uri)?;
    let virtual_docs = ctx.virtual_docs.as_ref()?;

    if path == template_request_path(ctx.uri).as_str() {
        return virtual_docs
            .template
            .as_ref()
            .map(CurrentVirtualDocument::Template);
    }

    for (variant_index, template) in virtual_docs.art_templates.iter().enumerate() {
        if path == art_template_request_path(ctx.uri, variant_index).as_str() {
            return template.as_ref().map(CurrentVirtualDocument::Template);
        }
    }

    if path == script_request_path(ctx.uri, false).as_str() {
        return virtual_docs
            .script
            .as_ref()
            .map(CurrentVirtualDocument::Script);
    }

    if path == script_request_path(ctx.uri, true).as_str() {
        return virtual_docs
            .script_setup
            .as_ref()
            .map(CurrentVirtualDocument::ScriptSetup);
    }

    None
}

pub(super) fn virtual_document_path(uri: &str) -> Option<String> {
    if let Ok(parsed) = Url::parse(uri) {
        return Some(parsed.path().to_string().into());
    }

    if let Some(path) = uri.strip_prefix("vize-virtual://") {
        return Some(path.to_string().into());
    }

    None
}
