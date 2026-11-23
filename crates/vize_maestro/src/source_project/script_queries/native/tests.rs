use super::*;
use crate::source_project::{SnapshotRefusal, SourceSnapshotCache};
use tower_lsp::lsp_types::Url;
use vize_l0::{
    Allocator,
    config::{VueDialect, VueVersion},
};
use vize_l1::{container::vue::DescriptorOptions, markup::SurfaceParseOptions};
use vize_l1_to_l2::native_file::lower_sfc_native;

fn options() -> DescriptorOptions {
    DescriptorOptions {
        version: VueVersion::V3,
        dialect: VueDialect::Vue,
        template: SurfaceParseOptions::default(),
    }
}

#[derive(Default, Debug)]
struct Coverage {
    sources: usize,
    invalid: usize,
    responses: usize,
    scripts: usize,
    reused: usize,
    reused_in_incomplete_files: usize,
    symbols_bytes: usize,
    specifiers: usize,
}

fn compare(state: &ServerState, uri: &Url, source: &str, name: &str, counts: &mut Coverage) {
    state
        .documents
        .open(uri.clone(), source.into(), 1, "vue".into());
    let snapshot = SourceSnapshotCache::default()
        .capture(&state.documents, uri)
        .unwrap();
    let arena = Allocator::default();
    let owner = lower_sfc_native(&arena, snapshot.source(), options());
    let got = snapshot.script_query_products_from(state, &owner).unwrap();
    let original = ScriptQueryProducts::reference_symbols(state, uri);
    assert_eq!(
        got.as_ref().map(|read| &read.symbols),
        original.as_ref(),
        "{name}"
    );
    let original_specifiers = state.sfc_descriptor(uri, source).map(|desc| {
        for block in [desc.script.as_ref(), desc.script_setup.as_ref()]
            .into_iter()
            .flatten()
        {
            counts.scripts += 1;
            if matching_script(&owner, block)
                .and_then(EmbedSyntax::program)
                .is_some()
            {
                counts.reused += 1;
                counts.reused_in_incomplete_files += usize::from(owner.admitted().is_none());
            }
        }
        ScriptQueryProducts::reference_specifiers(&desc)
    });
    assert_eq!(
        got.as_ref().map(|read| &read.module_specifiers),
        original_specifiers.as_ref(),
        "{name}"
    );
    assert_eq!(
        got,
        snapshot.script_query_products(state).unwrap(),
        "{name}"
    );
    counts.sources += 1;
    if let Some(got) = got {
        let wire = serde_json::to_vec(&got.symbols).unwrap();
        assert_eq!(
            wire,
            serde_json::to_vec(&original.unwrap()).unwrap(),
            "{name}"
        );
        counts.responses += 1;
        counts.symbols_bytes += wire.len();
        counts.specifiers += got.module_specifiers.len();
    }
}

#[test]
fn current_intact_siblings_supply_their_own_programs_during_recovery() {
    let state = ServerState::new();
    let uri = Url::parse("file:///editor/Sibling.vue").unwrap();
    let mut counts = Coverage::default();
    for (name, source) in [
        (
            "syntax",
            "<!-- 🌸 -->\r\n<script>const broken = ;</script>\r\n<script setup>import Leaf from './Leaf.vue'; const menu = { open() { return import('./menu') } };</script>",
        ),
        (
            "binding",
            "<script setup>export const value = require('./value');</script><template>{{value}}</template>",
        ),
        (
            "ordinary-after-setup",
            "<script setup lang='ts'>const x: = 1;</script><script lang='ts'>export class A { select() { return import('./select') } }</script>",
        ),
        ("flow", "<script>/* @flow */ super;</script>"),
        (
            "jsx",
            "<script setup lang='tsx'>import { A } from './A'; const example = <A/>;</script>",
        ),
        (
            "warning",
            "<script>export { a } from './leaf'; const a = /uv;</script>",
        ),
        ("empty", "<script></script><script setup> </script>"),
        (
            "duplicate",
            "<script>const a=1;</script><script>const b=2;</script>",
        ),
    ] {
        compare(&state, &uri, source, name, &mut counts);
    }
    assert!(counts.reused_in_incomplete_files >= 2, "{counts:?}");
    assert!(counts.scripts > counts.reused && counts.specifiers > 0);
}

#[test]
fn identical_bytes_on_another_owner_and_an_old_editor_revision_are_refused() {
    let state = ServerState::new();
    let uri = Url::parse("file:///editor/Owner.vue").unwrap();
    let source = "<script setup>const value = 1;</script>";
    let cache = SourceSnapshotCache::default();
    state
        .documents
        .open(uri.clone(), source.into(), 3, "vue".into());
    let first = cache.capture(&state.documents, &uri).unwrap();
    let arena = Allocator::default();
    let copy = source.to_owned();
    let foreign = lower_sfc_native(&arena, &copy, options());
    assert_eq!(
        first.script_query_products_from(&state, &foreign),
        Err(ScriptQueryRefusal::DescriptorSourceMismatch)
    );
    let original = lower_sfc_native(&arena, first.source(), options());
    let expected = first.script_query_products(&state).unwrap();
    assert_eq!(
        first.script_query_products_from(&state, &original).unwrap(),
        expected
    );
    state.documents.close(&uri);
    assert_eq!(
        first.script_query_products_from(&state, &original),
        Err(ScriptQueryRefusal::Host(SnapshotRefusal::MissingDocument))
    );
    state
        .documents
        .open(uri.clone(), source.into(), 3, "vue".into());
    let current = cache.capture(&state.documents, &uri).unwrap();
    assert_ne!(current.revision(), first.revision());
    assert_eq!(
        first.script_query_products_from(&state, &original),
        Err(ScriptQueryRefusal::Host(SnapshotRefusal::Superseded))
    );
    assert_eq!(
        current.script_query_products_from(&state, &original),
        Err(ScriptQueryRefusal::DescriptorSourceMismatch)
    );
}

#[test]
#[ignore = "reads the tracked Vue source manifest through the original symbol and rename readers"]
fn complete_native_script_products_on_all_tracked_sources() {
    let input = std::fs::read(std::env::var_os("VIZE_SCRIPT_QUERY_INPUTS").unwrap()).unwrap();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let state = ServerState::new();
    let uri = Url::parse("file:///projects/Original.vue").unwrap();
    let mut counts = Coverage::default();
    for item in input.split(|&b| b == 0).filter(|item| !item.is_empty()) {
        let name = std::str::from_utf8(item).unwrap();
        let bytes = std::fs::read(root.join(name)).unwrap();
        match std::str::from_utf8(&bytes) {
            Ok(source) => compare(&state, &uri, source, name, &mut counts),
            Err(_) => counts.invalid += 1,
        }
    }
    assert!(counts.sources > 0 && counts.reused_in_incomplete_files > 0);
    assert!(counts.scripts > counts.reused && counts.specifiers > 0);
    eprintln!("retained_script_corpus {counts:?}");
}

#[cfg(feature = "micro-profiling")]
mod profile;
