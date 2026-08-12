//! Template definite-assignment parity with `vue-tsc` (TS2454).
//!
//! Vue evaluates `<script setup>` to completion before the render function can
//! observe any binding, so `vue-tsc` never reports TS2454 for a template read of
//! a setup binding — it reaches those bindings through a `typeof` type query on
//! `__VLS_ctx`, which TypeScript resolves without control-flow analysis. Reads
//! that stay inside the setup body keep exact TypeScript control-flow semantics
//! and still report TS2454.
//!
//! Every expectation below is transcribed from a real `vue-tsc` run over
//! byte-identical fixtures (vue-tsc 3.3.4 / TypeScript 6.0.3 / vue 3.6.0-beta.10,
//! `strict: true`). See the PR body for the recorded output.

use std::path::Path;

use vize_canon::{BatchTypeChecker, BatchTypeCheckerTrait, SfcTypeCheckOptions, type_check_sfc};

#[path = "template_definite_assignment/fixtures.rs"]
mod fixtures;

/// `(file, code, line, column, message)` — the complete diagnostic tuple.
type Diagnostic = (String, Option<u32>, u32, u32, String);

/// The complete diagnostic list `vue-tsc` produces for `fixtures::FIXTURES`.
///
/// Empty for every template read: the seven shapes below cover an immediate
/// watcher callback (Misskey `follow-requests.vue`), a conditional branch whose
/// right-hand side is a top-level `await` (Elk `TimelineHome.vue`), a plain
/// conditional branch, an async callback, a non-immediate watcher, an
/// early-returning initializer, and a binding never assigned at all.
const EXPECTED: &[Diagnostic5] = &[
    // Script-scope reads keep exact TypeScript control-flow semantics.
    (
        "src/ScriptAsyncCallback.vue",
        Some(2454),
        8,
        15,
        "Variable 'followedTags' is used before being assigned.",
    ),
    (
        "src/ScriptConditional.vue",
        Some(2454),
        7,
        13,
        "Variable 'paginator' is used before being assigned.",
    ),
    (
        "src/ScriptNeverAssigned.vue",
        Some(2454),
        4,
        13,
        "Variable 'paginator' is used before being assigned.",
    ),
    (
        "src/ScriptWatchImmediate.vue",
        Some(2454),
        9,
        13,
        "Variable 'paginator' is used before being assigned.",
    ),
    // The template stays fully type-checked: the binding keeps its exact
    // declared type, so a bad member access still reports.
    (
        "src/TemplateStillChecked.vue",
        Some(2339),
        10,
        21,
        "Property 'nope' does not exist on type 'Paginator'.",
    ),
    (
        "src/TemplateVForStillChecked.vue",
        Some(2339),
        10,
        54,
        "Property 'nope' does not exist on type 'string'.",
    ),
    // Options API control: the read stays in the `setup()` body.
    (
        "src/OptionsApiConditional.vue",
        Some(2454),
        11,
        19,
        "Variable 'paginator' is used before being assigned.",
    ),
];

type Diagnostic5 = (&'static str, Option<u32>, u32, u32, &'static str);

#[test]
fn template_definite_assignment_matches_vue_tsc() {
    let project = create_project(fixtures::FIXTURES, VUE_STUB);
    let actual = project_diagnostics(project.path());
    assert_eq!(
        actual,
        expected(),
        "vize must match the recorded vue-tsc run"
    );
}

/// Vue 2.7 dialect control: the same shapes, checked against a Vue 2.7 stub.
#[test]
fn template_definite_assignment_matches_vue_tsc_on_vue_2_7() {
    let project = create_project(fixtures::FIXTURES, VUE_2_7_STUB);
    let actual = project_diagnostics(project.path());
    assert_eq!(
        actual,
        expected(),
        "the Vue 2.7 dialect must reach the same definite-assignment result"
    );
}

/// The shadow must be confined to `<script setup>`: a plain `<script>`
/// module-scope `let` is not a Vue template binding, so emitting a shadow for
/// it would invent a binding Vue does not expose.
#[test]
fn plain_script_module_bindings_get_no_deferred_shadow() {
    let result = type_check_sfc(
        fixtures::PLAIN_SCRIPT_SFC,
        &SfcTypeCheckOptions::new("PlainScriptConditional.vue").with_virtual_ts(),
    );
    let virtual_ts = result.virtual_ts.expect("virtual ts should be generated");
    assert_eq!(
        virtual_ts.matches("__D_paginator").count(),
        0,
        "plain `<script>` bindings must not be shadowed:\n{virtual_ts}"
    );
}

/// The shadow carries the binding's own declared type — no widening, no
/// `undefined`, no `any`.
#[test]
fn deferred_shadow_captures_the_declared_type_verbatim() {
    let result = type_check_sfc(
        fixtures::WATCH_IMMEDIATE_SFC,
        &SfcTypeCheckOptions::new("WatchImmediate.vue").with_virtual_ts(),
    );
    let virtual_ts = result.virtual_ts.expect("virtual ts should be generated");
    assert_eq!(
        virtual_ts
            .matches("  type __D_paginator = typeof paginator;\n")
            .count(),
        1,
        "setup scope should capture the declared type once:\n{virtual_ts}"
    );
    assert_eq!(
        virtual_ts
            .matches("    var paginator: __D_paginator = undefined as any;\n")
            .count(),
        1,
        "template scope should redeclare the binding once:\n{virtual_ts}"
    );
}

/// A `lang="tsx"` setup body must be scanned as TSX: parsed as plain
/// TypeScript, the JSX ahead of the declaration derails the parse and the
/// binding silently loses its shadow.
#[test]
fn tsx_setup_bindings_get_the_deferred_shadow() {
    let result = type_check_sfc(
        fixtures::TSX_CONDITIONAL_SFC,
        &SfcTypeCheckOptions::new("TsxConditional.vue").with_virtual_ts(),
    );
    let virtual_ts = result.virtual_ts.expect("virtual ts should be generated");
    assert_eq!(
        virtual_ts
            .matches("  type __D_paginator = typeof paginator;\n")
            .count(),
        1,
        "a TSX setup body should still capture the declared type:\n{virtual_ts}"
    );
}

/// A `const` binding keeps its setup narrowing: it is never shadowed.
#[test]
fn initialized_bindings_get_no_deferred_shadow() {
    let result = type_check_sfc(
        fixtures::DIRECT_INIT_SFC,
        &SfcTypeCheckOptions::new("DirectInit.vue").with_virtual_ts(),
    );
    let virtual_ts = result.virtual_ts.expect("virtual ts should be generated");
    assert_eq!(
        virtual_ts.matches("__D_").count(),
        0,
        "an initialized binding needs no shadow:\n{virtual_ts}"
    );
}

/// A `let x!: T` is already definitely assigned as far as TypeScript is
/// concerned, so it needs no shadow either.
#[test]
fn definite_assertion_bindings_get_no_deferred_shadow() {
    let result = type_check_sfc(
        fixtures::DEFINITE_ASSERTION_SFC,
        &SfcTypeCheckOptions::new("DefiniteAssertion.vue").with_virtual_ts(),
    );
    let virtual_ts = result.virtual_ts.expect("virtual ts should be generated");
    assert_eq!(
        virtual_ts.matches("__D_").count(),
        0,
        "a `!`-asserted binding needs no shadow:\n{virtual_ts}"
    );
}

fn expected() -> Vec<Diagnostic> {
    let mut rows: Vec<Diagnostic> = EXPECTED
        .iter()
        .map(|&(file, code, line, column, message)| {
            (file.to_owned(), code, line, column, message.to_owned())
        })
        .collect();
    rows.sort();
    rows
}

fn project_diagnostics(project_root: &Path) -> Vec<Diagnostic> {
    let mut checker = BatchTypeChecker::new(project_root).expect("batch checker should be created");
    checker.scan_project().expect("project should scan");
    let result = checker.check_project().expect("project should type check");

    // macOS resolves `/tmp` through a symlink, so the reported paths are
    // canonical while `TempDir` hands back the symlinked root.
    let root = std::fs::canonicalize(project_root).unwrap_or_else(|_| project_root.to_path_buf());
    let mut rows: Vec<Diagnostic> = result
        .diagnostics
        .into_iter()
        .map(|diagnostic| {
            (
                relative_path(&root, &diagnostic.file),
                diagnostic.code,
                diagnostic.line + 1,
                diagnostic.column + 1,
                diagnostic.message.to_string(),
            )
        })
        .collect();
    rows.sort();
    rows
}

fn create_project(files: &[(&str, &str)], vue_stub: &str) -> tempfile::TempDir {
    let project = tempfile::tempdir().expect("temp project should be created");
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
    write_file(project.path(), "node_modules/vue/index.d.ts", vue_stub);
    write_file(project.path(), "src/support.ts", fixtures::SUPPORT_TS);
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

fn relative_path(root: &Path, file: &Path) -> String {
    file.strip_prefix(root)
        .map(|path| path.display().to_string())
        .unwrap_or_else(|_| file.display().to_string())
}

const VUE_STUB: &str = r#"export interface Ref<T = unknown> { value: T }
export function ref<T>(value: T): Ref<T>;
export function watch<T>(
  source: Ref<T>,
  callback: (value: T, previous: T | undefined) => unknown,
  options?: { immediate?: boolean },
): void;
export function onMounted(callback: () => unknown): void;
export function defineComponent<T>(options: T): T;
"#;

const VUE_2_7_STUB: &str = r#"export interface Ref<T = unknown> { value: T }
export function ref<T>(value: T): Ref<T>;
export function watch<T>(
  source: Ref<T>,
  callback: (value: T, previous: T | undefined) => unknown,
  options?: { immediate?: boolean },
): void;
export function onMounted(callback: () => unknown): void;
export function defineComponent<T>(options: T): T;
export const version: '2.7.16';
"#;
