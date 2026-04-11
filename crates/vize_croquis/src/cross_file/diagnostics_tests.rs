//! Tests for cross-file diagnostics.

use super::{CrossFileDiagnostic, CrossFileDiagnosticKind, DiagnosticSeverity};
use crate::cross_file::FileId;
use vize_carton::append;

fn make_file_id() -> FileId {
    FileId::new(0)
}

// ============================================================
// Test: Diagnostic code() method returns correct identifiers
// ============================================================

#[test]
fn test_diagnostic_codes() {
    // Create diagnostics and check their codes
    let diag = CrossFileDiagnostic::new(
        CrossFileDiagnosticKind::UnmatchedInject { key: "test".into() },
        DiagnosticSeverity::Error,
        make_file_id(),
        0,
        "test",
    );
    assert_eq!(diag.code(), "vize:croquis/cf/unmatched-inject");

    // Provide/Inject without Symbol
    let diag_provide = CrossFileDiagnostic::new(
        CrossFileDiagnosticKind::ProvideInjectWithoutSymbol {
            key: "test".into(),
            is_provide: true,
        },
        DiagnosticSeverity::Warning,
        make_file_id(),
        0,
        "test",
    );
    assert_eq!(
        diag_provide.code(),
        "vize:croquis/cf/provide-without-symbol"
    );

    let diag_inject = CrossFileDiagnostic::new(
        CrossFileDiagnosticKind::ProvideInjectWithoutSymbol {
            key: "test".into(),
            is_provide: false,
        },
        DiagnosticSeverity::Warning,
        make_file_id(),
        0,
        "test",
    );
    assert_eq!(diag_inject.code(), "vize:croquis/cf/inject-without-symbol");

    // Circular dependency
    let diag_circular = CrossFileDiagnostic::new(
        CrossFileDiagnosticKind::CircularReactiveDependency {
            cycle: vec!["a".into(), "b".into()],
        },
        DiagnosticSeverity::Error,
        make_file_id(),
        0,
        "test",
    );
    assert_eq!(
        diag_circular.code(),
        "vize:croquis/cf/circular-reactive-dependency"
    );

    // Watch can be computed
    let diag_watch = CrossFileDiagnostic::new(
        CrossFileDiagnosticKind::WatchMutationCanBeComputed {
            watch_source: "count".into(),
            mutated_target: "doubled".into(),
            suggested_computed: "const doubled = computed(() => count.value * 2)".into(),
        },
        DiagnosticSeverity::Info,
        make_file_id(),
        0,
        "test",
    );
    assert_eq!(diag_watch.code(), "vize:croquis/cf/watch-can-be-computed");

    // DOM access without nextTick
    let diag_dom = CrossFileDiagnostic::new(
        CrossFileDiagnosticKind::DomAccessWithoutNextTick {
            api: "document.getElementById".into(),
            context: "setup".into(),
        },
        DiagnosticSeverity::Warning,
        make_file_id(),
        0,
        "test",
    );
    assert_eq!(
        diag_dom.code(),
        "vize:croquis/cf/dom-access-without-next-tick"
    );

    // Browser API in SSR
    let diag_ssr = CrossFileDiagnostic::new(
        CrossFileDiagnosticKind::BrowserApiInSsr {
            api: "localStorage".into(),
            context: "setup".into(),
        },
        DiagnosticSeverity::Warning,
        make_file_id(),
        0,
        "test",
    );
    assert_eq!(diag_ssr.code(), "vize:croquis/cf/browser-api-ssr");
}

// ============================================================
// Test: CrossFileDiagnostic builder methods
// ============================================================

#[test]
fn test_diagnostic_builder() {
    let diag = CrossFileDiagnostic::new(
        CrossFileDiagnosticKind::UnmatchedInject {
            key: "theme".into(),
        },
        DiagnosticSeverity::Error,
        make_file_id(),
        100,
        "No provider found for 'theme'",
    )
    .with_suggestion("Add provide('theme', value) in a parent component")
    .with_related(FileId::new(1), 200, "Consumer location");

    assert!(diag.suggestion.is_some());
    assert_eq!(diag.related_files.len(), 1);
    assert_eq!(diag.primary_offset, 100);
}

// ============================================================
// Test: to_markdown() generates readable output
// ============================================================

#[test]
fn test_to_markdown_destructuring() {
    let diag = CrossFileDiagnostic::new(
        CrossFileDiagnosticKind::DestructuringBreaksReactivity {
            source_name: "props".into(),
            destructured_keys: vec!["count".into(), "name".into()],
            suggestion: "toRefs".into(),
        },
        DiagnosticSeverity::Warning,
        make_file_id(),
        0,
        "Destructuring props loses reactivity",
    );

    let markdown = diag.to_markdown();

    insta::assert_snapshot!(markdown.as_str());
}

#[test]
fn test_to_markdown_circular_dependency() {
    let diag = CrossFileDiagnostic::new(
        CrossFileDiagnosticKind::CircularReactiveDependency {
            cycle: vec!["a".into(), "b".into(), "c".into()],
        },
        DiagnosticSeverity::Error,
        make_file_id(),
        0,
        "Circular dependency detected",
    );

    let markdown = diag.to_markdown();

    insta::assert_snapshot!(markdown.as_str());
}

#[test]
fn test_to_markdown_provide_inject_without_symbol() {
    let diag = CrossFileDiagnostic::new(
        CrossFileDiagnosticKind::ProvideInjectWithoutSymbol {
            key: "user".into(),
            is_provide: true,
        },
        DiagnosticSeverity::Warning,
        make_file_id(),
        0,
        "provide() uses string key",
    );

    let markdown = diag.to_markdown();

    insta::assert_snapshot!(markdown.as_str());
}

#[test]
fn test_to_markdown_watch_can_be_computed() {
    let diag = CrossFileDiagnostic::new(
        CrossFileDiagnosticKind::WatchMutationCanBeComputed {
            watch_source: "count".into(),
            mutated_target: "doubled".into(),
            suggested_computed: "const doubled = computed(() => count.value * 2)".into(),
        },
        DiagnosticSeverity::Info,
        make_file_id(),
        0,
        "watch can be replaced with computed",
    );

    let markdown = diag.to_markdown();

    insta::assert_snapshot!(markdown.as_str());
}

#[test]
fn test_to_markdown_dom_access_without_next_tick() {
    let diag = CrossFileDiagnostic::new(
        CrossFileDiagnosticKind::DomAccessWithoutNextTick {
            api: "document.getElementById('app')".into(),
            context: "setup".into(),
        },
        DiagnosticSeverity::Warning,
        make_file_id(),
        0,
        "DOM access in setup without nextTick",
    );

    let markdown = diag.to_markdown();

    insta::assert_snapshot!(markdown.as_str());
}

// ============================================================
// Test: Severity levels
// ============================================================

#[test]
fn test_severity_badges() {
    let kinds = [
        DiagnosticSeverity::Error,
        DiagnosticSeverity::Warning,
        DiagnosticSeverity::Info,
        DiagnosticSeverity::Hint,
    ];

    for severity in kinds {
        let diag = CrossFileDiagnostic::new(
            CrossFileDiagnosticKind::UnmatchedInject { key: "test".into() },
            severity,
            make_file_id(),
            0,
            "test",
        );

        let markdown = diag.to_markdown();
        insta::assert_snapshot!(markdown.as_str());
    }
}

// ============================================================
// Test: Reference escape scenarios (Rust-like tracking)
// ============================================================

#[test]
fn test_reactive_reference_escapes() {
    let diag = CrossFileDiagnostic::new(
        CrossFileDiagnosticKind::ReactiveReferenceEscapes {
            variable_name: "state".into(),
            escaped_via: "function call".into(),
            target_name: Some("processState".into()),
        },
        DiagnosticSeverity::Warning,
        make_file_id(),
        0,
        "Reactive reference escapes scope",
    );

    let markdown = diag.to_markdown();

    insta::assert_snapshot!(markdown.as_str());
}

#[test]
fn test_reactive_object_mutated_after_escape() {
    let diag = CrossFileDiagnostic::new(
        CrossFileDiagnosticKind::ReactiveObjectMutatedAfterEscape {
            variable_name: "data".into(),
            mutation_site: 200,
            escape_site: 100,
        },
        DiagnosticSeverity::Warning,
        make_file_id(),
        0,
        "Reactive object mutated after escape",
    );

    let markdown = diag.to_markdown();

    insta::assert_snapshot!(markdown.as_str());
}

// ============================================================
// Snapshot Tests
// ============================================================

#[test]
fn test_snapshot_all_diagnostic_kinds() {
    use insta::assert_snapshot;

    let file_id = make_file_id();

    let diagnostics = vec![
        // Provide/Inject
        CrossFileDiagnostic::new(
            CrossFileDiagnosticKind::UnmatchedInject {
                key: "theme".into(),
            },
            DiagnosticSeverity::Error,
            file_id,
            100,
            "No provider found for inject('theme')",
        ),
        CrossFileDiagnostic::new(
            CrossFileDiagnosticKind::UnusedProvide {
                key: "config".into(),
            },
            DiagnosticSeverity::Warning,
            file_id,
            50,
            "provide('config') is never injected",
        ),
        CrossFileDiagnostic::new(
            CrossFileDiagnosticKind::ProvideInjectTypeMismatch {
                key: "user".into(),
                provided_type: "Ref<User>".into(),
                injected_type: "User".into(),
            },
            DiagnosticSeverity::Warning,
            file_id,
            200,
            "Type mismatch between provide and inject",
        ),
        // Emit related
        CrossFileDiagnostic::new(
            CrossFileDiagnosticKind::UndeclaredEmit {
                emit_name: "update".into(),
            },
            DiagnosticSeverity::Error,
            file_id,
            300,
            "emit('update') is not declared in defineEmits",
        ),
        CrossFileDiagnostic::new(
            CrossFileDiagnosticKind::UnusedEmit {
                emit_name: "submit".into(),
            },
            DiagnosticSeverity::Warning,
            file_id,
            150,
            "Declared emit 'submit' is never called",
        ),
        // DOM related
        CrossFileDiagnostic::new(
            CrossFileDiagnosticKind::DuplicateElementId {
                id: "main-header".into(),
                locations: vec![(file_id, 10), (file_id, 250)],
            },
            DiagnosticSeverity::Error,
            file_id,
            10,
            "Duplicate id 'main-header' found",
        ),
        // SSR related
        CrossFileDiagnostic::new(
            CrossFileDiagnosticKind::BrowserApiInSsr {
                api: "window.localStorage".into(),
                context: "script setup".into(),
            },
            DiagnosticSeverity::Warning,
            file_id,
            400,
            "Browser API used in potentially SSR context",
        ),
        // Reactivity
        CrossFileDiagnostic::new(
            CrossFileDiagnosticKind::WatchMutationCanBeComputed {
                watch_source: "count".into(),
                mutated_target: "doubled".into(),
                suggested_computed: "count * 2".into(),
            },
            DiagnosticSeverity::Hint,
            file_id,
            500,
            "watch can be simplified to computed",
        ),
        CrossFileDiagnostic::new(
            CrossFileDiagnosticKind::ReactiveReferenceEscapes {
                variable_name: "state".into(),
                escaped_via: "props".into(),
                target_name: Some("childComponent".into()),
            },
            DiagnosticSeverity::Warning,
            file_id,
            600,
            "Reactive reference escapes via props",
        ),
    ];

    let mut output = String::new();
    output.push_str("=== All Diagnostic Kinds ===\n\n");

    for diag in &diagnostics {
        append!(output, "--- {:?} ---\n", diag.kind);
        append!(output, "Severity: {}\n", diag.severity.display_name());
        append!(output, "Message: {}\n", diag.message);
        output.push_str("\nMarkdown Output:\n");
        output.push_str(&diag.to_markdown());
        output.push_str("\n\n");
    }

    assert_snapshot!(output);
}

#[test]
fn test_snapshot_diagnostic_with_related_files() {
    use insta::assert_snapshot;

    let primary_file = make_file_id();
    let related_file = super::super::registry::FileId::new(1);

    let mut diag = CrossFileDiagnostic::new(
        CrossFileDiagnosticKind::ProvideInjectTypeMismatch {
            key: "userStore".into(),
            provided_type: "Ref<UserStore>".into(),
            injected_type: "UserStore".into(),
        },
        DiagnosticSeverity::Warning,
        primary_file,
        100,
        "Type mismatch: provide returns Ref<UserStore> but inject expects UserStore",
    );

    // Add related files directly
    diag.related_files
        .push((related_file, 50, "Provider defined here".into()));
    diag.related_files
        .push((primary_file, 200, "Value used here without .value".into()));

    let mut output = String::new();
    output.push_str("=== Diagnostic with Related Files ===\n\n");
    append!(output, "Primary file: {:?}\n", diag.primary_file);
    append!(
        output,
        "Offset: {} - {}\n",
        diag.primary_offset,
        diag.primary_end_offset
    );
    append!(
        output,
        "Related files count: {}\n",
        diag.related_files.len()
    );

    output.push_str("\nRelated files:\n");
    for (file_id, offset, msg) in &diag.related_files {
        append!(output, "  - {:?} at {offset}: {msg}\n", file_id);
    }

    output.push_str("\nMarkdown Output:\n");
    output.push_str(&diag.to_markdown());

    assert_snapshot!(output);
}

#[test]
fn test_snapshot_severity_levels() {
    use insta::assert_snapshot;

    let file_id = make_file_id();

    let severities = [
        DiagnosticSeverity::Error,
        DiagnosticSeverity::Warning,
        DiagnosticSeverity::Info,
        DiagnosticSeverity::Hint,
    ];

    let mut output = String::new();
    output.push_str("=== Severity Levels ===\n\n");

    for severity in severities {
        let diag = CrossFileDiagnostic::new(
            CrossFileDiagnosticKind::UnmatchedInject {
                key: "example".into(),
            },
            severity,
            file_id,
            0,
            "Example diagnostic",
        );

        append!(
            output,
            "== {} ==\n",
            severity.display_name().to_uppercase()
        );
        append!(output, "is_error: {}\n", diag.is_error());
        append!(output, "is_warning: {}\n", diag.is_warning());
        output.push_str("\nMarkdown:\n");
        output.push_str(&diag.to_markdown());
        output.push('\n');
    }

    assert_snapshot!(output);
}
