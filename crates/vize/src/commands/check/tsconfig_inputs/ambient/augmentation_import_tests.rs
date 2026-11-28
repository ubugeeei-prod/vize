#![expect(clippy::disallowed_methods, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_macros, reason = "fixtures use std strings")]
//! Sources that import a package plugin's module augmentation.

use std::path::{Path, PathBuf};

use super::{collect_ambient_declaration_files, collect_hidden_ambient_declaration_files};
use crate::commands::check::tsconfig_inputs::TsconfigInputCache;

fn write(root: &Path, rel: &str, content: &str) {
    let path = root.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

fn relative_paths(root: &Path, files: &[PathBuf]) -> Vec<String> {
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
fn ambient_collection_keeps_sources_that_import_plugin_augmentations() {
    let case = tempfile::tempdir().unwrap();
    let root = case.path().to_path_buf();
    write(
        &root,
        "src/main.ts",
        "import dayjs from 'dayjs'\nimport relativeTime from 'dayjs/plugin/relativeTime'\ndayjs.extend(relativeTime)\n",
    );
    write(
        &root,
        "src/debounce.ts",
        "import debounce from 'lodash-es/debounce'\nexport { debounce }\n",
    );
    write(
        &root,
        "src/store.ts",
        "import { defineStore } from 'pinia'\nexport const useStore = defineStore('s', {})\n",
    );
    write(&root, "src/App.vue", "<template />\n");
    write(
        &root,
        "node_modules/dayjs/package.json",
        r#"{ "types": "index.d.ts" }"#,
    );
    write(
        &root,
        "node_modules/dayjs/index.d.ts",
        "export interface Dayjs {}\n",
    );
    write(
        &root,
        "node_modules/dayjs/plugin/relativeTime.d.ts",
        "import { PluginFunc } from 'dayjs'\ndeclare const plugin: PluginFunc\nexport = plugin\ndeclare module 'dayjs' {\n  interface Dayjs { fromNow(): string }\n}\n",
    );
    write(
        &root,
        "node_modules/lodash-es/debounce.d.ts",
        "export default function debounce(): void\n",
    );
    // A package root entry is imported by its API consumers themselves.
    write(
        &root,
        "node_modules/pinia/package.json",
        r#"{ "types": "index.d.ts" }"#,
    );
    write(
        &root,
        "node_modules/pinia/index.d.ts",
        "export declare function defineStore(id: string, options: object): unknown\ndeclare module 'vue' {\n  interface ComponentCustomProperties { $pinia: unknown }\n}\n",
    );
    write(
        &root,
        "tsconfig.json",
        r#"{ "include": ["src/**/*.ts", "src/**/*.vue"] }"#,
    );

    let project_root = root.canonicalize().unwrap();
    let tsconfig = project_root.join("tsconfig.json");
    let files = collect_ambient_declaration_files(
        &project_root,
        Some(&tsconfig),
        &mut TsconfigInputCache::default(),
    );
    assert_eq!(relative_paths(&project_root, &files), vec!["src/main.ts"]);
    // A default run already loads every visible source.
    let hidden = collect_hidden_ambient_declaration_files(
        &project_root,
        Some(&tsconfig),
        &mut TsconfigInputCache::default(),
    );
    assert_eq!(hidden, Vec::<PathBuf>::new());
}
