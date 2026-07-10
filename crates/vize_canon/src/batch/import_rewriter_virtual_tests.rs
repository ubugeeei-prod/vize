use super::ImportRewriter;
use oxc_span::SourceType;
use std::fs;
use std::path::{Path, PathBuf};
use vize_carton::cstr;

fn unique_case_dir(name: &str) -> PathBuf {
    static NEXT_CASE_ID: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let case_id = NEXT_CASE_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    std::env::temp_dir().join(
        cstr!(
            "vize-import-rewriter-virtual-{name}-{}-{case_id}",
            std::process::id()
        )
        .as_str(),
    )
}

fn write(dir: &Path, rel: &str, contents: &str) -> PathBuf {
    let path = dir.join(rel);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(&path, contents).unwrap();
    path
}

#[test]
fn rewrites_absolute_module_declaration_import_that_needs_vue_virtual_path() {
    let root = unique_case_dir("declaration-with-vue-import");
    let _ = fs::remove_dir_all(&root);
    let feature = write(
        &root,
        "src/feature.d.mts",
        "import Widget from './Widget.vue'\nexport type Feature = typeof Widget\n",
    );
    write(
        &root,
        "src/Widget.vue",
        "<script setup lang=\"ts\">const label = 'ok'</script>",
    );

    let rewriter = ImportRewriter::new();
    let source = cstr!(
        "import type {{ Feature }} from '{}';",
        feature.with_file_name("feature").display()
    );
    let virtual_root = root.join("node_modules/.vize/canon");
    let roots = (root.as_path(), virtual_root.as_path());
    let result =
        rewriter.rewrite_for_virtual_project(source.as_str(), SourceType::ts(), roots, None);

    assert_eq!(
        result.code.as_str(),
        cstr!(
            "import type {{ Feature }} from '{}';",
            virtual_root.join("src/feature").display()
        )
        .as_str()
    );

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn rewrites_absolute_index_module_declaration_import_that_needs_vue_virtual_path() {
    let root = unique_case_dir("index-declaration-with-vue-import");
    let _ = fs::remove_dir_all(&root);
    write(
        &root,
        "src/feature/index.d.cts",
        "import Widget from '../Widget.vue'\nexport type Feature = typeof Widget\n",
    );
    write(
        &root,
        "src/Widget.vue",
        "<script setup lang=\"ts\">const label = 'ok'</script>",
    );

    let rewriter = ImportRewriter::new();
    let source = cstr!(
        "import type {{ Feature }} from '{}';",
        root.join("src/feature").display()
    );
    let virtual_root = root.join("node_modules/.vize/canon");
    let roots = (root.as_path(), virtual_root.as_path());
    let result =
        rewriter.rewrite_for_virtual_project(source.as_str(), SourceType::ts(), roots, None);

    assert_eq!(
        result.code.as_str(),
        cstr!(
            "import type {{ Feature }} from '{}';",
            virtual_root.join("src/feature").display()
        )
        .as_str()
    );

    let _ = fs::remove_dir_all(&root);
}
