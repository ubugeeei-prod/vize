//! Expand shorthand pattern bindings without renaming their matched property.

use tower_lsp::lsp_types::{
    DocumentChangeOperation, DocumentChanges, OneOf, Range, TextEdit, Url, WorkspaceEdit,
};
use vize_armature::patterns::{MatchPattern, PatternKind};
use vize_croquis::ScopeData;
use vize_s0::{FxHashMap, String, cstr};

use crate::ide::{IdeContext, corsa_support::CanonicalVirtualDocument};

struct Shorthand {
    binding: Range,
    property: Range,
    replacement: String,
}

pub(super) fn rewrite_shorthand_bindings(
    ctx: &IdeContext<'_>,
    document: &CanonicalVirtualDocument,
    edit: &mut WorkspaceEdit,
    new_name: &str,
) -> bool {
    // A workspace edit can contain many versioned/annotated edits per file.
    // Parse each authored source once, and reject unparseable pattern edits.
    let mut cache = FxHashMap::default();
    let mut valid = true;
    let mut rewrite = |uri: &Url, edit: &mut TextEdit| {
        if !uri.path().ends_with(".vue") {
            return;
        }
        let entries = cache.entry(uri.clone()).or_insert_with(|| {
            let source = if uri == ctx.uri {
                Some(ctx.content.clone())
            } else {
                ctx.state
                    .documents
                    .text(uri)
                    .or_else(|| document.authored_source(uri).map(str::to_owned))
            }?;
            let local = IdeContext::for_unopened(ctx.state, uri, 0, source);
            collect(&local, new_name)
        });
        let Some(entries) = entries else {
            valid = false;
            return;
        };
        if edit.new_text == new_name
            && let Some(entry) = entries.iter().find(|entry| entry.binding == edit.range)
        {
            edit.range = entry.property;
            edit.new_text = entry.replacement.to_string();
        }
    };
    if let Some(changes) = &mut edit.changes {
        for (uri, edits) in changes {
            for edit in edits {
                rewrite(uri, edit);
            }
        }
    }
    if let Some(changes) = &mut edit.document_changes {
        let mut document_edit = |edit: &mut tower_lsp::lsp_types::TextDocumentEdit| {
            for entry in &mut edit.edits {
                let text = match entry {
                    OneOf::Left(edit) => edit,
                    OneOf::Right(edit) => &mut edit.text_edit,
                };
                rewrite(&edit.text_document.uri, text);
            }
        };
        match changes {
            DocumentChanges::Edits(edits) => {
                for edit in edits {
                    document_edit(edit);
                }
            }
            DocumentChanges::Operations(operations) => {
                for operation in operations {
                    if let DocumentChangeOperation::Edit(edit) = operation {
                        document_edit(edit);
                    }
                }
            }
        }
    }
    valid
}

fn collect(ctx: &IdeContext<'_>, new_name: &str) -> Option<Vec<Shorthand>> {
    let (croquis, offset) = crate::ide::template_scope::analyze(ctx)?;
    if croquis
        .pattern_diagnostics
        .iter()
        .any(|diagnostic| !diagnostic.warning)
    {
        return None;
    }
    let mut entries = Vec::new();
    for scope in croquis.scopes.iter() {
        if let ScopeData::VWhen(when) = scope.data() {
            collect_pattern(
                &ctx.content,
                &when.arm.pattern,
                offset + when.offset as usize,
                new_name,
                &mut entries,
            );
        }
    }
    Some(entries)
}

fn collect_pattern(
    source: &str,
    pattern: &MatchPattern,
    offset: usize,
    new_name: &str,
    entries: &mut Vec<Shorthand>,
) {
    match &pattern.kind {
        PatternKind::Object { properties, .. } => {
            for property in properties {
                if let PatternKind::Binding(binding) = &property.pattern.kind
                    && property.key.span == binding.span
                {
                    let start = offset + property.span.start as usize;
                    let binding_start = offset + binding.span.start as usize;
                    let end = offset + binding.span.end as usize;
                    if let Some(prefix) = source.get(start..binding_start) {
                        entries.push(Shorthand {
                            binding: range(source, binding_start, end),
                            property: range(source, start, end),
                            replacement: cstr!("{}: {prefix}{new_name}", binding.name),
                        });
                    }
                } else {
                    collect_pattern(source, &property.pattern, offset, new_name, entries);
                }
            }
        }
        PatternKind::As { pattern, .. } => {
            collect_pattern(source, pattern, offset, new_name, entries)
        }
        PatternKind::Array { elements, .. } | PatternKind::Or(elements) => {
            for element in elements {
                collect_pattern(source, element, offset, new_name, entries);
            }
        }
        _ => {}
    }
}

fn range(source: &str, start: usize, end: usize) -> Range {
    let (start_line, start_character) = crate::ide::offset_to_position(source, start);
    let (end_line, end_character) = crate::ide::offset_to_position(source, end);
    Range::new(
        tower_lsp::lsp_types::Position::new(start_line, start_character),
        tower_lsp::lsp_types::Position::new(end_line, end_character),
    )
}
