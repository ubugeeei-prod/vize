use std::ops::Range as OffsetRange;

use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use tower_lsp::lsp_types::{Range, WorkspaceEdit};
use vize_croquis::{Drawer, DrawerOptions};

use crate::ide::{
    IdeContext, corsa_support::CanonicalVirtualDocument, definition::helpers, is_component_tag,
};

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
    if query_is_event_source(ctx) {
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
            let is_navigation = document
                .virtual_result
                .code
                .get(..start)
                .is_some_and(|head| {
                    head.rsplit_once('\n')
                        .map_or(head, |(_, line)| line)
                        .trim_start()
                        .starts_with("void __vize_kebab_events_nav_")
                });
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
    let authored = helpers::raw_attribute_with_source_span_at_offset(ctx)?;
    if !is_component_tag(&authored.tag_name) {
        return None;
    }
    let (_, start, end) = helpers::event_name_span(&authored.raw_name, authored.name_start)?;
    (ctx.offset >= start && ctx.offset < end).then_some(start..end)
}

pub(super) fn component_event_ranges(source: &str, filename: &str) -> Vec<OffsetRange<usize>> {
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
    let mut ranges = Vec::new();
    for usage in vize_croquis::facts::component_usage_list(&croquis) {
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
    append_pug_component_event_ranges(source, &mut ranges);
    ranges.sort_by_key(|range| (range.start, range.end));
    ranges.dedup();
    ranges
}

fn append_pug_component_event_ranges(source: &str, ranges: &mut Vec<OffsetRange<usize>>) {
    let mut append_at = |offset| {
        let Some(span) = crate::ide::pug::attribute_name_span_at_offset(source, offset) else {
            return;
        };
        let Some(tag_name) = source.get(span.tag_start..span.tag_end) else {
            return;
        };
        if !is_component_tag(tag_name) {
            return;
        }
        let Some(raw_name) = source.get(span.attribute_start..span.attribute_end) else {
            return;
        };
        let Some((_, start, end)) = helpers::event_name_span(raw_name, span.attribute_start) else {
            return;
        };
        ranges.push(start..end);
    };
    for (offset, _) in source.match_indices('@') {
        append_at(offset);
    }
    for (offset, _) in source.match_indices("v-on:") {
        append_at(offset);
    }
}

fn query_is_event_source(ctx: &IdeContext<'_>) -> bool {
    let Some(descriptor) = ctx.state.sfc_descriptor(ctx.uri, &ctx.content) else {
        return false;
    };
    let Some(script) = descriptor.script_setup.as_ref() else {
        return false;
    };
    let Some(relative) = ctx.offset.checked_sub(script.loc.start) else {
        return false;
    };
    let allocator = oxc_allocator::Allocator::default();
    let source_type = SourceType::ts().with_jsx(matches!(
        script.lang.as_deref().map(str::trim),
        Some("tsx" | "jsx")
    ));
    let parsed = vize_croquis::script_parser::parse_program_for_analysis(
        &allocator,
        &script.content,
        source_type,
    );
    if parsed.panicked {
        return false;
    }
    let mut drawer = Drawer::with_options(DrawerOptions {
        analyze_script: true,
        ..Default::default()
    });
    drawer.analyze_script_setup_program(
        &parsed.program,
        &script.content,
        script.attrs.get("generic").map(|generic| generic.as_ref()),
    );
    let croquis = drawer.finish();
    if croquis.macros.emits().iter().any(|event| {
        croquis
            .macros
            .emit_declaration(event.name.as_str())
            .is_some_and(|range| relative >= range.0 as usize && relative < range.1 as usize)
    }) {
        return true;
    }
    if croquis.macros.define_emits().is_none() || !parsed.diagnostics.is_empty() {
        return false;
    }
    let built = SemanticBuilder::new().build(&parsed.program);
    built.diagnostics.is_empty()
        && vize_canon::virtual_ts::owned_emit_navigation_source_ranges(
            &parsed.program,
            built.semantic.scoping(),
            &script.content,
            &croquis,
        )
        .iter()
        .any(|range| range.contains(&relative))
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

#[expect(clippy::string_slice, reason = "tests assert by panicking")]
#[cfg(test)]
mod tests {
    use tower_lsp::lsp_types::Url;
    use vize_l0::cstr;

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
    fn finds_multiline_pug_and_jade_component_event_names() {
        for language in ["pug", "jade"] {
            let language_attr = ["lang=\"", language, "\""].concat();
            let source = r#"<script setup>import Child from "./Child.vue"</script>
<template lang="pug">
Child(
  @save-item.once="handler"
  v-on:close.capture="close"
)
</template>"#
                .replace("lang=\"pug\"", &language_attr);
            let ranges = component_event_ranges(&source, "Parent.vue");
            let names = ranges
                .into_iter()
                .map(|range| &source[range])
                .collect::<Vec<_>>();
            assert_eq!(names, ["save-item", "close"], "{language}");
        }
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
