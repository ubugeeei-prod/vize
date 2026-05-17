use super::*;

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
