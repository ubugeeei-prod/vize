use super::*;
use vize_s0::path::canonicalize_non_verbatim;

fn write(dir: &Path, rel: &str, contents: &str) -> PathBuf {
    let path = dir.join(rel);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(&path, contents).unwrap();
    path
}

#[test]
fn collects_dynamic_imports_with_magic_comments() {
    let root = std::env::temp_dir().join(cstr!("vize-imports-magic-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("src")).unwrap();

    let entry = write(
        &root,
        "src/entry.ts",
        r#"const panel = import(
  /* webpackChunkName: "settings-panel" */
  "./Panel.vue"
)
void panel
"#,
    );
    let panel = write(
        &root,
        "src/Panel.vue",
        "<script setup lang=\"ts\">\nconst ready = true\n</script>\n<template>{{ ready }}</template>\n",
    );

    let discovered = collect_transitive_local_imports(
        &[entry],
        &root,
        &mut CanonicalPathCache::default(),
        false,
        None,
    );

    assert_eq!(
        discovered.registrations,
        vec![canonicalize_non_verbatim(&panel)]
    );

    let _ = std::fs::remove_dir_all(&root);
}
