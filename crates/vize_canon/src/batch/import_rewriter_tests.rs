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
            "vize-import-rewriter-{name}-{}-{case_id}",
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
fn test_rewrite_default_import() {
    let rewriter = ImportRewriter::new();
    let source = r#"import App from './App.vue';"#;
    let result = rewriter.rewrite(source, SourceType::ts());

    assert_eq!(result.code, r#"import App from './App.vue.ts';"#);
}

#[test]
fn test_rewrite_named_import() {
    let rewriter = ImportRewriter::new();
    let source = r#"import { helper, type Props } from './helper.vue';"#;
    let result = rewriter.rewrite(source, SourceType::ts());

    assert_eq!(
        result.code,
        r#"import { helper, type Props } from './helper.vue.ts';"#
    );
}

#[test]
fn test_rewrite_side_effect_import() {
    let rewriter = ImportRewriter::new();
    let source = r#"import './global.vue';"#;
    let result = rewriter.rewrite(source, SourceType::ts());

    assert_eq!(result.code, r#"import './global.vue.ts';"#);
}

#[test]
fn test_no_rewrite_npm_import() {
    let rewriter = ImportRewriter::new();
    let source = r#"import { ref } from 'vue';"#;
    let result = rewriter.rewrite(source, SourceType::ts());

    assert_eq!(result.code, r#"import { ref } from 'vue';"#);
}

#[test]
fn test_rewrite_alias_import() {
    let rewriter = ImportRewriter::new();
    let source = r#"import App, { type Props } from '@/App.vue';"#;
    let result = rewriter.rewrite(source, SourceType::ts());

    assert_eq!(
        result.code,
        r#"import App, { type Props } from '@/App.vue.ts';"#
    );
}

#[test]
fn test_rewrite_absolute_export_from_for_virtual_project() {
    let rewriter = ImportRewriter::new();
    let source = r#"export * from '/p/src/App.vue';"#;
    let roots = (Path::new("/p"), Path::new("/p/v"));
    let result = rewriter.rewrite_for_virtual_project(source, SourceType::ts(), roots);
    assert_eq!(result.code, r#"export * from '/p/v/src/App.vue.ts';"#);
}

#[cfg(unix)]
#[test]
fn rewrite_absolute_vue_specifier_through_symlinked_project_path() {
    let real = tempfile::tempdir().unwrap();
    let link_parent = tempfile::tempdir().unwrap();
    let link = link_parent.path().join("project-link");
    std::os::unix::fs::symlink(real.path(), &link).unwrap();
    std::fs::create_dir_all(real.path().join("src")).unwrap();
    std::fs::write(real.path().join("src/App.vue"), "<template />").unwrap();

    let source = cstr!(r#"export * from "{}";"#, link.join("src/App.vue").display());
    let virtual_root = real.path().join("node_modules/.vize/canon");
    let roots = (
        vize_carton::path::canonicalize_non_verbatim(&link),
        virtual_root.clone(),
    );
    let result = ImportRewriter::new().rewrite_for_virtual_project(
        source.as_str(),
        SourceType::ts(),
        (roots.0.as_path(), roots.1.as_path()),
    );

    assert_eq!(
        result.code.as_str(),
        cstr!(
            r#"export * from "{}";"#,
            virtual_root.join("src/App.vue.ts").display()
        )
        .as_str()
    );
}

#[test]
fn test_keeps_plain_absolute_generated_graphql_import_for_virtual_project() {
    let root = unique_case_dir("plain-generated-graphql");
    let _ = fs::remove_dir_all(&root);
    let schema = write(
        &root,
        "types/codegen/schema.ts",
        "// Generated GraphQL schema types.\nexport enum Kind { List = 'LIST' }\n",
    );
    let rewriter = ImportRewriter::new();
    let source = cstr!(
        "import type {{ Kind }} from '{}';",
        schema.with_extension("").display()
    );
    let virtual_root = root.join("node_modules/.vize/canon");
    let roots = (root.as_path(), virtual_root.as_path());
    let result = rewriter.rewrite_for_virtual_project(source.as_str(), SourceType::ts(), roots);
    assert_eq!(
        result.code.as_str(),
        cstr!(
            "import type {{ Kind }} from '{}';",
            schema.with_extension("").display()
        )
        .as_str()
    );

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn test_rewrite_absolute_ts_import_that_needs_vue_rewrite_for_virtual_project() {
    let root = unique_case_dir("ts-with-vue-import");
    let _ = fs::remove_dir_all(&root);
    let feature = write(
        &root,
        "src/feature.ts",
        "import { Widget } from './nested'\nexport { Widget }\n",
    );
    write(
        &root,
        "src/nested.ts",
        "import Widget from './Widget.vue'\nexport { Widget }\n",
    );
    write(
        &root,
        "src/Widget.vue",
        "<script setup lang=\"ts\">const label = 'ok'</script>",
    );
    let rewriter = ImportRewriter::new();
    let source = cstr!(
        "import {{ Widget }} from '{}';",
        feature.with_extension("").display()
    );
    let virtual_root = root.join("node_modules/.vize/canon");
    let roots = (root.as_path(), virtual_root.as_path());
    let result = rewriter.rewrite_for_virtual_project(source.as_str(), SourceType::ts(), roots);
    assert_eq!(
        result.code.as_str(),
        cstr!(
            "import {{ Widget }} from '{}';",
            virtual_root.join("src/feature").display()
        )
        .as_str()
    );

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn test_keeps_absolute_assets_for_virtual_project() {
    let rewriter = ImportRewriter::new();
    let source = r#"import '/p/assets/theme.css';"#;
    let roots = (Path::new("/p"), Path::new("/p/v"));
    let result = rewriter.rewrite_for_virtual_project(source, SourceType::ts(), roots);
    assert_eq!(result.code, source);
}

#[test]
fn test_keeps_absolute_node_modules_for_virtual_project() {
    let rewriter = ImportRewriter::new();
    let source = r#"import '/p/node_modules/pkg/index.js';"#;
    let roots = (Path::new("/p"), Path::new("/p/v"));
    let result = rewriter.rewrite_for_virtual_project(source, SourceType::ts(), roots);
    assert_eq!(result.code, source);
}

#[test]
fn test_rewrite_dynamic_import() {
    let rewriter = ImportRewriter::new();
    let source = r#"const App = () => import('./App.vue');"#;
    let result = rewriter.rewrite(source, SourceType::ts());

    assert_eq!(result.code, r#"const App = () => import('./App.vue.ts');"#);
}

#[test]
fn test_rewrite_parent_path() {
    let rewriter = ImportRewriter::new();
    let source = r#"import Parent from '../Parent.vue';"#;
    let result = rewriter.rewrite(source, SourceType::ts());

    assert_eq!(result.code, r#"import Parent from '../Parent.vue.ts';"#);
}

#[test]
fn test_source_map_offset() {
    let rewriter = ImportRewriter::new();
    let source = r#"import App from './App.vue';
import { ref } from 'vue';
const x = 1;"#;
    let result = rewriter.rewrite(source, SourceType::ts());

    let virtual_offset = 30;
    let original_offset = result.source_map.get_original_offset(virtual_offset);

    assert!(original_offset < virtual_offset);
}

#[test]
fn test_collect_relative_vue_specifiers() {
    let rewriter = ImportRewriter::new();
    let source = r#"import App from './App.vue';
import Sibling from '../shared/Sibling.vue';
import Aliased from '@/Aliased.vue';
import { ref } from 'vue';
import Lazy from './App.vue';
const Lazy2 = () => import('./Lazy.vue');
export { default as Re } from './Re.vue';
"#;
    let mut found = rewriter.collect_relative_vue_specifiers(source, SourceType::ts());
    found.sort();
    assert_eq!(
        found.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
        [
            "../shared/Sibling.vue",
            "./App.vue",
            "./Lazy.vue",
            "./Re.vue"
        ]
    );
}

#[test]
fn test_multiple_rewrites() {
    let rewriter = ImportRewriter::new();
    let source = r#"import App from './App.vue';
import Child from './Child.vue';
import { ref } from 'vue';"#;
    let result = rewriter.rewrite(source, SourceType::ts());

    assert_eq!(
        result.code,
        r#"import App from './App.vue.ts';
import Child from './Child.vue.ts';
import { ref } from 'vue';"#
    );
}
