#![expect(clippy::disallowed_methods, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_macros, reason = "fixtures use std strings")]
use super::{
    DeclarationContents, collect_tsconfig_type_packages, reference_type_packages,
    resolve_type_reference_declaration_files, resolve_type_reference_declaration_files_reading,
};
use std::path::{Path, PathBuf};

fn write(root: &Path, rel: &str, content: &str) {
    let path = root.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

fn unique_case_dir(name: &str) -> PathBuf {
    static NEXT_CASE_ID: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let case_id = NEXT_CASE_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "vize-type-reference-{name}-{}-{case_id}",
        std::process::id()
    ))
}

fn relative_paths(root: &Path, files: &[PathBuf]) -> Vec<std::string::String> {
    files
        .iter()
        .map(|path| {
            path.strip_prefix(root)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/")
        })
        .collect()
}

#[test]
fn reference_type_packages_collects_quoted_directives() {
    assert_eq!(
        reference_type_packages(
            r#"
/// <reference types="vitest/importMeta" />
  /// <reference types='@vizejs/vite-plugin-musea/client' />
/// <reference path="./local.d.ts" />
"#,
        ),
        vec![
            "vitest/importMeta".to_string(),
            "@vizejs/vite-plugin-musea/client".to_string(),
        ]
    );
}

#[test]
fn type_reference_resolution_supports_subpaths_exports_and_graphs() {
    let root = unique_case_dir("subpath");
    let _ = std::fs::remove_dir_all(&root);
    write(
        &root,
        "node_modules/vitest/importMeta.d.ts",
        "/// <reference path=\"./globals.d.ts\" />\ndeclare global { interface ImportMeta { vitest: boolean; } }\n",
    );
    write(
        &root,
        "node_modules/vitest/globals.d.ts",
        "declare const describe: (name: string) => void;\n",
    );
    write(
        &root,
        "node_modules/@vizejs/vite-plugin-musea/package.json",
        r#"{ "exports": { "./client": { "types": "./client.d.ts" } } }"#,
    );
    write(
        &root,
        "node_modules/@vizejs/vite-plugin-musea/client.d.ts",
        "declare function defineArt(source: string): void;\n",
    );

    let root = root.canonicalize().unwrap();
    let vitest = resolve_type_reference_declaration_files(&root, "vitest/importMeta");
    assert_eq!(
        relative_paths(&root, &vitest),
        vec![
            "node_modules/vitest/importMeta.d.ts",
            "node_modules/vitest/globals.d.ts",
        ]
    );
    let musea = resolve_type_reference_declaration_files(&root, "@vizejs/vite-plugin-musea/client");
    assert_eq!(
        relative_paths(&root, &musea),
        vec!["node_modules/@vizejs/vite-plugin-musea/client.d.ts"]
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn type_reference_resolution_prefers_nested_exports_types_conditions() {
    let root = unique_case_dir("nested-exports-types");
    let _ = std::fs::remove_dir_all(&root);
    write(
        &root,
        "node_modules/vue-plugin/package.json",
        r#"{
  "exports": {
    "./global-components": {
      "vue": {
        "types": "./types/vue-components.d.ts",
        "default": "./dist/vue-components.js"
      },
      "import": "./dist/global-components.mjs",
      "default": "./dist/global-components.js"
    }
  }
}"#,
    );
    write(
        &root,
        "node_modules/vue-plugin/types/vue-components.d.ts",
        "/// <reference path=\"./shared.d.ts\" />\nexport {};\ndeclare module 'vue' { export interface GlobalComponents { VpButton: unknown } }\n",
    );
    write(
        &root,
        "node_modules/vue-plugin/types/shared.d.ts",
        "export {};\ndeclare global { interface Window { __vp: true } }\n",
    );
    write(
        &root,
        "node_modules/vue-plugin/dist/global-components.mjs",
        "export default {}\n",
    );

    let root = root.canonicalize().unwrap();
    let files = resolve_type_reference_declaration_files(&root, "vue-plugin/global-components");

    assert_eq!(
        relative_paths(&root, &files),
        vec![
            "node_modules/vue-plugin/types/vue-components.d.ts",
            "node_modules/vue-plugin/types/shared.d.ts",
        ]
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn type_reference_resolution_supports_module_declaration_suffixes() {
    let root = unique_case_dir("module-suffixes");
    let _ = std::fs::remove_dir_all(&root);
    write(
        &root,
        "node_modules/modern-env/package.json",
        r#"{ "types": "index.d.mts" }"#,
    );
    write(
        &root,
        "node_modules/modern-env/index.d.mts",
        "/// <reference path=\"./globals.d.cts\" />\nexport {};\n",
    );
    write(
        &root,
        "node_modules/modern-env/globals.d.cts",
        "declare const MODERN_ENV: true;\n",
    );
    write(
        &root,
        "node_modules/modern-subpath/worker.d.cts",
        "declare const WORKER_ENV: true;\n",
    );

    let root = root.canonicalize().unwrap();
    let modern_env = resolve_type_reference_declaration_files(&root, "modern-env");
    assert_eq!(
        relative_paths(&root, &modern_env),
        vec![
            "node_modules/modern-env/index.d.mts",
            "node_modules/modern-env/globals.d.cts",
        ]
    );
    let modern_subpath = resolve_type_reference_declaration_files(&root, "modern-subpath/worker");
    assert_eq!(
        relative_paths(&root, &modern_subpath),
        vec!["node_modules/modern-subpath/worker.d.cts"]
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn tsconfig_type_packages_follow_extends_and_local_overrides() {
    let root = unique_case_dir("tsconfig-types");
    let _ = std::fs::remove_dir_all(&root);
    write(
        &root,
        "base.json",
        r#"{
  "compilerOptions": {
    "types": ["nuxt-i18n", "@nuxtjs/vuetify"]
  }
}"#,
    );
    write(
        &root,
        "tsconfig.inherit.json",
        r#"{
  "extends": "./base.json"
}"#,
    );
    write(
        &root,
        "tsconfig.override.json",
        r#"{
  "extends": "./base.json",
  "compilerOptions": {
    "types": ["@nuxt/types/app"]
  }
}"#,
    );

    assert_eq!(
        collect_tsconfig_type_packages(Some(&root.join("tsconfig.inherit.json"))),
        vec!["nuxt-i18n".to_string(), "@nuxtjs/vuetify".to_string()]
    );
    assert_eq!(
        collect_tsconfig_type_packages(Some(&root.join("tsconfig.override.json"))),
        vec!["@nuxt/types/app".to_string()]
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn reference_directive_lines_are_the_split_lines_that_name_a_reference() {
    for content in [
        "",
        "<reference",
        "/// <reference types=\"node\" />",
        "/// <reference path=\"a.d.ts\" />\r\n/// <reference types=\"vite/client\" />\r\n",
        "export {};\n// a <reference and another <reference on one line\n/// <reference path=\"b.d.ts\" />",
        "/// <reference path=\"c.d.ts\" />\r",
        "const a = 1;\r/// <reference path=\"lone-cr.d.ts\" />\nconst b = 2;\n",
        "// élan\n\t /// <reference types=\"ünicode\" />\n\n<reference\n",
        "\n\n\n/// <reference path=\"last.d.ts\" />\n",
    ] {
        let expected = content
            .lines()
            .filter(|line| line.contains("<reference"))
            .collect::<Vec<_>>();
        assert_eq!(
            super::reference_directive_lines(content).collect::<Vec<_>>(),
            expected,
            "{content:?}"
        );
    }
}

/// The package graph walk keeps the text of every declaration it returns, so
/// the walk that follows the returned files reads none of them again.
#[test]
fn package_graph_walk_keeps_the_text_of_every_declaration_it_returns() {
    let root = unique_case_dir("graph-contents");
    let package = "node_modules/@types/graph";
    write(
        &root,
        &format!("{package}/package.json"),
        r#"{"types":"index.d.ts"}"#,
    );
    let index = "/// <reference path=\"globals.d.ts\" />\n/// <reference path=\"../../outside.d.ts\" />\nexport {}\n";
    write(&root, &format!("{package}/index.d.ts"), index);
    write(
        &root,
        &format!("{package}/globals.d.ts"),
        "declare const g: 1;\n",
    );
    write(&root, "node_modules/outside.d.ts", "declare const o: 1;\n");

    let mut contents = DeclarationContents::default();
    let files = resolve_type_reference_declaration_files_reading(&root, "graph", &mut contents);
    assert_eq!(
        files,
        resolve_type_reference_declaration_files(&root, "graph"),
        "keeping the text changes no result"
    );
    let mut kept = contents.keys().cloned().collect::<Vec<_>>();
    kept.sort();
    let mut returned = files.clone();
    returned.sort();
    assert_eq!(kept, returned);
    for file in &files {
        assert_eq!(
            contents.get(file).map(std::string::String::as_str),
            Some(std::fs::read_to_string(file).unwrap().as_str()),
            "{}",
            file.display()
        );
    }
    std::fs::remove_dir_all(root).unwrap();
}
