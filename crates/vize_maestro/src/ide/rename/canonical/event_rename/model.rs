use std::ops::Range;

use tower_lsp::lsp_types::{DocumentChangeOperation, DocumentChanges, OneOf, WorkspaceEdit};
use vize_croquis::{Drawer, DrawerOptions};
use vize_s0::FxHashSet;

use crate::ide::IdeContext;
use crate::ide::corsa_support::{CanonicalSemanticPosition, CanonicalVirtualDocument};

pub(super) fn query_is_declaration(ctx: &IdeContext<'_>) -> bool {
    declaration_ranges(&ctx.content, ctx.uri.path())
        .into_iter()
        .any(|range| ctx.offset >= range.start && ctx.offset < range.end)
}

pub(super) fn usage_range_at(ctx: &IdeContext<'_>) -> Option<Range<usize>> {
    usage_ranges(&ctx.content, ctx.uri.path())
        .into_iter()
        .find(|range| ctx.offset >= range.start && ctx.offset < range.end)
}

pub(super) fn usage_ranges(source: &str, filename: &str) -> Vec<Range<usize>> {
    let Ok(descriptor) = vize_atelier_sfc::parse_sfc(
        source,
        vize_atelier_sfc::SfcParseOptions {
            filename: filename.to_string().into(),
            ..Default::default()
        },
    ) else {
        return Vec::new();
    };
    let Some(template) = descriptor.template else {
        return Vec::new();
    };
    let Some(template_source) = source.get(template.loc.start..template.loc.end) else {
        return Vec::new();
    };
    let allocator = vize_s0::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, template_source);
    let mut drawer = Drawer::with_options(DrawerOptions {
        analyze_template_scopes: true,
        track_usage: true,
        ..Default::default()
    });
    drawer.draw_template(&root);
    let croquis = drawer.finish();
    croquis
        .component_usages
        .iter()
        .flat_map(|usage| &usage.props)
        .filter_map(|prop| {
            if prop.name_is_dynamic {
                return None;
            }
            let start = template.loc.start + prop.start as usize;
            let end = template.loc.start + prop.end as usize;
            let directive = source.get(start..end)?;
            directive
                .strip_prefix("v-model:")?
                .starts_with(prop.name.as_str())
                .then(|| {
                    let start = start + "v-model:".len();
                    start..start + prop.name.len()
                })
        })
        .collect()
}

pub(super) fn declaration_ranges(source: &str, filename: &str) -> Vec<Range<usize>> {
    let Ok(descriptor) = vize_atelier_sfc::parse_sfc(
        source,
        vize_atelier_sfc::SfcParseOptions {
            filename: filename.to_string().into(),
            ..Default::default()
        },
    ) else {
        return Vec::new();
    };
    let Some(script) = descriptor.script_setup else {
        return Vec::new();
    };
    let mut drawer = Drawer::with_options(DrawerOptions {
        analyze_script: true,
        ..Default::default()
    });
    drawer.analyze_script_setup(&script.content);
    let croquis = drawer.finish();
    croquis
        .macros
        .models()
        .iter()
        .filter_map(|model| {
            let (start, end) = croquis.macros.model_declaration(model.name.as_str())?;
            let mut range = script.loc.start + start as usize..script.loc.start + end as usize;
            let authored = source.get(range.clone())?;
            let unquoted = authored
                .strip_prefix(['\'', '"'])
                .and_then(|name| name.strip_suffix(['\'', '"']))
                .unwrap_or(authored);
            (unquoted == model.name).then(|| {
                if unquoted.len() != authored.len() {
                    range.start += 1;
                    range.end -= 1;
                }
                range
            })
        })
        .collect()
}

pub(super) fn linked_positions(
    ctx: &IdeContext<'_>,
    document: &CanonicalVirtualDocument,
    edit: &WorkspaceEdit,
) -> Vec<CanonicalSemanticPosition> {
    let mut positions = FxHashSet::default();
    let mut collect = |uri: &tower_lsp::lsp_types::Url, range| {
        collect_linked_positions(ctx, document, uri.as_str(), range, &mut positions);
    };
    if let Some(changes) = &edit.changes {
        for (uri, edits) in changes {
            for edit in edits {
                collect(uri, edit.range);
            }
        }
    }
    if let Some(changes) = &edit.document_changes {
        match changes {
            DocumentChanges::Edits(edits) => {
                for edit in edits {
                    for entry in &edit.edits {
                        collect(&edit.text_document.uri, annotatable_range(entry));
                    }
                }
            }
            DocumentChanges::Operations(operations) => {
                for operation in operations {
                    if let DocumentChangeOperation::Edit(edit) = operation {
                        for entry in &edit.edits {
                            collect(&edit.text_document.uri, annotatable_range(entry));
                        }
                    }
                }
            }
        }
    }
    positions.into_iter().collect()
}

fn collect_linked_positions(
    ctx: &IdeContext<'_>,
    document: &CanonicalVirtualDocument,
    uri: &str,
    range: tower_lsp::lsp_types::Range,
    positions: &mut FxHashSet<CanonicalSemanticPosition>,
) {
    let Some((request_uri, source_uri, source, result)) = virtual_result(ctx, document, uri) else {
        return;
    };
    let Some(start_post) =
        crate::ide::position_to_offset(&result.code, range.start.line, range.start.character)
    else {
        return;
    };
    let start = result
        .import_source_map
        .get_original_offset(start_post as u32) as usize;
    let Some(seed) = result
        .source_mappings
        .iter()
        .filter(|mapping| start >= mapping.gen_range.start && start <= mapping.gen_range.end)
        .min_by_key(|mapping| mapping.gen_range.len())
    else {
        return;
    };
    if !mapping_is_model_declaration(source, source_uri.path(), &seed.src_range) {
        return;
    }
    for mapping in result.source_mappings.iter().filter(|mapping| {
        mapping.src_range == seed.src_range && mapping.gen_range != seed.gen_range
    }) {
        let generated = result
            .import_source_map
            .get_virtual_offset(mapping.gen_range.start as u32) as usize;
        let query = generated
            + usize::from(
                result
                    .code
                    .as_bytes()
                    .get(generated)
                    .is_some_and(|byte| matches!(byte, b'\'' | b'"')),
            );
        let (line, character) = crate::ide::offset_to_position(&result.code, query);
        positions.insert(CanonicalSemanticPosition {
            request_uri: request_uri.clone(),
            line,
            character,
        });
    }
}

fn virtual_result<'a>(
    ctx: &'a IdeContext<'_>,
    document: &'a CanonicalVirtualDocument,
    uri: &str,
) -> Option<(
    &'a vize_s0::String,
    &'a tower_lsp::lsp_types::Url,
    &'a str,
    &'a crate::ide::diagnostics::VirtualTsResult,
)> {
    if uri == document.request_uri {
        return Some((
            &document.request_uri,
            ctx.uri,
            &ctx.content,
            &document.virtual_result,
        ));
    }
    document
        .dependencies
        .iter()
        .find(|dependency| uri == dependency.request_uri)
        .map(|dependency| {
            (
                &dependency.request_uri,
                &dependency.source_uri,
                dependency.source.as_str(),
                &dependency.virtual_result,
            )
        })
}

fn mapping_is_model_declaration(source: &str, filename: &str, mapping: &Range<usize>) -> bool {
    let Some(authored) = source.get(mapping.clone()) else {
        return false;
    };
    if authored.len() < 2
        || !matches!(authored.as_bytes()[0], b'\'' | b'"')
        || authored.as_bytes()[0] != authored.as_bytes()[authored.len() - 1]
    {
        return false;
    }
    declaration_ranges(source, filename)
        .into_iter()
        .any(|range| {
            let Some(start) = range.start.checked_sub(1) else {
                return false;
            };
            let end = range.end.saturating_add(1);
            source.get(start..end).is_some_and(|authored| {
                authored.len() >= 2
                    && matches!(authored.as_bytes()[0], b'\'' | b'"')
                    && authored.as_bytes()[0] == authored.as_bytes()[authored.len() - 1]
                    && &(start..end) == mapping
            })
        })
}

fn annotatable_range(
    edit: &OneOf<tower_lsp::lsp_types::TextEdit, tower_lsp::lsp_types::AnnotatedTextEdit>,
) -> tower_lsp::lsp_types::Range {
    match edit {
        OneOf::Left(edit) => edit.range,
        OneOf::Right(edit) => edit.text_edit.range,
    }
}

#[cfg(test)]
mod tests {
    use super::{declaration_ranges, usage_ranges};

    #[test]
    fn collects_explicit_model_names_without_their_syntax() {
        let source = r#"<script setup lang="ts">
defineModel<string>('title');
</script>
<template>
  <Child v-model:title.trim="value" />
  <Child v-model="value" />
  <Child v-model:[dynamic]="value" />
</template>
"#;
        let declarations = declaration_ranges(source, "Child.vue");
        let usages = usage_ranges(source, "Child.vue");

        assert_eq!(declarations.len(), 1, "{declarations:#?}");
        assert_eq!(&source[declarations[0].clone()], "title");
        assert_eq!(usages.len(), 1, "{usages:#?}");
        assert_eq!(&source[usages[0].clone()], "title");
    }
}
