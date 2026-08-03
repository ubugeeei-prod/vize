#[path = "support/corsa_requirement.rs"]
mod corsa_requirement;

use std::{path::Path, process::Command};

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root should exist")
}

fn unique_case_dir(name: &str) -> std::path::PathBuf {
    static NEXT_CASE_ID: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let case_id = NEXT_CASE_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    workspace_root()
        .join("target")
        .join("vize-tests")
        .join("tests")
        .join(format!("{name}-{}-{case_id}", std::process::id()))
}

/// Mirror the workspace dependencies into a **real** `node_modules` directory.
///
/// A `node_modules` that is itself a symlink made the Corsa CLI drop every
/// diagnostic for the project (fixed separately in #3374), which silently turned
/// the `errorCount` assertions below into no-ops. Installed trees keep a real
/// directory at `node_modules` and link the individual packages inside it, so
/// this reproduces that shape and the assertions stay load-bearing.
///
/// `.vize` is deliberately not mirrored: that is where vize writes the virtual
/// TypeScript for the project, and linking it out to the shared workspace copy
/// puts the generated files behind a symlink again — the same layout that
/// swallows the diagnostics, plus a collision with every concurrent case.
fn mirror_workspace_node_modules(project_root: &Path) {
    let source = workspace_root().join("node_modules");
    let target = project_root.join("node_modules");
    if target.exists() {
        return;
    }
    std::fs::create_dir_all(&target).unwrap();
    let Ok(entries) = std::fs::read_dir(&source) else {
        return;
    };
    for entry in entries.flatten() {
        if entry.file_name() == std::ffi::OsStr::new(".vize") {
            continue;
        }
        let link = target.join(entry.file_name());
        #[cfg(unix)]
        let _ = std::os::unix::fs::symlink(entry.path(), &link);
        #[cfg(windows)]
        let _ = std::os::windows::fs::symlink_dir(entry.path(), &link);
    }
}

fn create_cli_project(name: &str, files: &[(&str, &str)]) -> std::path::PathBuf {
    let project_root = unique_case_dir(name);
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(&project_root).unwrap();
    mirror_workspace_node_modules(&project_root);
    std::fs::write(
        project_root.join("tsconfig.json"),
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
    )
    .unwrap();

    for (path, source) in files {
        let file_path = project_root.join(path);
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(file_path, source).unwrap();
    }

    project_root
}

fn resolve_test_corsa_path() -> Option<String> {
    let workspace_root = workspace_root();
    let sibling_cache = workspace_root.parent()?.join("corsa-bind/.cache/tsgo");
    if sibling_cache.exists() {
        return Some(sibling_cache.display().to_string());
    }

    let workspace_bin = workspace_root.join("node_modules/.bin/tsgo");
    workspace_bin
        .exists()
        .then(|| workspace_bin.display().to_string())
}

fn run_check_json(project_root: &Path, corsa_path: &str) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project_root)
        .env("CORSA_PATH", corsa_path)
        .args(["check", ".", "--format", "json"])
        .output()
        .unwrap()
}

#[test]
fn check_preserves_named_exports_from_normal_script_vue() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let project_root = create_cli_project(
        "normal-script-named-exports",
        &[
            (
                "src/components/ParseMdFileDialog.vue",
                r#"<script lang="ts">
import { defineComponent } from "vue";

export default defineComponent({
  name: "ParseMdFileDialog",
});

export const setupParseMdFileDialogCtx = () => ({ ready: true });

export enum DiffDisplayMode {
  Unified = 'unified',
  Split = 'split',
}

export enum DiffRevision {
  Head,
  Base,
}

export const enum DiffMarker {
  Added = 'added',
}

export class DiffCursor {
  line = 0;
}
</script>
"#,
            ),
            (
                "src/pages/Consumer.vue",
                r#"<script setup lang="ts">
import {
  DiffCursor,
  DiffDisplayMode,
  DiffMarker,
  DiffRevision,
  setupParseMdFileDialogCtx,
} from "../components/ParseMdFileDialog.vue";

const ctx = setupParseMdFileDialogCtx();
const ready: boolean = ctx.ready;

// Every value+type declaration has to survive the plain-<script> export bridge
// in *both* declaration spaces: the annotation is the type side, the
// initializer the value side.
const mode: DiffDisplayMode = DiffDisplayMode.Unified;
const revision: DiffRevision = DiffRevision.Head;
const marker: DiffMarker = DiffMarker.Added;
const cursor: DiffCursor = new DiffCursor();

// The enum type must stay nominal rather than widen to its member primitives,
// which is what a `typeof`-only bridge would have produced.
const line: number = cursor.line;
</script>

<template>
  <div>{{ ready }} {{ mode }} {{ revision }} {{ marker }} {{ cursor }} {{ line }}</div>
</template>
"#,
            ),
        ],
    );

    let output = run_check_json(&project_root, &corsa_path);
    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    let stderr = std::str::from_utf8(&output.stderr).unwrap();
    assert!(
        output.status.success(),
        "check failed:\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );

    let json: serde_json::Value = serde_json::from_str(stdout).unwrap();
    assert_eq!(json["errorCount"], 0, "{stdout}\n{stderr}");
    assert!(!stdout.contains("TS2614"), "{stdout}\n{stderr}");
    assert!(!stdout.contains("TS2749"), "{stdout}\n{stderr}");

    let _ = std::fs::remove_dir_all(&project_root);
}

/// The mirror of the test above: a value-only export must not be handed a type
/// meaning it never had. `TS2749` here is the *correct* answer, and its presence
/// also proves the harness above is not passing vacuously on a project whose
/// diagnostics were dropped.
#[test]
fn check_reports_value_only_plain_script_exports_used_as_types() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let project_root = create_cli_project(
        "normal-script-value-only-exports",
        &[
            (
                "src/components/ValueOnly.vue",
                r#"<script lang="ts">
import { defineComponent } from "vue";

export default defineComponent({
  name: "ValueOnly",
});

export const pageSize = 20;

export function toPageCount(total: number) {
  return Math.ceil(total / pageSize);
}
</script>
"#,
            ),
            (
                "src/pages/ValueOnlyConsumer.vue",
                r#"<script setup lang="ts">
import { pageSize, toPageCount } from "../components/ValueOnly.vue";

const size: number = pageSize;
const count: number = toPageCount(100);

const badSize: pageSize = 20;
const badCount: toPageCount = 5;
</script>

<template>
  <div>{{ size }} {{ count }} {{ badSize }} {{ badCount }}</div>
</template>
"#,
            ),
        ],
    );

    // No `status.success()` assertion here: reporting the errors is the point,
    // and `vize check` exits non-zero when it finds any.
    let output = run_check_json(&project_root, &corsa_path);
    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    let stderr = std::str::from_utf8(&output.stderr).unwrap();

    let json: serde_json::Value = serde_json::from_str(stdout).unwrap();
    assert_eq!(json["errorCount"], 2, "{stdout}\n{stderr}");
    assert!(
        stdout.contains("'pageSize' refers to a value, but is being used as a type here"),
        "an exported const must not gain a type meaning:\n{stdout}\n{stderr}"
    );
    assert!(
        stdout.contains("'toPageCount' refers to a value, but is being used as a type here"),
        "an exported function must not gain a type meaning:\n{stdout}\n{stderr}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}
