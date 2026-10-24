use super::*;

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

    let diag_race = CrossFileDiagnostic::new(
        CrossFileDiagnosticKind::InjectedAsyncMutationRace {
            key: "store".into(),
            target_name: "store".into(),
            async_context: "watch await".into(),
            writer_count: 2,
        },
        DiagnosticSeverity::Error,
        make_file_id(),
        0,
        "test",
    );
    assert_eq!(
        diag_race.code(),
        "vize:croquis/cf/injected-async-mutation-race"
    );
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
