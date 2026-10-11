use std::ops::Range;

use tower_lsp::lsp_types::{DocumentChangeOperation, DocumentChanges, OneOf, WorkspaceEdit};
use vize_croquis::{Drawer, DrawerOptions};
use vize_l0::FxHashSet;

use crate::ide::IdeContext;
pub(super) use crate::ide::corsa_support::model_declaration_ranges as declaration_ranges;
use crate::ide::corsa_support::{
    CanonicalSemanticPosition, CanonicalVirtualDocument, model_linked_semantic_positions,
};

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
    let Some(descriptor) = vize_incremental::parse_descriptor(filename, source) else {
        return Vec::new();
    };
    let Some(template) = descriptor.template.as_ref() else {
        return Vec::new();
    };
    let Some(template_source) = source.get(template.loc.start..template.loc.end) else {
        return Vec::new();
    };
    let allocator = vize_l0::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, template_source);
    let mut drawer = Drawer::with_options(DrawerOptions {
        analyze_template_scopes: true,
        track_usage: true,
        ..Default::default()
    });
    drawer.draw_template(&root);
    let croquis = drawer.finish();
    vize_croquis::facts::component_usage_list(&croquis)
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
    positions.extend(model_linked_semantic_positions(
        ctx,
        document,
        uri,
        &crate::ide::corsa_support::tower_range(range),
    ));
}

fn annotatable_range(
    edit: &OneOf<tower_lsp::lsp_types::TextEdit, tower_lsp::lsp_types::AnnotatedTextEdit>,
) -> tower_lsp::lsp_types::Range {
    match edit {
        OneOf::Left(edit) => edit.range,
        OneOf::Right(edit) => edit.text_edit.range,
    }
}

#[expect(clippy::string_slice, reason = "tests assert by panicking")]
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
