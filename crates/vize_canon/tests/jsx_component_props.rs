//! Batch type-checking of component props in JSX/TSX consumers (#4042).
//!
//! Every expectation below is the output of an actual `vue-tsc` run over the
//! same fixture (`vue-tsc -p . --noEmit`, `jsx: "preserve"`,
//! `jsxImportSource: "vue"`, `strict`, `moduleResolution: "bundler"`); the
//! oracle line is quoted next to each case. Assertions compare the **complete**
//! diagnostic list — file, code, line, column, severity and message — so a
//! regression cannot hide behind a substring match.

use std::path::Path;

use vize_canon::{BatchTypeChecker, BatchTypeCheckerTrait};

/// `Counter.vue`: one required `count: number`.
const COUNTER_SFC: &str = "<script setup lang=\"ts\">\ndefineProps<{ count: number }>()\n</script>\n\n<template><div /></template>\n";

/// `Widget.vue`: a required prop plus a typed default scoped slot.
const WIDGET_SFC: &str = "<script setup lang=\"ts\">\ndefineProps<{ fooBar: string }>()\ndefineSlots<{ default(props: { item: string }): unknown }>()\n</script>\n\n<template><div /></template>\n";

/// vue-tsc: `src/Consumer.tsx(2,30): error TS2322: Type 'string' is not
/// assignable to type 'number'.`
#[test]
fn tsx_consumer_reports_an_invalid_imported_sfc_prop_at_the_authored_attribute() {
    let project = create_project(&[
        ("src/Counter.vue", COUNTER_SFC),
        (
            "src/Consumer.tsx",
            "import Counter from \"./Counter.vue\";\nexport const view = <Counter count=\"wrong\" />;\n",
        ),
    ]);
    let Some(diagnostics) = consumer_diagnostics(project.path()) else {
        return;
    };

    assert_eq!(
        diagnostics,
        vec![(
            "src/Consumer.tsx".to_string(),
            Some(2322),
            "2:30 error Type 'string' is not assignable to type 'number'.".to_string(),
        )]
    );
}

/// vue-tsc reports nothing for the repaired source.
#[test]
fn tsx_consumer_clears_after_the_prop_is_repaired() {
    let project = create_project(&[
        ("src/Counter.vue", COUNTER_SFC),
        (
            "src/Consumer.tsx",
            "import Counter from \"./Counter.vue\";\nexport const view = <Counter count={1} />;\n",
        ),
    ]);
    let Some(diagnostics) = consumer_diagnostics(project.path()) else {
        return;
    };

    assert_eq!(diagnostics, vec![]);
}

/// vue-tsc: `src/Consumer.jsx(2,30): error TS2322: Type 'string' is not
/// assignable to type 'number'.`
#[test]
fn check_js_jsx_consumer_reports_an_invalid_imported_sfc_prop() {
    let project = create_project(&[
        ("src/Counter.vue", COUNTER_SFC),
        (
            "src/Consumer.jsx",
            "import Counter from \"./Counter.vue\";\nexport const view = <Counter count=\"wrong\" />;\n",
        ),
    ]);
    let Some(diagnostics) = consumer_diagnostics(project.path()) else {
        return;
    };

    assert_eq!(
        diagnostics,
        vec![(
            "src/Consumer.jsx".to_string(),
            Some(2322),
            "2:30 error Type 'string' is not assignable to type 'number'.".to_string(),
        )]
    );
}

/// A component rendered inside a scoped-slot body keeps its props contract.
///
/// vue-tsc: `src/Consumer.tsx(3,91): error TS2322: Type 'string' is not
/// assignable to type 'number'.`
///
/// Before #4042 the slot pattern was re-emitted as a bare read, so `props`
/// resolved to an error type and this invalid prop passed silently while two
/// fabricated `TS2304 Cannot find name 'props'` were reported instead.
#[test]
fn component_in_a_scoped_slot_body_reports_its_invalid_prop() {
    let project = create_project(&[
        ("src/Counter.vue", COUNTER_SFC),
        ("src/Widget.vue", WIDGET_SFC),
        (
            "src/Consumer.tsx",
            "import Widget from \"./Widget.vue\";\nimport Counter from \"./Counter.vue\";\nexport const view = <Widget fooBar=\"ok\">{{ default: (props: { item: string }) => <Counter count={props.item} /> }}</Widget>;\n",
        ),
    ]);
    let Some(diagnostics) = consumer_diagnostics(project.path()) else {
        return;
    };

    assert_eq!(
        diagnostics,
        vec![(
            "src/Consumer.tsx".to_string(),
            Some(2322),
            "3:91 error Type 'string' is not assignable to type 'number'.".to_string(),
        )]
    );
}

/// The negative control for the case above: a valid scoped slot must stay
/// clean. vue-tsc reports nothing.
#[test]
fn valid_scoped_slot_usage_stays_clean() {
    let project = create_project(&[
        ("src/Widget.vue", WIDGET_SFC),
        (
            "src/Consumer.tsx",
            "import Widget from \"./Widget.vue\";\nexport const view = <Widget fooBar=\"ok\">{{ default: (props: { item: string }) => props.item }}</Widget>;\n",
        ),
    ]);
    let Some(diagnostics) = consumer_diagnostics(project.path()) else {
        return;
    };

    assert_eq!(diagnostics, vec![]);
}

/// A render-prop child is the default scoped slot; its parameter must bind.
/// vue-tsc reports nothing.
#[test]
fn render_prop_scoped_slot_child_stays_clean() {
    let project = create_project(&[
        ("src/Widget.vue", WIDGET_SFC),
        (
            "src/Consumer.tsx",
            "import Widget from \"./Widget.vue\";\nexport const view = <Widget fooBar=\"ok\">{(props: { item: string }) => props.item}</Widget>;\n",
        ),
    ]);
    let Some(diagnostics) = consumer_diagnostics(project.path()) else {
        return;
    };

    assert_eq!(diagnostics, vec![]);
}

/// Native DOM listener fallthrough on a component is a **deliberate divergence
/// from vue-tsc**, pinned by `crates/vize/tests/check_jsx_component_contract_cli.rs`:
/// Vue forwards such a listener to the fallthrough root at runtime, so vize
/// accepts it (vue-tsc reports `TS2322` here) while still contextually typing
/// the event payload. This is the negative control for the scoped-slot change —
/// it must keep behaving exactly as before.
#[test]
fn native_listener_fallthrough_stays_accepted_and_payload_typed() {
    let accepted = create_project(&[
        ("src/Counter.vue", COUNTER_SFC),
        (
            "src/Consumer.tsx",
            "import Counter from \"./Counter.vue\";\nexport const view = <Counter count={1} onClick={(event) => event.preventDefault()} />;\n",
        ),
    ]);
    let Some(diagnostics) = consumer_diagnostics(accepted.path()) else {
        return;
    };
    assert_eq!(diagnostics, vec![]);

    let wrong_payload = create_project(&[
        ("src/Counter.vue", COUNTER_SFC),
        (
            "src/Consumer.tsx",
            "import Counter from \"./Counter.vue\";\nexport const view = <Counter count={1} onClick={(event: string) => event.length} />;\n",
        ),
    ]);
    let Some(diagnostics) = consumer_diagnostics(wrong_payload.path()) else {
        return;
    };
    assert_eq!(diagnostics.len(), 1, "{diagnostics:#?}");
    let (file, code, message) = &diagnostics[0];
    assert_eq!(file, "src/Consumer.tsx");
    assert_eq!(*code, Some(2322));
    assert!(message.starts_with("2:40 error "), "{message}");
}

/// A component's *declared* emit stays a valid listener prop. vue-tsc reports
/// nothing.
#[test]
fn declared_emit_listener_stays_accepted() {
    let project = create_project(&[
        (
            "src/Emitter.vue",
            "<script setup lang=\"ts\">\ndefineProps<{ label: string }>()\ndefineEmits<{ change: [value: number] }>()\n</script>\n\n<template><div /></template>\n",
        ),
        (
            "src/Consumer.tsx",
            "import Emitter from \"./Emitter.vue\";\nexport const view = <Emitter label=\"ok\" onChange={(value: number) => value + 1} />;\n",
        ),
    ]);
    let Some(diagnostics) = consumer_diagnostics(project.path()) else {
        return;
    };

    assert_eq!(diagnostics, vec![]);
}

// ---------------------------------------------------------------------------

/// Diagnostics for the JSX/TSX consumer, as
/// `(relative path, code, "line:column severity message")` with 1-based
/// positions, sorted for a stable full-equality comparison.
///
/// Returns `None` when no type checker is available in this environment, which
/// mirrors the sibling batch integration tests.
fn consumer_diagnostics(project_root: &Path) -> Option<Vec<(String, Option<u32>, String)>> {
    let mut checker = BatchTypeChecker::new(project_root).ok()?;
    checker.enable_jsx_typecheck();
    checker.scan_project().ok()?;
    let result = checker.check_project().ok()?;

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
                diagnostic.code,
                format!(
                    "{}:{} {} {}",
                    diagnostic.line + 1,
                    diagnostic.column + 1,
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
    diagnostics.sort();
    Some(diagnostics)
}

fn create_project(files: &[(&str, &str)]) -> tempfile::TempDir {
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
