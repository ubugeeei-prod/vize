use vize_canon::{SfcBlockType, VirtualProject};

#[test]
fn generated_guard_positions_map_inside_a_short_sfc() {
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
    let virtual_offset = virtual_file
        .content
        .match_indices(guard)
        .last()
        .expect("guarded interpolation should repeat the generated guard")
        .0;
    assert!(
        virtual_offset > source.len(),
        "the regression requires a generated offset past the source EOF"
    );
    let prefix = &virtual_file.content[..virtual_offset];
    let line = prefix.bytes().filter(|byte| *byte == b'\n').count() as u32;
    let column = prefix
        .rsplit_once('\n')
        .map_or(prefix.len(), |(_, tail)| tail.len()) as u32;

    let original = project
        .map_to_original(&virtual_file.virtual_path, line, column)
        .expect("generated guard diagnostics must not be dropped");

    assert_eq!(original.path, source_path);
    assert_eq!(original.block_type, Some(SfcBlockType::Template));
    assert!(original.line < source.lines().count() as u32);
}
