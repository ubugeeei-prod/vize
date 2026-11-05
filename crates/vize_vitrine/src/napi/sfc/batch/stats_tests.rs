//! Aggregate stats must equal independently compiled configured inputs.

use super::{BatchCompileOptionsNapi, compile_sfc_batch};
use std::path::Path;

fn write(root: &Path, file: &str, source: &str) {
    let path = root.join(file);
    std::fs::create_dir_all(path.parent().expect("parent")).expect("fixture directory");
    std::fs::write(path, source).expect("fixture input");
}

#[test]
fn imported_type_stats_do_not_fold_filename_selected_owners() {
    let root = tempfile::tempdir().expect("reference project");
    let source = "<script setup lang=\"ts\">import type { FooProps } from '@lib/FooProps'; const props = defineProps<FooProps>();</script><template><div>{{ props.bar }}</div></template>";
    write(
        root.path(),
        "tsconfig.json",
        r#"{"files":[],"references":[{"path":"./first"},{"path":"./second"}]}"#,
    );
    for (config, file, field_type) in [
        ("first", "Card", "Record<string, unknown>"),
        ("second", "Pane", "boolean"),
    ] {
        write(
            root.path(),
            &format!("{config}/tsconfig.json"),
            &format!(
                r#"{{"include":["../app/{file}.vue"],"compilerOptions":{{"paths":{{"@lib/*":["./types/*"]}}}}}}"#
            ),
        );
        write(
            root.path(),
            &format!("{config}/types/FooProps.ts"),
            &format!("export interface FooProps {{ bar?: {field_type} }}"),
        );
        write(root.path(), &format!("app/{file}.vue"), source);
    }
    let options = || {
        Some(BatchCompileOptionsNapi {
            threads: Some(1),
            ..Default::default()
        })
    };
    let first = compile_sfc_batch(
        root.path()
            .join("app/Card.vue")
            .to_string_lossy()
            .into_owned(),
        options(),
    )
    .expect("first independent native compile");
    let second = compile_sfc_batch(
        root.path()
            .join("app/Pane.vue")
            .to_string_lossy()
            .into_owned(),
        options(),
    )
    .expect("second independent native compile");
    assert_eq!((first.success, first.failed), (1, 0));
    assert_eq!((second.success, second.failed), (1, 0));
    assert_ne!(first.output_bytes, second.output_bytes);
    let together = compile_sfc_batch(
        root.path().join("app/*.vue").to_string_lossy().into_owned(),
        options(),
    )
    .expect("combined native compile");
    assert_eq!((together.success, together.failed), (2, 0));
    assert_eq!(
        together.output_bytes,
        first.output_bytes + second.output_bytes
    );
}
