//! `vize check` over plain-`<script>` SFCs that declare a `namespace`.
//!
//! Covers #3383: the namespace used to stay inside the generated `__setup()`
//! function, which TypeScript rejects with TS1235, and the export bridge never
//! reached it so consumers got TS2614 as well. These cases run the real Corsa
//! CLI so the diagnostics come from a type checker rather than from string
//! inspection of the generated module.

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
/// diagnostic for the project (fixed in #3374), which silently turned the
/// `errorCount` assertions below into no-ops — that is how this whole family of
/// defects stayed hidden. `.vize` is deliberately excluded: that is where vize
/// writes the project's virtual TypeScript, so linking it out would put the
/// generated files behind a symlink again and re-hide the diagnostics.
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

fn run_check_json(project_root: &Path, corsa_path: &str) -> serde_json::Value {
    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project_root)
        .env("CORSA_PATH", corsa_path)
        .args(["check", ".", "--format", "json"])
        .output()
        .unwrap();
    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    serde_json::from_str(stdout).unwrap_or_else(|error| {
        panic!(
            "check should print JSON ({error}):\nstdout:\n{stdout}\nstderr:\n{}",
            std::str::from_utf8(&output.stderr).unwrap()
        )
    })
}

/// Every reported diagnostic line, so a case can assert the complete diagnostic
/// set instead of probing for one substring.
fn diagnostics(report: &serde_json::Value) -> Vec<String> {
    let Some(files) = report["files"].as_array() else {
        return Vec::new();
    };
    files
        .iter()
        .flat_map(|file| file["diagnostics"].as_array().cloned().unwrap_or_default())
        .map(|diagnostic| diagnostic.as_str().unwrap_or_default().to_string())
        .collect()
}

#[test]
fn check_accepts_plain_script_namespaces_used_as_values_and_types() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let project_root = create_cli_project(
        "namespace-plain-script",
        &[
            (
                "src/components/Namespaces.vue",
                r#"<script lang="ts">
const localBase = 10;

namespace Bare {
  export const label = "bare";
}

export namespace Config {
  export type Kind = "wide" | "tall";
  export const spacing = 4;
}

export namespace Nest.Inner {
  export const depth = 2;
}

export namespace Twice {
  export const one = 1;
}
export namespace Twice {
  export type Two = string;
}

export class Cursor {
  line = 0;
}
export namespace Cursor {
  export type Anchor = number;
}

export function build() {
  return localBase;
}
export namespace build {
  export const version = 1;
}

export const captured = 7;
export namespace Derived {
  export const total = captured + localBase;
}

export default { name: "Namespaces" };
</script>

<template>
  <div>{{ Bare.label }} {{ Config.spacing }}</div>
</template>
"#,
            ),
            (
                "src/pages/Consumer.vue",
                r#"<script setup lang="ts">
import {
  build,
  captured,
  Config,
  Cursor,
  Derived,
  Nest,
  Twice,
} from "../components/Namespaces.vue";

// Namespace as a value and as a type container.
const spacing: number = Config.spacing;
const kind: Config.Kind = "wide";
const depth: number = Nest.Inner.depth;

// Both blocks of a merged namespace have to survive the relocation.
const one: number = Twice.one;
const two: Twice.Two = "s";

// A namespace merged with a class keeps the class's own meanings.
const cursor: Cursor = new Cursor();
const anchor: Cursor.Anchor = cursor.line;

// A namespace merged with a function keeps the call signature.
const built: number = build();
const version: number = build.version;

// A namespace body reading a plain-script binding still type-checks.
const total: number = Derived.total;
const capturedValue: number = captured;
</script>

<template>
  <div>
    {{ spacing }} {{ kind }} {{ depth }} {{ one }} {{ two }} {{ cursor }}
    {{ anchor }} {{ built }} {{ version }} {{ total }} {{ capturedValue }}
  </div>
</template>
"#,
            ),
        ],
    );

    let report = run_check_json(&project_root, &corsa_path);
    assert_eq!(diagnostics(&report), Vec::<String>::new());
    assert_eq!(report["errorCount"], 0);

    let _ = std::fs::remove_dir_all(&project_root);
}

/// A namespace that was never exported must not gain an export on the way out of
/// `__setup()`: the relocation is verbatim, so `Bare` stays module-local and the
/// consumer's import is a real error.
#[test]
fn check_reports_an_import_of_an_unexported_plain_script_namespace() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let project_root = create_cli_project(
        "namespace-plain-script-unexported",
        &[
            (
                "src/components/Local.vue",
                r#"<script lang="ts">
namespace Bare {
  export const label = "bare";
}

export default { name: "Local", data: () => ({ label: Bare.label }) };
</script>
"#,
            ),
            (
                "src/pages/LocalConsumer.vue",
                r#"<script setup lang="ts">
import { Bare } from "../components/Local.vue";

const label: string = Bare.label;
</script>

<template>
  <div>{{ label }}</div>
</template>
"#,
            ),
        ],
    );

    let report = run_check_json(&project_root, &corsa_path);
    assert_eq!(
        diagnostics(&report),
        vec![
            "error:2:10 [TS2614] Module '\"../components/Local.vue\"' has no exported member 'Bare'. Did you mean to use 'import Bare from \"../components/Local.vue\"' instead?"
                .to_string()
        ]
    );
    assert_eq!(report["errorCount"], 1);

    let _ = std::fs::remove_dir_all(&project_root);
}

/// The legacy `module` keyword is relocated unchanged rather than rewritten to
/// `namespace`. TypeScript's TS1540 for it is the authored code's diagnostic, and
/// its presence also proves the harness above is not passing vacuously on a
/// project whose diagnostics were dropped.
#[test]
fn check_reports_the_legacy_module_keyword_from_a_plain_script() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let project_root = create_cli_project(
        "namespace-plain-script-legacy-module",
        &[(
            "src/components/Legacy.vue",
            r#"<script lang="ts">
export module Legacy {
  export const a = 1;
}

export default { name: "Legacy" };
</script>
"#,
        )],
    );

    let report = run_check_json(&project_root, &corsa_path);
    assert_eq!(
        diagnostics(&report),
        vec![
            "error:2:15 [TS1540] A 'namespace' declaration should not be declared using the 'module' keyword. Please use the 'namespace' keyword instead."
                .to_string()
        ]
    );
    assert_eq!(report["errorCount"], 1);

    let _ = std::fs::remove_dir_all(&project_root);
}
