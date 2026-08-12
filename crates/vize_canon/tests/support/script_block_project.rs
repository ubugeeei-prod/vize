//! Shared project harness for the classic-`<script>` / `<script setup>` scope
//! tests.
//!
//! Every expectation those tests assert was recorded from a real
//! `vue-tsc` 3.3.4 + `vue` 3.6.0-beta.10 run over byte-identical fixtures, so
//! the formatted rows produced here are directly comparable to `vue-tsc`'s
//! `file(line,column): error TSxxxx: message` output.

use std::path::Path;

use vize_canon::{BatchTypeChecker, BatchTypeCheckerTrait};

const TSCONFIG: &str = r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "noEmit": true
  },
  "include": ["src/**/*"]
}"#;

const VUE_STUB: &str = r#"export interface Ref<T = any> {
  value: T;
}
export interface ShallowRef<T = any> {
  value: T;
}
export declare function ref<T>(value: T): Ref<T>;
export declare function computed<T>(getter: () => T): Ref<T>;
export declare function defineComponent(options: any): any;
export interface ComponentPublicInstance {
  $attrs: Record<string, unknown>;
  $slots: Record<string, unknown>;
  $refs: Record<string, unknown>;
  $emit: (...args: unknown[]) => void;
}
"#;

/// Type-check `files` as one project and return every diagnostic formatted as
/// `path(line,column): error TSxxxx: message`, sorted, with authored 1-based
/// positions — the exact shape `vue-tsc` prints.
pub fn check(files: &[(&str, &str)]) -> Vec<String> {
    let project = tempfile::tempdir().expect("temporary project should be created");
    let root = project.path();
    write_file(root, "tsconfig.json", TSCONFIG);
    write_file(
        root,
        "node_modules/vue/package.json",
        r#"{ "name": "vue", "types": "index.d.ts" }"#,
    );
    write_file(root, "node_modules/vue/index.d.ts", VUE_STUB);
    for (path, source) in files {
        write_file(root, path, source);
    }

    let mut checker = BatchTypeChecker::new(root).expect("type checker should start");
    checker.scan_project().expect("project should scan");
    let result = checker.check_project().expect("project should type check");
    // The checker reports canonical paths; `tempdir()` may hand back a symlink.
    let root = &std::fs::canonicalize(root).unwrap_or_else(|_| root.to_path_buf());

    let mut rows: Vec<String> = result
        .diagnostics
        .iter()
        .map(|diagnostic| {
            let path = diagnostic
                .file
                .strip_prefix(root)
                .unwrap_or(&diagnostic.file)
                .to_string_lossy()
                .replace('\\', "/");
            let severity = match diagnostic.severity {
                1 => "error",
                2 => "warning",
                3 => "info",
                _ => "hint",
            };
            let code = diagnostic
                .code
                .map(|code| format!("TS{code}"))
                .unwrap_or_else(|| String::from("TS0"));
            format!(
                "{path}({},{}): {severity} {code}: {}",
                diagnostic.line + 1,
                diagnostic.column + 1,
                diagnostic.message
            )
        })
        .collect();
    rows.sort();
    rows
}

fn write_file(root: &Path, path: &str, source: &str) {
    let path = root.join(path);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("parent directory should be created");
    }
    std::fs::write(path, source).expect("fixture should be written");
}
