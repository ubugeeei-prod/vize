#![expect(clippy::string_slice, reason = "tests assert by panicking")]
use vize_canon::{SfcBlockType, VirtualProject};

#[test]
fn authored_condition_owns_diagnostics_and_inference_guards_are_unmapped() {
    let temp_dir = tempfile::tempdir().expect("temp project should be created");
    let project_root = temp_dir.path().canonicalize().unwrap();
    let source_path = project_root.join("App.vue");
    let source = r#"<script setup lang="ts">
const value = { kind: 'x' as 'x' | 'y' }
</script>
<template><div v-if="value.kind === 'x'">{{ value.kind }}</div></template>
"#;
    std::fs::write(&source_path, source).unwrap();

    let mut project = VirtualProject::new(&project_root).unwrap();
    project.register_path(&source_path).unwrap();
    let virtual_file = project.find_by_original(&source_path).unwrap();
    let guard = "value.kind === 'x'";
    let virtual_offsets: Vec<_> = virtual_file
        .content
        .match_indices(guard)
        .map(|(offset, _)| offset)
        .collect();
    assert!(virtual_offsets.len() > 1, "inference must repeat the guard");
    assert!(
        virtual_offsets.iter().all(|offset| *offset > source.len()),
        "the regression requires a generated offset past the source EOF"
    );
    let mapped: Vec<_> = virtual_offsets
        .into_iter()
        .filter_map(|offset| {
            let prefix = &virtual_file.content[..offset];
            let line = prefix.bytes().filter(|byte| *byte == b'\n').count() as u32;
            let column = prefix
                .rsplit_once('\n')
                .map_or(prefix.len(), |(_, tail)| tail.len()) as u32;
            project.map_to_original(&virtual_file.virtual_path, line, column)
        })
        .collect();
    assert_eq!(
        mapped.len(),
        1,
        "only the authored condition owns diagnostics"
    );
    let original = &mapped[0];

    assert_eq!(original.path, source_path);
    assert_eq!(original.block_type, Some(SfcBlockType::Template));
    assert_eq!(original.line, 3);
    assert_eq!(
        original.column as usize,
        source.lines().nth(3).unwrap().find(guard).unwrap()
    );
}
