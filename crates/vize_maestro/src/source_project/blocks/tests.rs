#![expect(
    clippy::unwrap_used,
    reason = "the directed host and original stage are authoritative test inputs"
)]

use super::*;
use crate::source_project::SourceSnapshotCache;
use std::{fs, path::Path};
use tower_lsp::lsp_types::{TextDocumentContentChangeEvent, Url};
use vize_incremental::{
    ParsedSfc, StageConfig, artifact::split_blocks, compute_file_artifacts,
    descriptor::parse_outcome,
};

const STAGES: [BlockStage; 3] = [BlockStage::Source, BlockStage::Surface, BlockStage::Page];

#[derive(Debug, Default)]
struct Counts {
    files: usize,
    rejected: usize,
    invalid_utf8: usize,
    blocks: usize,
    nonempty: usize,
    queries: usize,
    surfaces: usize,
    pages: usize,
    findings: usize,
}

fn compare(
    state: &ServerState,
    cache: &SourceSnapshotCache,
    uri: &Url,
    source: &str,
    totals: &mut Counts,
) {
    state
        .documents
        .open(uri.clone(), source.into(), 7, "vue".into());
    let snapshot = cache.capture(&state.documents, uri).unwrap();
    totals.files += 1;
    let finish = || {
        state.close_document(uri);
        cache.forget(uri);
    };
    if let ParsedSfc::Failed(error) = parse_outcome(uri.path(), source) {
        for stage in STAGES {
            assert_eq!(
                snapshot.open_block_at(state, 0, stage),
                Err(OpenBlockError::Descriptor(error.clone())),
                "{uri}"
            );
        }
        totals.rejected += 1;
        finish();
        return;
    }
    let slots = split_blocks(source);
    let original = compute_file_artifacts(source, StageConfig::default());
    assert_eq!(slots.len(), original.len());
    for (slot, old) in slots.into_iter().zip(original) {
        totals.blocks += 1;
        if slot.source.text.is_empty() {
            continue;
        }
        totals.nonempty += 1;
        let end = slot.start + slot.source.text.len() as u32;
        let last = slot.source.text.char_indices().next_back().unwrap().0 as u32 + slot.start;
        let probes = [slot.start, last];
        let location = Span::new(slot.start, end);
        for byte in probes {
            for stage in STAGES {
                let read = snapshot.open_block_at(state, byte, stage).unwrap().unwrap();
                assert_eq!(
                    (&read.kind, read.ordinal, read.content, read.source_key),
                    (&old.kind, old.ordinal, location, old.source_key),
                    "{uri} {byte}"
                );
                match stage {
                    BlockStage::Source => assert_eq!(read.findings, BlockFindings::SourceOnly),
                    BlockStage::Surface => match (read.findings, old.surface.as_ref()) {
                        (BlockFindings::Surface { key, findings }, Some(expected)) => {
                            assert_eq!((key, &findings), (expected.key, &expected.findings));
                            totals.surfaces += 1;
                            totals.findings += findings.len();
                        }
                        (BlockFindings::Unavailable, None) => {}
                        result => panic!("{uri} {byte} {result:?}"),
                    },
                    BlockStage::Page => match (read.findings, old.page.as_ref()) {
                        (BlockFindings::Page { key, diagnostics }, Some(expected)) => {
                            assert_eq!((key, &diagnostics), (expected.key, &expected.diagnostics));
                            totals.pages += 1;
                            totals.findings += diagnostics.len();
                        }
                        (BlockFindings::Unavailable, None) => {}
                        result => panic!("{uri} {byte} {result:?}"),
                    },
                }
                totals.queries += 1;
            }
        }
    }
    assert_eq!(
        snapshot.open_block_at(state, source.len() as u32, BlockStage::Source),
        Ok(None)
    );
    finish();
    assert_eq!(
        snapshot.open_block_at(state, 0, BlockStage::Page),
        Err(OpenBlockError::Host(SnapshotRefusal::MissingDocument))
    );
}

#[test]
fn original_stages_agree_at_the_first_and_last_character_of_each_editor_block() {
    let state = ServerState::default();
    let cache = SourceSnapshotCache::default();
    let mut totals = Counts::default();
    let uri = Url::parse("file:///actual/Component.vue").unwrap();
    for source in [
        "<template>雪🪴<b>{{x}}</b></template><style>.a{color:v-bind(theme)}</style><i18n>日</i18n>",
        "<template><img></p></template><script setup>const a=1</script><style scoped src='./a.css'/>",
        "<template lang='pug'>div hello</template><style lang='scss'>.a { color: red }</style>",
        "<template></template><template/>",
        "",
        "<script lang='ts'>let t</script>",
    ] {
        compare(&state, &cache, &uri, source, &mut totals);
    }
    assert!(totals.surfaces > 0 && totals.pages > 0);
}

#[test]
fn byte_boundaries_language_and_document_lifetimes_precede_publication() {
    let state = ServerState::default();
    let cache = SourceSnapshotCache::default();
    let uri = Url::parse("file:///actual/Component.vue").unwrap();
    let source = "<template>雪</template>";
    state
        .documents
        .open(uri.clone(), source.into(), 3, "vue".into());
    let saved = cache.capture(&state.documents, &uri).unwrap();
    let offset = source.find('雪').unwrap() as u32;
    assert_eq!(
        saved.open_block_at(&state, offset + 1, BlockStage::Surface),
        Err(OpenBlockError::Position(
            BlockPositionError::NotCharBoundary
        ))
    );
    assert_eq!(
        saved.open_block_at(&state, source.len() as u32 + 1, BlockStage::Surface),
        Err(OpenBlockError::Position(BlockPositionError::OutOfBounds))
    );
    assert_eq!(saved.open_block_at(&state, 0, BlockStage::Source), Ok(None));
    assert!(state.documents.apply_changes(
        &uri,
        vec![TextDocumentContentChangeEvent {
            range: None,
            range_length: None,
            text: "<template>x</template>".into()
        }],
        4
    ));
    assert_eq!(
        saved.open_block_at(&state, offset, BlockStage::Surface),
        Err(OpenBlockError::Host(SnapshotRefusal::Superseded))
    );
    let changed = cache.capture(&state.documents, &uri).unwrap();
    let other = Url::parse("file:///actual/Renamed.vue").unwrap();
    assert!(state.rename_document(&uri, other.clone()));
    assert_eq!(
        changed.open_block_at(&state, offset, BlockStage::Page),
        Err(OpenBlockError::Host(SnapshotRefusal::MissingDocument))
    );
    state.close_document(&other);
    for (path, language) in [
        ("file:///actual/Component.vue", "typescript"),
        ("file:///actual/index.html", "vue"),
    ] {
        let uri = Url::parse(path).unwrap();
        state
            .documents
            .open(uri.clone(), source.into(), 3, language.into());
        let snap = cache.capture(&state.documents, &uri).unwrap();
        assert_eq!(
            snap.open_block_at(&state, offset, BlockStage::Source),
            Err(OpenBlockError::Language)
        );
        state.close_document(&uri);
    }
}

#[test]
#[ignore = "requires the NUL-delimited manifest of original editor documents"]
fn original_editor_documents() {
    let paths = fs::read(std::env::var_os("VIZE_BLOCK_QUERY_MANIFEST").unwrap()).unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let state = ServerState::default();
    let cache = SourceSnapshotCache::default();
    let mut totals = Counts::default();
    for row in paths.split(|&c| c == 0).filter(|p| !p.is_empty()) {
        let path = root.join(std::str::from_utf8(row).unwrap());
        let uri = Url::from_file_path(&path).unwrap();
        let bytes = fs::read(path).unwrap();
        match std::str::from_utf8(&bytes) {
            Ok(source) => compare(&state, &cache, &uri, source, &mut totals),
            Err(_) => totals.invalid_utf8 += 1,
        }
    }
    assert!(totals.blocks > 0 && totals.queries > 0);
    eprintln!("{totals:?}");
}

#[test]
fn a_prepared_block_read_cannot_publish_after_cancellation_or_close() {
    let state = ServerState::default();
    let cache = SourceSnapshotCache::default();
    let uri = Url::parse("file:///actual/Cancelled.vue").unwrap();
    for cancelled in [true, false] {
        state.documents.open(
            uri.clone(),
            "<template><b/></template>".into(),
            1,
            "vue".into(),
        );
        let (query, cancellation) = cache.begin_query(&state.documents, &uri).unwrap();
        let state_ref = &state;
        let result = crate::runtime::block_on(query.run(&state.documents, |snapshot| async move {
            snapshot.open_block_at(state_ref, 10, BlockStage::Page)
        }))
        .unwrap();
        let refusal = if cancelled {
            cancellation.abort();
            SnapshotRefusal::Cancelled
        } else {
            state.close_document(&uri);
            SnapshotRefusal::MissingDocument
        };
        assert_eq!(result.publish(&state.documents, |read| read), Err(refusal));
    }
}
