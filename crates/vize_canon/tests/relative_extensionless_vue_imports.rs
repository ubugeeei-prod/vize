//! Relative extensionless `.vue` imports (#3329).
//!
//! Webpack-era apps spell SFC imports without the extension
//! (`import svgIcon from './components/common/svg'`). TypeScript appends
//! extensions to the specifier and never tries `.vue`, so it resolved to
//! nothing: the component typed as `any`, every prop contract at the usage site
//! silently disappeared, and the `TS2307` that would have made the hole visible
//! was excused by the on-disk-sibling suppression. The import rewriter now
//! redirects such a specifier onto the target's mirror module, the same
//! resolution the alias-mapped spelling gets from its `paths` candidate (#3300).

use std::path::{Path, PathBuf};

use vize_canon::{BatchTypeChecker, BatchTypeCheckerTrait};
use vize_s0::{String, ToCompactString};

const CHILD: &str = r#"<script setup lang="ts">
defineProps<{ count: number }>()
</script>
<template><div /></template>
"#;

const APP: &str = r#"<script setup lang="ts">
import Child from './components/Child'
const label: number = 'not a number'
</script>
<template>
  <Child :count="'not a number'" />
  <span>{{ label }}</span>
</template>
"#;

/// A diagnostic reduced to `(file suffix, code, line, column, message)`.
/// Lines and columns are zero-based authored positions in the `.vue` source.
type Located = (String, Option<u32>, u32, u32, String);

#[test]
fn extensionless_sfc_import_enforces_the_component_contract_at_authored_ranges() {
    let project = create_project(&[("src/App.vue", APP), ("src/components/Child.vue", CHILD)]);
    let diagnostics = project_diagnostics(project.path(), None);

    // Redirecting the specifier lengthens the virtual module at the import, so
    // both a diagnostic before the shift (the template usage is emitted after
    // the script, but authored above it) and one after it must still land on
    // their exact authored positions.
    assert_eq!(
        diagnostics,
        vec![
            (
                "src/App.vue".into(),
                Some(2322),
                2,
                6,
                "Type 'string' is not assignable to type 'number'.".into(),
            ),
            (
                "src/App.vue".into(),
                Some(2322),
                5,
                10,
                "Type 'string' is not assignable to type 'number'.".into(),
            ),
        ],
        "the redirected import must enforce the prop contract at `count` (line 6, column 11 \
         as authored) without shifting the script diagnostic off `label`"
    );
}

#[test]
fn an_ordinary_sibling_module_still_wins_over_a_same_named_sfc() {
    // The redirect may only turn a failing resolution into a successful one:
    // `./Widget` already resolves to `Widget.ts`, which must keep winning.
    let app = r#"<script setup lang="ts">
import Widget from './Widget'
const widget: Record<string, never> = Widget
void widget
</script>
<template><div /></template>
"#;
    let project = create_project(&[
        ("src/App.vue", app),
        (
            "src/Widget.ts",
            "export default { fromTheScriptModule: true };\n",
        ),
        ("src/Widget.vue", CHILD),
    ]);

    // The sentinel property only exists on the script module: had the redirect
    // fired, `Widget` would be the SFC's component type instead.
    let diagnostics = project_diagnostics(project.path(), None);
    let [(file, code, line, column, message)] = diagnostics.as_slice() else {
        panic!("exactly one assignment must be rejected: {diagnostics:#?}");
    };
    assert_eq!(
        (file.as_str(), *code, *line, *column),
        ("src/App.vue", Some(2322), 2, 6)
    );
    assert!(
        message.starts_with(
            "Type '{ fromTheScriptModule: boolean; }' is not assignable to type \
             'Record<string, never>'."
        ),
        "an extensionless specifier TypeScript already resolves must keep its spelling: {message}"
    );
}

#[test]
fn a_genuinely_missing_relative_module_still_reports_ts2307() {
    let app = r#"<script setup lang="ts">
import Absent from './Absent'
void Absent
</script>
<template><div /></template>
"#;
    let project = create_project(&[("src/App.vue", app)]);
    let codes: Vec<Option<u32>> = project_diagnostics(project.path(), None)
        .into_iter()
        .map(|(_, code, ..)| code)
        .collect();

    assert_eq!(
        codes,
        vec![Some(2307)],
        "a specifier with no target on disk must stay loud"
    );
}

#[test]
fn a_partial_subset_run_still_suppresses_unregistered_siblings() {
    // `vize check src/App.vue` registers only the named file, so siblings that
    // exist on disk but sit outside the subset resolve for `tsc` and not for the
    // virtual project. Both the redirected SFC spelling and a plain script
    // sibling must stay suppressed.
    let app = r#"<script setup lang="ts">
import Child from './components/Child'
import { label } from './util'
void Child
void label
</script>
<template><div /></template>
"#;
    let project = create_project(&[
        ("src/App.vue", app),
        ("src/components/Child.vue", CHILD),
        ("src/util.ts", "export const label = 'ok';\n"),
    ]);
    let diagnostics = project_diagnostics(project.path(), Some(&["src/App.vue"]));

    assert!(
        diagnostics.is_empty(),
        "a partial subset must not report its unregistered siblings: {diagnostics:#?}"
    );
}

fn create_project(files: &[(&str, &str)]) -> tempfile::TempDir {
    let project = tempfile::tempdir().expect("temporary project should be created");
    write_file(
        project.path(),
        "tsconfig.json",
        r#"{
  "compilerOptions": {
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
        r#"export interface ComponentPublicInstance {
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

/// Type check `root`, either the whole project or only `subset`, and reduce every
/// diagnostic to a project-relative [`Located`] tuple.
fn project_diagnostics(root: &Path, subset: Option<&[&str]>) -> Vec<Located> {
    let mut checker = BatchTypeChecker::new(root).expect("type checker should start");
    match subset {
        Some(paths) => {
            // The checker canonicalizes its root, so a subset path must be
            // canonical too or it will not mirror under the virtual root.
            let canonical = std::fs::canonicalize(root).expect("project root should canonicalize");
            let paths: Vec<PathBuf> = paths.iter().map(|path| canonical.join(path)).collect();
            checker.scan_paths(&paths).expect("subset should scan");
        }
        None => checker.scan_project().expect("project should scan"),
    }
    let mut diagnostics: Vec<Located> = checker
        .check_project()
        .expect("project should type check")
        .diagnostics
        .into_iter()
        .map(|diagnostic| {
            (
                relative_file(root, &diagnostic.file),
                diagnostic.code,
                diagnostic.line,
                diagnostic.column,
                diagnostic.message,
            )
        })
        .collect();
    diagnostics.sort();
    diagnostics
}

/// Diagnostic paths are absolute and the project root is a symlinked temporary
/// directory on macOS, so compare the trailing project-relative segments.
fn relative_file(root: &Path, file: &Path) -> String {
    let root = root
        .file_name()
        .and_then(|name| name.to_str())
        .expect("temporary project should have a name");
    let file = file.to_string_lossy();
    match file.split_once(root) {
        Some((_, relative)) => relative.trim_start_matches(['/', '\\']).to_compact_string(),
        None => file.to_compact_string(),
    }
}

fn write_file(root: &Path, path: &str, source: &str) {
    let path = root.join(path);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("parent directory should be created");
    }
    std::fs::write(path, source).expect("fixture should be written");
}
