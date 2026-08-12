//! Shared project harness for the JSX/TSX component-props tests (#4042).
//!
//! The fixtures written here are byte-identical to the ones the expectations in
//! `tests/jsx_component_props.rs` were recorded against with a real `vue-tsc`
//! run (`vue-tsc -p . --noEmit`, `jsx: "preserve"`, `jsxImportSource: "vue"`,
//! `strict`, `moduleResolution: "bundler"`), so the rows produced here are
//! directly comparable to `vue-tsc`'s output.

use std::path::Path;

use vize_canon::{BatchTypeChecker, BatchTypeCheckerTrait};

/// Diagnostics for the JSX/TSX consumer, as
/// `(relative path, code, "line:column severity message")` with 1-based
/// positions, sorted for a stable full-equality comparison.
///
/// Returns `None` when no type checker is available in this environment, which
/// mirrors the sibling batch integration tests. Only *construction* is that
/// environment probe: scanning and checking are the behavior under test, so a
/// failure there must fail the test instead of turning every case into a no-op.
pub fn consumer_diagnostics(project_root: &Path) -> Option<Vec<(String, Option<u32>, String)>> {
    let mut checker = BatchTypeChecker::new(project_root).ok()?;
    checker.enable_jsx_typecheck();
    checker.scan_project().expect("project should scan");
    let result = checker.check_project().expect("project should type check");

    // The temporary root and the reported path can differ by a symlinked prefix
    // (`/tmp` -> `/private/tmp` on macOS), so canonicalize before stripping.
    let root = std::fs::canonicalize(project_root).unwrap_or_else(|_| project_root.to_path_buf());
    let mut diagnostics: Vec<_> = result
        .diagnostics
        .into_iter()
        .map(|diagnostic| {
            let file =
                std::fs::canonicalize(&diagnostic.file).unwrap_or_else(|_| diagnostic.file.clone());
            (
                file.strip_prefix(&root)
                    .map(|path| path.display().to_string())
                    .unwrap_or_else(|_| file.display().to_string()),
                diagnostic.line,
                diagnostic.column,
                diagnostic.code,
                format!(
                    "{} {}",
                    match diagnostic.severity {
                        1 => "error",
                        2 => "warning",
                        3 => "info",
                        _ => "hint",
                    },
                    diagnostic.message
                ),
            )
        })
        .collect();
    // Sort while line and column are still numeric: ordering the formatted
    // "{line}:{column}" text lexicographically would place line 10 before line 2.
    diagnostics.sort();
    Some(
        diagnostics
            .into_iter()
            .map(|(file, line, column, code, message)| {
                (file, code, format!("{}:{} {message}", line + 1, column + 1))
            })
            .collect(),
    )
}

/// A temporary project with the JSX-enabled `tsconfig.json`, the minimal `vue`
/// type stub and the given `(relative path, source)` files.
pub fn create_project(files: &[(&str, &str)]) -> tempfile::TempDir {
    let project = tempfile::tempdir().expect("temp project should be created");
    write_file(
        project.path(),
        "tsconfig.json",
        r#"{
  "compilerOptions": {
    "allowJs": true,
    "checkJs": true,
    "jsx": "preserve",
    "jsxImportSource": "vue",
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "noEmit": true
  },
  "include": ["src/**/*"]
}"#,
    );
    write_file(
        project.path(),
        "node_modules/vue/package.json",
        r#"{ "name": "vue", "types": "index.d.ts" }"#,
    );
    write_file(
        project.path(),
        "node_modules/vue/index.d.ts",
        r#"export interface Ref<T = unknown> {
  value: T;
}

export function ref<T>(value: T): Ref<T>;

export interface ComponentPublicInstance {
  $attrs: Record<string, unknown>;
  $slots: Record<string, unknown>;
  $refs: Record<string, unknown>;
  $emit: (...args: unknown[]) => void;
}
"#,
    );
    for (path, source) in files {
        write_file(project.path(), path, source);
    }
    project
}

fn write_file(project_root: &Path, path: &str, source: &str) {
    let path = project_root.join(path);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(path, source).unwrap();
}
