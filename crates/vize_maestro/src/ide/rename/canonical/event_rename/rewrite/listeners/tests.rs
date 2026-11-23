#![expect(
    clippy::unwrap_used,
    reason = "assert complete publications from original test inputs"
)]
use super::super::{RenameKind, component_event_ranges, offset_range, rewrite_text_edits};
use super::*;
use crate::server::ServerState;
use tower_lsp::lsp_types::Url;
use vize_l0::{
    Allocator,
    config::{VueDialect, VueVersion},
};
use vize_l1::{container::vue::DescriptorOptions, markup::SurfaceParseOptions};
use vize_l1_to_l2::native_file::{NativeSfcObservation, lower_sfc_with_listener_references};

const INPUT: &str = "<script setup lang='ts'>import Child from './Child.vue'; import { save } from './actions';</script>\r\n<template>😀<Child @saveResult.once='save'><Child v-on:update:modelValue='save' @完了='save'/></Child></template>";

fn observe<'a>(arena: &'a Allocator, source: &'a str) -> NativeSfcObservation<'a> {
    lower_sfc_with_listener_references(
        arena,
        source,
        DescriptorOptions {
            version: VueVersion::V3,
            dialect: VueDialect::Vue,
            template: SurfaceParseOptions::default(),
        },
    )
}
fn document<T>(source: &str, read: impl FnOnce(&IdeContext<'_>) -> T) -> T {
    let state = ServerState::new();
    let uri = Url::parse("file:///project/Parent.vue").unwrap();
    state
        .documents
        .open(uri.clone(), source.to_owned(), 1, "vue".into());
    let ctx = IdeContext::new(&state, &uri, 0).unwrap();
    read(&ctx)
}

fn input_edits(ctx: &IdeContext<'_>) -> Vec<TextEdit> {
    let source = ctx.content.as_str();
    let mut edits = vec![TextEdit::new(
        offset_range(source, 0..0),
        "preserve typed edit".into(),
    )];
    for range in component_event_ranges(source, ctx.uri.path()) {
        for (delta, _) in source[range.clone()].char_indices() {
            let offset = range.start + delta;
            edits.push(TextEdit::new(
                offset_range(source, offset..offset),
                "from semantic service".into(),
            ));
        }
        edits.push(TextEdit::new(
            offset_range(source, range.end..range.end),
            "outside name".into(),
        ));
    }
    edits
}

fn compare(ctx: &IdeContext<'_>, imports: &SetupComponentImports<'_, '_>) -> Vec<TextEdit> {
    let input = input_edits(ctx);
    let mut expected = input.clone();
    rewrite_text_edits(ctx, ctx.uri, &mut expected, "nextValue", RenameKind::Event);
    let actual =
        RenameService::rewrite_component_event_edits(ctx, imports, input, "nextValue").unwrap();
    assert_eq!(actual, expected, "{}", ctx.content);
    actual
}

#[test]
fn semantic_edits_preserve_unicode_ranges_update_prefixes_and_unrelated_edits() {
    document(INPUT, |ctx| {
        let arena = Allocator::new();
        let observed = observe(&arena, &ctx.content);
        let imports = observed.setup_component_imports().unwrap();
        let actual = compare(ctx, &imports);
        assert_eq!(actual[0].new_text, "preserve typed edit");
        assert!(actual.iter().any(|e| e.new_text == "next-value"));
        assert!(actual.iter().any(|e| e.new_text == "update:next-value"));
        assert!(actual.iter().any(|e| e.new_text == "outside name"));
        let other = observe(&arena, INPUT);
        assert_eq!(
            RenameService::rewrite_component_event_edits(
                ctx,
                &other.setup_component_imports().unwrap(),
                Vec::new(),
                "nextValue"
            ),
            Err(ComponentEventEditRefusal::ForeignFile)
        );
        ctx.state.documents.close(ctx.uri);
        assert_eq!(
            RenameService::rewrite_component_event_edits(ctx, &imports, Vec::new(), "nextValue"),
            Err(ComponentEventEditRefusal::Closed)
        );
        ctx.state
            .documents
            .open(ctx.uri.clone(), INPUT.into(), 1, "vue".into());
        assert_eq!(
            RenameService::rewrite_component_event_edits(ctx, &imports, Vec::new(), "nextValue"),
            Err(ComponentEventEditRefusal::Superseded)
        );
    });
}

#[test]
#[ignore = "original Vue files and complete semantic edit response for every admitted caller"]
fn original_event_rewrites() {
    let list = std::fs::read_to_string(std::env::var_os("VIZE_EXPRESSION_QUERY_MANIFEST").unwrap())
        .unwrap();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let (mut files, mut invalid, mut admitted, mut adjusted, mut edits) = (0, 0, 0, 0, 0);
    for file in list.split_terminator('\0') {
        files += 1;
        let bytes = std::fs::read(root.join(file)).unwrap();
        let Ok(source) = std::str::from_utf8(&bytes) else {
            invalid += 1;
            continue;
        };
        document(source, |ctx| {
            let arena = Allocator::new();
            let observed = observe(&arena, &ctx.content);
            if let Ok(imports) = observed.setup_component_imports() {
                let result = compare(ctx, &imports);
                admitted += 1;
                adjusted += usize::from(result.len() > 1);
                edits += result.len();
            }
        });
    }
    assert!(admitted > 0);
    if std::env::var_os("VIZE_REQUIRE_REPOSITORY_LISTENERS").is_some() {
        assert!(adjusted >= 12);
    }
    eprintln!(
        "component_event_rewrites files={files} invalid={invalid} admitted={admitted} event_files={adjusted} edits={edits}"
    );
}

#[cfg(feature = "micro-profiling")]
mod profile;
