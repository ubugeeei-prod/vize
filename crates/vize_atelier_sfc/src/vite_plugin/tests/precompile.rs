use super::super::{
    PrecompileChunkOptions, PrecompileFileMetadata, PrecompileFileMetadataEntry,
    chunk_precompile_files, diff_precompile_files, has_file_metadata_changed,
    normalize_precompile_batch_size,
};
use vize_carton::String;

#[test]
fn snapshots_precompile_diff_and_chunks() {
    let previous = vec![
        metadata_entry("/src/unchanged.vue", 10.0, 100.0),
        metadata_entry("/src/changed.vue", 10.0, 100.0),
        metadata_entry("/src/removed.vue", 10.0, 100.0),
    ];
    let current = vec![
        metadata_entry("/src/unchanged.vue", 10.0, 100.0),
        metadata_entry("/src/changed.vue", 20.0, 100.0),
        metadata_entry("/src/new.vue", 30.0, 50.0),
    ];
    let files = compact_files(&["/src/unchanged.vue", "/src/changed.vue", "/src/new.vue"]);

    assert!(!has_file_metadata_changed(
        previous.first().map(|entry| &entry.metadata),
        &current[0].metadata,
    ));
    assert!(has_file_metadata_changed(
        previous.get(1).map(|entry| &entry.metadata),
        &current[1].metadata,
    ));
    assert_eq!(normalize_precompile_batch_size(Some(3.8)), 3);

    insta::assert_debug_snapshot!(
        diff_precompile_files(&files, &current, &previous),
        @r###"
        PrecompileDiff {
            changed_files: [
                "/src/changed.vue",
                "/src/new.vue",
            ],
            deleted_files: [
                "/src/removed.vue",
            ],
        }
        "###
    );

    insta::assert_debug_snapshot!(
        chunk_precompile_files(
            &compact_files(&["a.vue", "b.vue", "c.vue"]),
            Some(10.0),
            PrecompileChunkOptions {
                max_bytes: Some(10.0),
                metadata: Some(&[
                    metadata_entry("a.vue", 1.0, 4.0),
                    metadata_entry("b.vue", 1.0, 4.0),
                    metadata_entry("c.vue", 1.0, 9.0),
                ]),
            },
        ),
        @r###"
        [
            [
                "a.vue",
                "b.vue",
            ],
            [
                "c.vue",
            ],
        ]
        "###
    );
}

fn metadata_entry(path: &str, mtime_ms: f64, size: f64) -> PrecompileFileMetadataEntry {
    PrecompileFileMetadataEntry {
        path: String::from(path),
        metadata: PrecompileFileMetadata { mtime_ms, size },
    }
}

fn compact_files(files: &[&str]) -> Vec<String> {
    files.iter().map(|file| String::from(*file)).collect()
}
