use std::ops::Range as OffsetRange;

use tower_lsp::lsp_types::{Range, WorkspaceEdit};
use vize_croquis::{Drawer, DrawerOptions};

use crate::ide::{IdeContext, corsa_support::CanonicalVirtualDocument};

use super::super::RenameService;

mod model;
mod rewrite;

#[derive(Clone, Copy)]
pub(super) enum RenameKind {
    Event,
    Model,
}

pub(super) fn query_kind(ctx: &IdeContext<'_>) -> Option<RenameKind> {
    if let Some(range) = component_event_range_at(ctx) {
        return Some(if ctx.content.get(range)?.starts_with("update:") {
            RenameKind::Model
        } else {
            RenameKind::Event
        });
    }
    if model::usage_range_at(ctx).is_some() {
        return Some(RenameKind::Model);
    }
    if query_is_event_declaration(ctx) {
        return Some(RenameKind::Event);
    }
    model::query_is_declaration(ctx).then_some(RenameKind::Model)
}

pub(super) fn semantic_name(kind: Option<RenameKind>, new_name: &str) -> Option<String> {
    let candidate = match kind {
        Some(RenameKind::Model) => new_name.strip_prefix("update:").unwrap_or(new_name),
        _ => new_name,
    };
    if RenameService::is_valid_identifier(candidate) {
        return Some(candidate.to_string());
    }
    kind?;
    if !vize_croquis::naming::is_kebab_case(candidate) {
        return None;
    }
    let camel = crate::ide::definition::helpers::kebab_to_camel(candidate);
    RenameService::is_valid_identifier(&camel).then_some(camel)
}

pub(super) fn prepare_range(ctx: &IdeContext<'_>) -> Option<Range> {
    component_event_range_at(ctx)
        .or_else(|| model::usage_range_at(ctx))
        .map(|range| offset_range(&ctx.content, range))
}

pub(super) fn semantic_position(
    ctx: &IdeContext<'_>,
    document: &CanonicalVirtualDocument,
) -> Option<(u32, u32)> {
    let source_range = component_event_range_at(ctx)?;
    let authored = ctx.content.get(source_range.clone())?;
    let semantic = crate::ide::definition::helpers::kebab_to_camel(authored);
    document
        .virtual_result
        .source_mappings
        .iter()
        .filter(|mapping| mapping.src_range == source_range)
        .find_map(|mapping| {
            let start = document
                .virtual_result
                .import_source_map
                .get_virtual_offset(mapping.gen_range.start as u32)
                as usize;
            let end = document
                .virtual_result
                .import_source_map
                .get_virtual_offset(mapping.gen_range.end as u32) as usize;
            let line_start = document.virtual_result.code[..start]
                .rfind('\n')
                .map_or(0, |index| index + 1);
            let is_navigation = document.virtual_result.code[line_start..start]
                .trim_start()
                .starts_with("void __vize_kebab_events_nav_");
            (is_navigation
                && document.virtual_result.code.get(start..end) == Some(semantic.as_str()))
            .then(|| {
                crate::ide::offset_to_position(
                    &document.virtual_result.code,
                    start + usize::from(start < end),
                )
            })
        })
}

pub(super) fn rewrite_edits(
    ctx: &IdeContext<'_>,
    edit: &mut WorkspaceEdit,
    semantic_name: &str,
    kind: RenameKind,
) {
    rewrite::edits(ctx, edit, semantic_name, kind);
}

pub(super) fn model_linked_positions(
    ctx: &IdeContext<'_>,
    document: &CanonicalVirtualDocument,
    edit: &WorkspaceEdit,
) -> Vec<crate::ide::corsa_support::CanonicalSemanticPosition> {
    model::linked_positions(ctx, document, edit)
}

fn component_event_range_at(ctx: &IdeContext<'_>) -> Option<OffsetRange<usize>> {
    component_event_ranges(&ctx.content, ctx.uri.path())
        .into_iter()
        .find(|range| ctx.offset >= range.start && ctx.offset < range.end)
}

pub(super) fn component_event_ranges(source: &str, filename: &str) -> Vec<OffsetRange<usize>> {
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
    let mut ranges = Vec::new();
    for usage in &croquis.component_usages {
        for event in &usage.events {
            if event.name_is_dynamic {
                continue;
            }
            let start = template.loc.start + event.start as usize;
            let end = template.loc.start + event.end as usize;
            let Some(directive) = source.get(start..end) else {
                continue;
            };
            let Some(prefix_len) = ["@", "v-on:"].into_iter().find_map(|prefix| {
                directive
                    .strip_prefix(prefix)
                    .filter(|rest| rest.starts_with(event.name.as_str()))
                    .map(|_| prefix.len())
            }) else {
                continue;
            };
            let name_start = start + prefix_len;
            ranges.push(name_start..name_start + event.name.len());
        }
    }
    ranges
}

fn query_is_event_declaration(ctx: &IdeContext<'_>) -> bool {
    let Ok(descriptor) = vize_atelier_sfc::parse_sfc(
        &ctx.content,
        vize_atelier_sfc::SfcParseOptions {
            filename: ctx.uri.path().to_string().into(),
            ..Default::default()
        },
    ) else {
        return false;
    };
    let Some(script) = descriptor.script_setup else {
        return false;
    };
    let Some(relative) = ctx.offset.checked_sub(script.loc.start) else {
        return false;
    };
    let mut drawer = Drawer::with_options(DrawerOptions {
        analyze_script: true,
        ..Default::default()
    });
    drawer.analyze_script_setup(&script.content);
    let croquis = drawer.finish();
    croquis.macros.emits().iter().any(|event| {
        croquis
            .macros
            .emit_declaration(event.name.as_str())
            .is_some_and(|range| relative >= range.0 as usize && relative < range.1 as usize)
    })
}

pub(super) fn offset_range(source: &str, range: OffsetRange<usize>) -> Range {
    let (start_line, start_character) = crate::ide::offset_to_position(source, range.start);
    let (end_line, end_character) = crate::ide::offset_to_position(source, range.end);
    Range {
        start: tower_lsp::lsp_types::Position {
            line: start_line,
            character: start_character,
        },
        end: tower_lsp::lsp_types::Position {
            line: end_line,
            character: end_character,
        },
    }
}

#[cfg(test)]
mod tests {
    use tower_lsp::lsp_types::Url;
    use vize_s0::cstr;

    use super::{RenameKind, component_event_ranges, semantic_name};
    use crate::ide::IdeContext;
    use crate::server::ServerState;

    #[test]
    fn finds_the_whole_authored_component_event_name() {
        let source = r#"<script setup>import Child from "./Child.vue"</script>
<template><Child @save-item="handler" /></template>"#;
        let ranges = component_event_ranges(source, "Parent.vue");
        assert_eq!(ranges.len(), 1, "{ranges:#?}");
        assert_eq!(&source[ranges[0].clone()], "save-item");
    }

    #[test]
    fn normalizes_model_replacements_without_leaking_the_update_event_prefix() {
        assert_eq!(
            semantic_name(Some(RenameKind::Model), "update:next-value").as_deref(),
            Some("nextValue")
        );
        assert_eq!(
            semantic_name(Some(RenameKind::Model), "nextValue").as_deref(),
            Some("nextValue")
        );
        assert_eq!(semantic_name(Some(RenameKind::Model), "bad:name"), None);
    }

    #[test]
    fn accepts_kebab_replacements_for_static_emit_declarations() {
        for (index, (source, needle)) in [
            ("defineEmits<{ saveItem: [id: string] }>();", "saveItem"),
            (
                "defineEmits<{ (event: 'saveItem', id: string): void }>();",
                "saveItem",
            ),
            ("defineEmits(['saveItem']);", "saveItem"),
            (
                "defineEmits({ saveItem: (id: string) => true });",
                "saveItem",
            ),
        ]
        .into_iter()
        .enumerate()
        {
            let source = cstr!("<script setup lang=\"ts\">{source}</script>");
            let uri = Url::parse(&cstr!("file:///Child{index}.vue")).expect("uri");
            let state = ServerState::new();
            state
                .documents
                .open(uri.clone(), source.clone().into(), 1, "vue".to_string());
            let offset = source.find(needle).expect("event declaration") + 1;
            let ctx = IdeContext::new(&state, &uri, offset).expect("context");
            assert!(super::query_kind(&ctx).is_some(), "{source}");
            assert_eq!(
                semantic_name(Some(RenameKind::Event), "next-event").as_deref(),
                Some("nextEvent")
            );
        }
    }
}
