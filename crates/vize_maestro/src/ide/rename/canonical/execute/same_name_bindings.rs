//! Preserve both meanings of Vue's same-name `v-bind` shorthand.

use std::ops::Range as OffsetRange;

use oxc_ast::AstKind;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use tower_lsp::lsp_types::{
    DocumentChangeOperation, DocumentChanges, OneOf, TextEdit, Url, WorkspaceEdit,
};
use vize_l0::{FxHashMap, String, cstr};
use vize_relief::{ExpressionNode, PropNode, TemplateChildNode};

use crate::ide::{IdeContext, corsa_support::CanonicalVirtualDocument};

struct Shorthand {
    argument: OffsetRange<usize>,
    directive: OffsetRange<usize>,
    expression: String,
}

/// A reactive destructure declares a local binding even though the same token
/// also names a public prop. Native prop navigation must not choose that role
/// when the actual cursor is on the authored binding declaration.
pub(super) fn is_binding_declaration(ctx: &IdeContext<'_>) -> bool {
    let Some(descriptor) = ctx.descriptor() else {
        return false;
    };
    let Some(script) = descriptor.script_setup.as_ref() else {
        return false;
    };
    let Some(offset) = ctx.offset.checked_sub(script.loc.start) else {
        return false;
    };
    let allocator = oxc_allocator::Allocator::default();
    let parsed = Parser::new(&allocator, &script.content, SourceType::ts()).parse();
    if parsed.panicked || !parsed.diagnostics.is_empty() {
        return false;
    }
    let semantic = SemanticBuilder::new()
        .with_build_nodes(true)
        .build(&parsed.program)
        .semantic;
    semantic.nodes().iter().any(|node| {
        matches!(node.kind(), AstKind::BindingIdentifier(binding)
            if offset >= binding.span.start as usize && offset < binding.span.end as usize)
    })
}

pub(super) fn rewrite(
    ctx: &IdeContext<'_>,
    document: &CanonicalVirtualDocument,
    edit: &mut WorkspaceEdit,
    new_name: &str,
    property_rename: bool,
) -> bool {
    // Only edits already selected by the native symbol queries participate.
    // A parsed directive supplies its complete geometry and implicit value;
    // neither its spelling nor a generated helper name discovers an edit.
    let mut cache = FxHashMap::default();
    let mut valid = true;
    let mut rewrite = |uri: &Url, edit: &mut TextEdit| {
        if !uri.path().ends_with(".vue") {
            return;
        }
        let parsed = cache.entry(uri.clone()).or_insert_with(|| {
            let source = if uri == ctx.uri {
                Some(ctx.content.clone())
            } else {
                document
                    .authored_source(uri)
                    .map(str::to_owned)
                    .or_else(|| ctx.state.documents.text(uri))
            }?;
            let local = IdeContext::for_unopened(ctx.state, uri, 0, source.clone());
            Some((source, collect(&local)?))
        });
        let Some((source, entries)) = parsed else {
            valid = false;
            return;
        };
        let Some(start) = crate::ide::position_to_offset(
            source,
            edit.range.start.line,
            edit.range.start.character,
        ) else {
            valid = false;
            return;
        };
        let Some(end) =
            crate::ide::position_to_offset(source, edit.range.end.line, edit.range.end.character)
        else {
            valid = false;
            return;
        };
        let Some(entry) = entries.iter().find(|entry| {
            start >= entry.argument.start && end <= entry.argument.end && start <= end
        }) else {
            return;
        };
        if edit.new_text != new_name {
            valid = false;
            return;
        }
        let Some(original) = source.get(entry.directive.clone()) else {
            valid = false;
            return;
        };
        let replacement = if property_rename {
            let relative = entry.argument.start - entry.directive.start
                ..entry.argument.end - entry.directive.start;
            let mut directive = original.to_owned();
            let replacement = if entry.expression.as_str()
                != source.get(entry.argument.clone()).unwrap_or_default()
            {
                crate::ide::pascal_to_kebab(new_name)
            } else {
                new_name.to_owned()
            };
            directive.replace_range(relative, &replacement);
            cstr!("{directive}=\"{}\"", entry.expression)
        } else {
            cstr!("{original}=\"{new_name}\"")
        };
        edit.range = super::super::event_rename::offset_range(source, entry.directive.clone());
        edit.new_text = replacement.into();
    };
    if let Some(changes) = &mut edit.changes {
        for (uri, edits) in changes {
            for edit in edits.iter_mut() {
                rewrite(uri, edit);
            }
            edits.dedup_by(|a, b| a.range == b.range && a.new_text == b.new_text);
        }
    }
    if let Some(changes) = &mut edit.document_changes {
        let mut document_edit = |edit: &mut tower_lsp::lsp_types::TextDocumentEdit| {
            for entry in &mut edit.edits {
                rewrite(&edit.text_document.uri, text_edit_mut(entry));
            }
            edit.edits.dedup_by(|a, b| {
                let a = text_edit_mut(a);
                let b = text_edit_mut(b);
                a.range == b.range && a.new_text == b.new_text
            });
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

fn text_edit_mut(
    edit: &mut OneOf<TextEdit, tower_lsp::lsp_types::AnnotatedTextEdit>,
) -> &mut TextEdit {
    match edit {
        OneOf::Left(edit) => edit,
        OneOf::Right(edit) => &mut edit.text_edit,
    }
}

fn collect(ctx: &IdeContext<'_>) -> Option<Vec<Shorthand>> {
    let descriptor = ctx.descriptor()?;
    let Some(template) = descriptor.template.as_ref() else {
        return Some(Vec::new());
    };
    let allocator = vize_l0::Allocator::new();
    let (root, errors) = vize_armature::parse(&allocator, &template.content);
    if errors.iter().any(|error| !error.is_recoverable()) {
        return None;
    }
    let mut entries = Vec::new();
    collect_children(&root.children, template.loc.start, &mut entries);
    Some(entries)
}

fn collect_children(
    children: &[TemplateChildNode<'_>],
    offset: usize,
    entries: &mut Vec<Shorthand>,
) {
    for child in children {
        if let TemplateChildNode::Element(element) = child {
            for prop in &element.props {
                if let PropNode::Directive(directive) = prop
                    && directive.name == "bind"
                    && directive.shorthand
                    && let Some(ExpressionNode::Simple(argument)) = &directive.arg
                    && argument.is_static
                    && let Some(ExpressionNode::Simple(expression)) = &directive.exp
                {
                    entries.push(Shorthand {
                        argument: offset + argument.loc.span.start as usize
                            ..offset + argument.loc.span.end as usize,
                        directive: offset + directive.loc.span.start as usize
                            ..offset + directive.loc.span.end as usize,
                        expression: String::from(expression.content),
                    });
                }
            }
            collect_children(&element.children, offset, entries);
        }
    }
}

/// Native aliases can project two ranges onto the same Vue token. Refuse the
/// whole transaction if expansion still leaves conflicting or overlapping
/// edits; clients must never receive order-dependent source mutations.
pub(super) fn coherent(edit: &WorkspaceEdit) -> bool {
    let mut files: FxHashMap<&Url, Vec<&TextEdit>> = FxHashMap::default();
    if let Some(changes) = &edit.changes {
        for (uri, edits) in changes {
            files.entry(uri).or_default().extend(edits);
        }
    }
    if let Some(changes) = &edit.document_changes {
        match changes {
            DocumentChanges::Edits(edits) => {
                for edit in edits {
                    collect_document_edits(&mut files, edit);
                }
            }
            DocumentChanges::Operations(operations) => {
                for operation in operations {
                    if let DocumentChangeOperation::Edit(edit) = operation {
                        collect_document_edits(&mut files, edit);
                    }
                }
            }
        }
    }
    files.into_values().all(|mut edits| {
        edits.sort_by_key(|edit| (edit.range.start, edit.range.end));
        edits.iter().all(|edit| edit.range.start <= edit.range.end)
            && edits.windows(2).all(|pair| {
                let [left, right] = pair else {
                    return false;
                };
                if left.range == right.range {
                    left.new_text == right.new_text
                } else {
                    left.range.end <= right.range.start
                }
            })
    })
}

fn collect_document_edits<'a>(
    files: &mut FxHashMap<&'a Url, Vec<&'a TextEdit>>,
    edit: &'a tower_lsp::lsp_types::TextDocumentEdit,
) {
    files
        .entry(&edit.text_document.uri)
        .or_default()
        .extend(edit.edits.iter().map(|entry| match entry {
            OneOf::Left(edit) => edit,
            OneOf::Right(edit) => &edit.text_edit,
        }));
}

#[cfg(test)]
#[path = "same_name_bindings_tests.rs"]
mod tests;
