use super::*;

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
    let related_file = FileId::new(1);

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

        append!(output, "== {} ==\n", severity.display_name().to_uppercase());
        append!(output, "is_error: {}\n", diag.is_error());
        append!(output, "is_warning: {}\n", diag.is_warning());
        output.push_str("\nMarkdown:\n");
        output.push_str(&diag.to_markdown());
        output.push('\n');
    }

    assert_snapshot!(output);
}
