//! Shared Corsa helpers for mapping virtual document responses back to Vue SFCs.
#![allow(clippy::disallowed_types, clippy::disallowed_methods)]

use std::collections::HashMap;
#[cfg(feature = "native")]
use std::sync::Arc;

use tower_lsp::lsp_types::{
    AnnotatedTextEdit, DocumentChangeOperation, DocumentChanges, Location, OneOf,
    PrepareRenameResponse, Range, TextEdit, Url, WorkspaceEdit,
};
#[cfg(feature = "native")]
use vize_canon::CorsaBridge;
use vize_canon::LspLocation;
use vize_carton::{String, cstr};

use super::IdeContext;
#[cfg(feature = "native")]
use super::diagnostics::VirtualTsResult;
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

/// Build the virtual template request path used for Corsa queries.
pub(crate) fn template_request_path(uri: &Url) -> String {
    cstr!("{}.template.ts", uri.path())
}

/// Build the virtual template request path for a specific art variant.
pub(crate) fn art_template_request_path(uri: &Url, variant_index: usize) -> String {
    cstr!("{}.art_variant_{variant_index}.template.ts", uri.path())
}

/// Build the virtual script request path used for Corsa queries.
pub(crate) fn script_request_path(uri: &Url, is_setup: bool) -> String {
    if is_setup {
        cstr!("{}.setup.ts", uri.path())
    } else {
        cstr!("{}.script.ts", uri.path())
    }
}

/// Build the canonical Vue virtual TS request path (`<file>.vue.ts`).
#[cfg(feature = "native")]
pub(crate) fn canonical_request_path(uri: &Url) -> String {
    cstr!("{}.ts", uri.path())
}

/// Build the per-query DOM tag request path.
#[cfg(feature = "native")]
pub(crate) fn html_tag_request_path(uri: &Url) -> String {
    cstr!("{}.html_tag.ts", uri.path())
}

/// Normalize a filesystem path into the `file://` form expected by Corsa.
pub(crate) fn request_file_uri(path: &str) -> String {
    if path.starts_with("file://") {
        String::from(path)
    } else {
        cstr!("file://{path}")
    }
}

#[cfg(feature = "native")]
pub(crate) struct CanonicalVirtualDocument {
    pub(crate) request_uri: String,
    pub(crate) virtual_result: VirtualTsResult,
}

#[cfg(feature = "native")]
pub(crate) struct HtmlTagVirtualDocument {
    pub(crate) content: String,
    pub(crate) hover_offset: usize,
    pub(crate) definition_offset: usize,
}

#[cfg(feature = "native")]
pub(crate) async fn open_canonical_virtual_document(
    ctx: &IdeContext<'_>,
    bridge: &Arc<CorsaBridge>,
) -> Option<CanonicalVirtualDocument> {
    if !ctx.uri.path().ends_with(".vue") || ctx.uri.path().ends_with(".art.vue") {
        return None;
    }

    let options_api = ctx.state.options_api_enabled();
    let legacy_vue2 = ctx.state.legacy_vue2_enabled();
    let virtual_result = crate::ide::DiagnosticService::generate_virtual_ts(
        ctx.uri,
        &ctx.content,
        options_api,
        legacy_vue2,
    )?;

    crate::ide::diagnostics::corsa::collect::overlay_sibling_vue_mirrors(
        bridge,
        ctx.uri,
        &virtual_result.relative_vue_imports,
        options_api,
        legacy_vue2,
    )
    .await;
    crate::ide::diagnostics::corsa::collect::overlay_relative_ts_imports(
        bridge,
        ctx.uri,
        &virtual_result.relative_ts_imports,
    )
    .await;

    let request_path = canonical_request_path(ctx.uri);
    let request_uri = bridge
        .open_or_update_virtual_document(&request_path, &virtual_result.code)
        .await
        .ok()?;

    Some(CanonicalVirtualDocument {
        request_uri,
        virtual_result,
    })
}

#[cfg(feature = "native")]
pub(crate) fn canonical_source_offset_to_position(
    doc: &CanonicalVirtualDocument,
    source_offset: usize,
) -> Option<(u32, u32)> {
    let generated_offset = source_offset_to_canonical_generated_offset(doc, source_offset)?;
    Some(super::offset_to_position(
        &doc.virtual_result.code,
        generated_offset,
    ))
}

#[cfg(feature = "native")]
fn source_offset_to_canonical_generated_offset(
    doc: &CanonicalVirtualDocument,
    source_offset: usize,
) -> Option<usize> {
    let mapping = mapping_for_source_offset(&doc.virtual_result.source_mappings, source_offset)?;
    let generated_pre_rewrite = map_source_offset_to_generated(mapping, source_offset);
    let generated_post_rewrite = doc
        .virtual_result
        .import_source_map
        .get_virtual_offset(generated_pre_rewrite as u32);
    Some(generated_post_rewrite as usize)
}

#[cfg(feature = "native")]
fn mapping_for_source_offset(
    mappings: &[vize_canon::virtual_ts::VizeMapping],
    offset: usize,
) -> Option<&vize_canon::virtual_ts::VizeMapping> {
    mappings
        .iter()
        .filter(|mapping| offset >= mapping.src_range.start && offset <= mapping.src_range.end)
        .min_by_key(|mapping| {
            mapping
                .src_range
                .end
                .saturating_sub(mapping.src_range.start)
        })
}

#[cfg(feature = "native")]
fn map_source_offset_to_generated(
    mapping: &vize_canon::virtual_ts::VizeMapping,
    source_offset: usize,
) -> usize {
    if let Some(span) = mapping
        .sub_spans
        .iter()
        .find(|span| source_offset >= span.src_range.start && source_offset <= span.src_range.end)
    {
        let relative = source_offset.saturating_sub(span.src_range.start);
        return span
            .gen_range
            .start
            .saturating_add(relative.min(span.gen_range.end.saturating_sub(span.gen_range.start)));
    }

    let relative = source_offset.saturating_sub(mapping.src_range.start);
    mapping.gen_range.start.saturating_add(
        relative.min(
            mapping
                .gen_range
                .end
                .saturating_sub(mapping.gen_range.start),
        ),
    )
}

#[cfg(feature = "native")]
pub(crate) fn map_canonical_corsa_locations(
    ctx: &IdeContext<'_>,
    doc: &CanonicalVirtualDocument,
    locations: Vec<LspLocation>,
) -> Vec<Location> {
    locations
        .iter()
        .filter_map(|location| map_canonical_corsa_location(ctx, doc, location))
        .collect()
}

#[cfg(feature = "native")]
pub(crate) fn map_canonical_corsa_location(
    ctx: &IdeContext<'_>,
    doc: &CanonicalVirtualDocument,
    location: &LspLocation,
) -> Option<Location> {
    if location_matches_uri(&location.uri, doc.request_uri.as_str())
        || virtual_document_path(&location.uri).as_deref()
            == Some(canonical_request_path(ctx.uri).as_str())
    {
        let range = map_canonical_lsp_range(ctx, doc, &location.range)?;
        return Some(Location {
            uri: ctx.uri.clone(),
            range,
        });
    }

    if let Some(location) = map_vue_virtual_mirror_location(location) {
        return Some(location);
    }

    let uri = Url::parse(&location.uri).ok()?;
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

#[cfg(feature = "native")]
pub(crate) fn map_canonical_lsp_range(
    ctx: &IdeContext<'_>,
    doc: &CanonicalVirtualDocument,
    range: &vize_canon::LspRange,
) -> Option<Range> {
    let generated_start_post = super::position_to_offset(
        &doc.virtual_result.code,
        range.start.line,
        range.start.character,
    )?;
    let generated_end_post = super::position_to_offset(
        &doc.virtual_result.code,
        range.end.line,
        range.end.character,
    )
    .unwrap_or(generated_start_post);

    let generated_start_pre = doc
        .virtual_result
        .import_source_map
        .get_original_offset(generated_start_post as u32) as usize;
    let generated_end_pre = doc
        .virtual_result
        .import_source_map
        .get_original_offset(generated_end_post as u32) as usize;

    let start_mapping =
        mapping_for_generated_offset(&doc.virtual_result.source_mappings, generated_start_pre)?;
    let source_start = map_generated_offset_to_source(start_mapping, generated_start_pre, false);
    let source_end =
        mapping_for_generated_offset(&doc.virtual_result.source_mappings, generated_end_pre)
            .map(|mapping| map_generated_offset_to_source(mapping, generated_end_pre, true))
            .unwrap_or_else(|| {
                source_start
                    .saturating_add(generated_end_pre.saturating_sub(generated_start_pre))
                    .min(start_mapping.src_range.end)
            })
            .max(source_start);

    let (start_line, start_character) = super::offset_to_position(&ctx.content, source_start);
    let (end_line, end_character) = super::offset_to_position(&ctx.content, source_end);

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

#[cfg(feature = "native")]
fn mapping_for_generated_offset(
    mappings: &[vize_canon::virtual_ts::VizeMapping],
    offset: usize,
) -> Option<&vize_canon::virtual_ts::VizeMapping> {
    mappings
        .iter()
        .filter(|mapping| offset >= mapping.gen_range.start && offset <= mapping.gen_range.end)
        .min_by_key(|mapping| {
            mapping
                .gen_range
                .end
                .saturating_sub(mapping.gen_range.start)
        })
}

#[cfg(feature = "native")]
fn map_generated_offset_to_source(
    mapping: &vize_canon::virtual_ts::VizeMapping,
    generated_offset: usize,
    prefer_end: bool,
) -> usize {
    if let Some(span) = mapping.sub_spans.iter().find(|span| {
        generated_offset >= span.gen_range.start && generated_offset <= span.gen_range.end
    }) {
        let relative = generated_offset.saturating_sub(span.gen_range.start);
        let source_len = span.src_range.end.saturating_sub(span.src_range.start);
        return span
            .src_range
            .start
            .saturating_add(relative.min(source_len));
    }

    if prefer_end && generated_offset >= mapping.gen_range.end {
        return mapping.src_range.end;
    }

    let relative = generated_offset.saturating_sub(mapping.gen_range.start);
    let source_len = mapping
        .src_range
        .end
        .saturating_sub(mapping.src_range.start);
    mapping
        .src_range
        .start
        .saturating_add(relative.min(source_len))
}

#[cfg(feature = "native")]
fn location_matches_uri(actual: &str, expected: &str) -> bool {
    actual == expected
        || virtual_document_path(actual).as_deref() == virtual_document_path(expected).as_deref()
}

#[cfg(feature = "native")]
fn map_vue_virtual_mirror_location(location: &LspLocation) -> Option<Location> {
    let parsed = Url::parse(&location.uri).ok()?;
    let path = parsed.to_file_path().ok()?;
    let file_name = path.file_name()?.to_str()?;
    let vue_file_name = file_name.strip_suffix(".ts")?;
    if !vue_file_name.ends_with(".vue") {
        return None;
    }
    let source_path = path.with_file_name(vue_file_name);
    if !source_path.is_file() {
        return None;
    }

    let uri = Url::from_file_path(source_path).ok()?;
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

#[cfg(feature = "native")]
pub(crate) fn html_tag_virtual_document(tag_name: &str) -> Option<HtmlTagVirtualDocument> {
    if !is_native_html_tag_candidate(tag_name) {
        return None;
    }

    let content = cstr!(
        "/// <reference lib=\"es2022\" />\n\
         /// <reference lib=\"dom\" />\n\
         /// <reference lib=\"dom.iterable\" />\n\
         type __VizeHtmlElement = HTMLElementTagNameMap[\"{tag_name}\"];\n\
         declare const __vizeHtmlElement: __VizeHtmlElement;\n\
         __vizeHtmlElement;\n"
    );
    let definition_offset = content.find("HTMLElementTagNameMap")?;
    let hover_offset = content.rfind("__vizeHtmlElement")?;

    Some(HtmlTagVirtualDocument {
        content,
        hover_offset,
        definition_offset,
    })
}

#[cfg(feature = "native")]
fn is_native_html_tag_candidate(tag_name: &str) -> bool {
    !tag_name.is_empty()
        && !matches!(
            tag_name,
            "component" | "template" | "slot" | "teleport" | "suspense"
        )
        && tag_name
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
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

    let uri = Url::parse(&location.uri).ok()?;
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

/// Rewrite a workspace edit so virtual-document edits target the Vue source.
pub(crate) fn map_corsa_workspace_edit(
    ctx: &IdeContext<'_>,
    mut edit: WorkspaceEdit,
) -> Option<WorkspaceEdit> {
    if let Some(changes) = edit.changes.take() {
        let mut mapped_changes = HashMap::with_capacity(changes.len());

        for (uri, edits) in changes {
            if let Some(current_doc) = match_current_virtual_document(ctx, uri.as_str()) {
                let entry = mapped_changes
                    .entry(ctx.uri.clone())
                    .or_insert_with(Vec::new);
                entry.extend(
                    edits
                        .into_iter()
                        .filter_map(|edit| map_text_edit(ctx, current_doc.document(), edit)),
                );
            } else {
                mapped_changes.insert(uri, edits);
            }
        }

        if !mapped_changes.is_empty() {
            edit.changes = Some(mapped_changes);
        }
    }

    if let Some(document_changes) = edit.document_changes.take() {
        let mapped_document_changes = match document_changes {
            DocumentChanges::Edits(edits) => {
                let edits = edits
                    .into_iter()
                    .filter_map(|edit| map_document_edit(ctx, edit))
                    .collect::<Vec<_>>();

                if edits.is_empty() {
                    None
                } else {
                    Some(DocumentChanges::Edits(edits))
                }
            }
            DocumentChanges::Operations(operations) => {
                let operations = operations
                    .into_iter()
                    .filter_map(|operation| map_document_change_operation(ctx, operation))
                    .collect::<Vec<_>>();

                if operations.is_empty() {
                    None
                } else {
                    Some(DocumentChanges::Operations(operations))
                }
            }
        };

        if let Some(document_changes) = mapped_document_changes {
            edit.document_changes = Some(document_changes);
        }
    }

    if workspace_edit_is_empty(&edit) {
        None
    } else {
        Some(edit)
    }
}

fn workspace_edit_is_empty(edit: &WorkspaceEdit) -> bool {
    let changes_empty = edit
        .changes
        .as_ref()
        .is_none_or(|changes| changes.values().all(Vec::is_empty));
    let document_changes_empty =
        edit.document_changes
            .as_ref()
            .is_none_or(|changes| match changes {
                DocumentChanges::Edits(edits) => edits.is_empty(),
                DocumentChanges::Operations(operations) => operations.is_empty(),
            });

    changes_empty && document_changes_empty
}

fn map_document_change_operation(
    ctx: &IdeContext<'_>,
    operation: DocumentChangeOperation,
) -> Option<DocumentChangeOperation> {
    match operation {
        DocumentChangeOperation::Edit(edit) => {
            map_document_edit(ctx, edit).map(DocumentChangeOperation::Edit)
        }
        DocumentChangeOperation::Op(op) => Some(DocumentChangeOperation::Op(op)),
    }
}

fn map_document_edit(
    ctx: &IdeContext<'_>,
    mut edit: tower_lsp::lsp_types::TextDocumentEdit,
) -> Option<tower_lsp::lsp_types::TextDocumentEdit> {
    let current_doc = match_current_virtual_document(ctx, edit.text_document.uri.as_str());

    if let Some(current_doc) = current_doc {
        edit.text_document.uri = ctx.uri.clone();
        edit.edits = edit
            .edits
            .into_iter()
            .filter_map(|entry| match entry {
                OneOf::Left(text_edit) => {
                    map_text_edit(ctx, current_doc.document(), text_edit).map(OneOf::Left)
                }
                OneOf::Right(annotated) => {
                    map_annotated_text_edit(ctx, current_doc.document(), annotated)
                        .map(OneOf::Right)
                }
            })
            .collect();
    }

    if edit.edits.is_empty() {
        None
    } else {
        Some(edit)
    }
}

fn map_annotated_text_edit(
    ctx: &IdeContext<'_>,
    document: &VirtualDocument,
    mut edit: AnnotatedTextEdit,
) -> Option<AnnotatedTextEdit> {
    edit.text_edit = map_text_edit(ctx, document, edit.text_edit)?;
    Some(edit)
}

fn map_text_edit(
    ctx: &IdeContext<'_>,
    document: &VirtualDocument,
    mut edit: TextEdit,
) -> Option<TextEdit> {
    edit.range = map_virtual_range(ctx, document, &edit.range)?;
    Some(edit)
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

fn virtual_document_path(uri: &str) -> Option<String> {
    if let Ok(parsed) = Url::parse(uri) {
        return Some(parsed.path().to_string().into());
    }

    if let Some(path) = uri.strip_prefix("vize-virtual://") {
        return Some(path.to_string().into());
    }

    None
}

#[cfg(all(test, feature = "native"))]
mod tests {
    use tower_lsp::lsp_types::Url;

    use super::{CanonicalVirtualDocument, canonical_source_offset_to_position};

    #[test]
    fn canonical_source_offset_maps_template_expression_to_generated_position() {
        let uri = Url::parse("file:///tmp/TypedTemplate.vue").expect("uri");
        let source = r#"<script setup lang="ts">
const user = { name: 'Ada' as string }
</script>

<template>
  {{ user.name }}
</template>
"#;
        let virtual_result =
            crate::ide::DiagnosticService::generate_virtual_ts(&uri, source, false, false)
                .expect("virtual ts");
        let doc = CanonicalVirtualDocument {
            request_uri: super::request_file_uri(super::canonical_request_path(&uri).as_str()),
            virtual_result,
        };

        let source_offset = source.rfind("name").unwrap() + "na".len();
        let (line, character) =
            canonical_source_offset_to_position(&doc, source_offset).expect("mapped position");
        let generated_offset =
            crate::ide::position_to_offset(&doc.virtual_result.code, line, character)
                .expect("generated offset");
        let expected_offset = doc.virtual_result.code.find("user.name").unwrap() + "user.na".len();

        assert_eq!(generated_offset, expected_offset);
    }

    #[test]
    fn canonical_source_offset_accounts_for_vue_import_rewrite_before_script_body() {
        let uri = Url::parse("file:///tmp/Parent.vue").expect("uri");
        let source = r#"<script setup lang="ts">
import Child from "./Child.vue";
const selected = Child;
</script>
"#;
        let virtual_result =
            crate::ide::DiagnosticService::generate_virtual_ts(&uri, source, false, false)
                .expect("virtual ts");
        let doc = CanonicalVirtualDocument {
            request_uri: super::request_file_uri(super::canonical_request_path(&uri).as_str()),
            virtual_result,
        };

        let source_offset = source.rfind("Child").unwrap() + "Ch".len();
        let (line, character) =
            canonical_source_offset_to_position(&doc, source_offset).expect("mapped position");
        let generated_offset =
            crate::ide::position_to_offset(&doc.virtual_result.code, line, character)
                .expect("generated offset");
        let expected_offset = doc.virtual_result.code.rfind("Child").unwrap() + "Ch".len();

        assert_eq!(generated_offset, expected_offset);
    }

    #[test]
    fn html_tag_virtual_document_queries_lib_dom_types() {
        let doc = super::html_tag_virtual_document("button").expect("html tag doc");

        assert!(doc.content.contains("HTMLElementTagNameMap[\"button\"]"));
        assert_eq!(
            &doc.content
                [doc.definition_offset..doc.definition_offset + "HTMLElementTagNameMap".len()],
            "HTMLElementTagNameMap",
        );
        assert_eq!(
            &doc.content[doc.hover_offset..doc.hover_offset + "__vizeHtmlElement".len()],
            "__vizeHtmlElement",
        );
    }
}
