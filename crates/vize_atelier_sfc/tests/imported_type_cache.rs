#![cfg(feature = "compile")]

use std::path::{Path, PathBuf};
use std::time::Duration;

use vize_atelier_sfc::{
    SfcCompileOptions, SfcCompileResult, SfcParseOptions, begin_type_resolution_batch, compile_sfc,
    parse_sfc,
};
use vize_carton::ToCompactString;

fn temp_project_dir() -> PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "vize-sfc-imported-type-cache-{}-{nonce}",
        std::process::id()
    ))
}

fn compile_imported_props(source: &str, component: &Path) -> SfcCompileResult {
    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("SFC should parse");
    let mut options = SfcCompileOptions::default();
    options.script.id = Some(component.to_string_lossy().as_ref().to_compact_string());
    compile_sfc(&descriptor, options).expect("SFC should compile")
}

fn rewrite_with_newer_mtime(path: &Path, content: &str) {
    let previous_mtime = std::fs::metadata(path)
        .and_then(|metadata| metadata.modified())
        .expect("type file should have an mtime");
    std::fs::write(path, content).unwrap();
    std::fs::File::options()
        .write(true)
        .open(path)
        .unwrap()
        .set_times(std::fs::FileTimes::new().set_modified(previous_mtime + Duration::from_secs(1)))
        .unwrap();
}

#[test]
fn single_compile_revalidates_imported_types_after_batch() {
    let project = temp_project_dir();
    std::fs::create_dir_all(&project).unwrap();
    let component = project.join("App.vue");
    let types = project.join("types.ts");
    let source = r#"<script setup lang="ts">
import type { Props } from './types'
defineProps<Props>()
</script>"#;

    std::fs::write(&types, "export interface Props { first: string }\n").unwrap();
    let batch = {
        let _batch = begin_type_resolution_batch();
        compile_imported_props(source, &component)
    };
    assert!(batch.code.contains("first: {"), "{}", batch.code);

    // Same-length replacement: only metadata revalidation can detect it.
    rewrite_with_newer_mtime(&types, "export interface Props { other: string }\n");
    let hmr = compile_imported_props(source, &component);
    assert!(hmr.code.contains("other: {"), "{}", hmr.code);
    assert!(!hmr.code.contains("first: {"), "{}", hmr.code);

    let _ = std::fs::remove_dir_all(project);
}
