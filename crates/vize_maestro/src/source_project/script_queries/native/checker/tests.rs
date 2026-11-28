use super::*;
use crate::source_project::{SnapshotRefusal, SourceSnapshotCache};
use tower_lsp::lsp_types::Url;
use vize_canon::{SfcTypeCheckOptions, type_check_sfc};
use vize_l0::config::{VueDialect, VueVersion};
use vize_l1::{container::vue::DescriptorOptions, markup::SurfaceParseOptions};
use vize_l1_to_l2::native_file::lower_sfc_native;

fn options() -> DescriptorOptions {
    DescriptorOptions {
        version: VueVersion::V3,
        dialect: VueDialect::Vue,
        template: SurfaceParseOptions::default(),
    }
}

#[derive(Debug, Default)]
struct Coverage {
    files: usize,
    responses: usize,
    blocks: usize,
    clean_checker_programs: usize,
    reused_from_incomplete_files: usize,
    diagnostics: usize,
    invalid: usize,
}

type Products = (ScriptQueryProducts, Vec<SfcTypeDiagnostic>);

fn same(actual: Option<&Products>, expected: Option<&Products>, name: &str) {
    assert_eq!(actual.map(|p| &p.0), expected.map(|p| &p.0), "{name}");
    assert_eq!(
        serde_json::to_vec(&actual.map(|p| &p.1)).unwrap(),
        serde_json::to_vec(&expected.map(|p| &p.1)).unwrap(),
        "{name}"
    );
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
    let actual = snapshot
        .script_query_products_with_diagnostics_from(state, &owner)
        .unwrap();
    let expected = snapshot
        .script_query_products_with_diagnostics(state)
        .unwrap();
    same(actual.as_ref(), expected.as_ref(), name);
    counts.files += 1;
    if let Some((editor, diagnostics)) = actual {
        counts.responses += 1;
        counts.diagnostics += diagnostics.len();
        assert_eq!(
            Some(editor),
            snapshot.script_query_products(state).unwrap(),
            "{name}"
        );
        let descriptor = state.sfc_descriptor(uri, source).unwrap();
        let root = vize_l0::SourceRoot::new(snapshot.source()).unwrap();
        for block in [descriptor.script.as_ref(), descriptor.script_setup.as_ref()]
            .into_iter()
            .flatten()
        {
            counts.blocks += 1;
            let (frame, _) = script_frame(root, block).unwrap();
            if !frame.source().is_empty()
                && matching_script(&owner, block).is_some_and(|syntax| {
                    clean_checker_program(
                        syntax,
                        frame,
                        ScriptDiagnosticProfile::for_language(block.lang.as_deref()),
                    )
                })
            {
                counts.clean_checker_programs += 1;
                counts.reused_from_incomplete_files += usize::from(owner.admitted().is_none());
            }
        }
    }
}

#[test]
fn exact_original_profiles_keep_complete_editor_and_public_checker_packets() {
    let state = ServerState::new();
    let uri = Url::parse("file:///native/Diagnostics.vue").unwrap();
    let mut counts = Coverage::default();
    for source in [
        "<script setup lang='ts'>import { ref } from 'vue'; const value = ref(1); defineProps<{ title: string }>();</script><template>{{value}}</template>",
        "<script lang='js'>const value: number = 1;</script><script setup lang='ts'>import Child from './Child.vue'; const title = '🌱';</script>",
        "<!-- 雪🌱 -->\r\n<script lang='tsx'>export const view = <span/>;</script>\r\n<script setup lang='ts'>const bad: = ;</script>",
        "<script>const broken = ;</script><script setup lang='ts'>import Child from './Child.vue'; const title = '🌱';</script>",
        "<script>/* @flow */ super;</script>",
        "<script lang='ts'>const a = /x/uv;</script>",
        "<script lang=' tsx '>export default <span/>;</script>",
        "<script lang='coffee'>const value = 2;</script>",
        "<script></script><script setup lang='ts'> </script>",
        "<script lang='ts'/><script setup lang='jsx'>const node = <span/>;</script>",
        "<template><span>🌱</span></template>",
    ] {
        compare(&state, &uri, source, source, &mut counts);
        let snapshot = SourceSnapshotCache::default()
            .capture(&state.documents, &uri)
            .unwrap();
        let arena = Allocator::default();
        let owner = lower_sfc_native(&arena, snapshot.source(), options());
        if let Some((_, actual)) = snapshot
            .script_query_products_with_diagnostics_from(&state, &owner)
            .unwrap()
        {
            let expected: Vec<_> = type_check_sfc(source, &SfcTypeCheckOptions::default())
                .diagnostics
                .into_iter()
                .filter(|error| error.code.as_deref() == Some("script-parse-error"))
                .collect();
            assert_eq!(
                serde_json::to_vec(&actual).unwrap(),
                serde_json::to_vec(&expected).unwrap(),
                "{source}"
            );
        }
    }
    assert!(
        counts.clean_checker_programs > 0
            && counts.reused_from_incomplete_files > 0
            && counts.diagnostics > 0,
        "{counts:?}"
    );
}

#[test]
fn foreign_identical_source_and_equal_version_reopen_do_not_publish() {
    let state = ServerState::new();
    let uri = Url::parse("file:///native/Revision.vue").unwrap();
    let source = "<script setup lang='ts'>const value = 1;</script>";
    state
        .documents
        .open(uri.clone(), source.into(), 7, "vue".into());
    let cache = SourceSnapshotCache::default();
    let snapshot = cache.capture(&state.documents, &uri).unwrap();
    let arena = Allocator::default();
    let copied = source.to_owned();
    let foreign = lower_sfc_native(&arena, &copied, options());
    assert!(matches!(
        snapshot.script_query_products_with_diagnostics_from(&state, &foreign),
        Err(ScriptQueryRefusal::DescriptorSourceMismatch)
    ));
    let owner = lower_sfc_native(&arena, snapshot.source(), options());
    let expected = snapshot
        .script_query_products_with_diagnostics(&state)
        .unwrap();
    same(
        snapshot
            .script_query_products_with_diagnostics_from(&state, &owner)
            .unwrap()
            .as_ref(),
        expected.as_ref(),
        "original",
    );
    state.documents.close(&uri);
    assert!(matches!(
        snapshot.script_query_products_with_diagnostics_from(&state, &owner),
        Err(ScriptQueryRefusal::Host(SnapshotRefusal::MissingDocument))
    ));
    state
        .documents
        .open(uri.clone(), source.into(), 7, "vue".into());
    assert!(matches!(
        snapshot.script_query_products_with_diagnostics_from(&state, &owner),
        Err(ScriptQueryRefusal::Host(SnapshotRefusal::Superseded))
    ));
    let current = cache.capture(&state.documents, &uri).unwrap();
    assert!(matches!(
        current.script_query_products_with_diagnostics_from(&state, &owner),
        Err(ScriptQueryRefusal::DescriptorSourceMismatch)
    ));
}

#[test]
#[ignore = "compares complete packets over every original tracked Vue source"]
fn original_native_editor_and_checker_packets() {
    let manifest = std::fs::read(std::env::var_os("VIZE_SCRIPT_QUERY_INPUTS").unwrap()).unwrap();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let state = ServerState::new();
    let uri = Url::parse("file:///native/Corpus.vue").unwrap();
    let mut counts = Coverage::default();
    for entry in manifest
        .split(|&b| b == 0)
        .filter(|entry| !entry.is_empty())
    {
        let name = std::str::from_utf8(entry).unwrap();
        let bytes = std::fs::read(root.join(name)).unwrap();
        match std::str::from_utf8(&bytes) {
            Ok(source) => compare(&state, &uri, source, name, &mut counts),
            Err(_) => counts.invalid += 1,
        }
    }
    assert!(
        counts.files > 0
            && counts.clean_checker_programs > 0
            && counts.reused_from_incomplete_files > 0,
        "{counts:?}"
    );
    eprintln!("native_editor_checker_corpus {counts:?}");
}

#[cfg(feature = "micro-profiling")]
mod profile;
