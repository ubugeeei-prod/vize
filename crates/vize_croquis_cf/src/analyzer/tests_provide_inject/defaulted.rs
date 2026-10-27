use super::*;
use crate::diagnostics::{CrossFileDiagnosticKind, DiagnosticSeverity};

fn project(consumer: &str, context: &str, provider: &str) -> CrossFileAnalyzer {
    let mut analyzer =
        CrossFileAnalyzer::new(CrossFileOptions::default().with_provide_inject(true));
    assert!(matches!(context, "all" | "partial" | "none"));
    let children: &[&str] = if context == "all" {
        &["Provider"]
    } else if context == "partial" {
        &["Provider", "Child"]
    } else {
        &["Child"]
    };
    analyzer.add_file_with_analysis(
        Path::new("App.vue"),
        "",
        script_analysis("// authored roots", children),
    );
    if context != "none" {
        analyzer.add_file_with_analysis(
            Path::new("Provider.vue"),
            "",
            script_analysis(provider, &["Child"]),
        );
    }
    analyzer.add_file_with_analysis(Path::new("Child.vue"), "", script_analysis(consumer, &[]));
    analyzer.rebuild_component_edges();
    analyzer
}

#[test]
fn optional_inject_defaults_preserve_branches_without_findings() {
    let mut vectors = 0;
    for context in ["none", "partial", "all"] {
        for default in [
            "false",
            "0",
            "null",
            "''",
            "undefined",
            "() => ({ count: 0 }), true",
            "() => false, false",
        ] {
            let script =
                format!("import {{ inject }} from 'vue'; const state = inject('state', {default})");
            let mut analyzer = project(
                &script,
                context,
                "import { provide } from 'vue'; provide('state', true)",
            );
            let result = analyzer.analyze();
            let unmatched = result
                .diagnostics
                .iter()
                .filter(|diagnostic| {
                    matches!(
                        diagnostic.kind,
                        CrossFileDiagnosticKind::UnmatchedInject { .. }
                    )
                })
                .collect::<Vec<_>>();
            assert_eq!(unmatched.len(), 0, "{context}: {default}");
            assert_eq!(
                result.provide_inject_matches.len(),
                usize::from(context != "none"),
                "{context}: {default}"
            );
            let summary = result.provide_inject_tree_summary.expect("tree summary");
            assert_eq!(summary.inject_count, 1);
            assert_eq!(summary.defaulted_inject_count, 1);
            assert_eq!(
                summary.unmatched_inject_count,
                usize::from(context != "all")
            );
            assert_eq!(summary.matched_inject_count, usize::from(context == "all"));
            vectors += 1;
        }
    }
    assert_eq!(vectors, 21);
}

#[test]
fn required_injects_keep_missing_branch_errors() {
    for (context, expected_message) in [
        (
            "none",
            Some(
                "**Unmatched Inject**: `inject('state')` has no matching `provide()` in any ancestor component and returns `undefined` at runtime.",
            ),
        ),
        (
            "partial",
            Some(
                "**Conditionally Unmatched Inject**: `inject('state')` has no matching `provide()` in 1 of 2 ancestor branches. It returns `undefined` in those render contexts.",
            ),
        ),
        ("all", None),
    ] {
        let mut analyzer = project(
            "import { inject } from 'vue'; const state = inject('state')",
            context,
            "import { provide } from 'vue'; provide('state', true)",
        );
        let result = analyzer.analyze();
        let unmatched = result
            .diagnostics
            .iter()
            .filter(|diagnostic| {
                matches!(
                    diagnostic.kind,
                    CrossFileDiagnosticKind::UnmatchedInject { .. }
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(unmatched.len(), usize::from(expected_message.is_some()));
        if let Some(message) = expected_message {
            assert_eq!(unmatched[0].severity, DiagnosticSeverity::Error);
            assert_eq!(unmatched[0].message.as_str(), message);
        }
    }
}

#[test]
fn optional_inject_keeps_provider_type_mismatch() {
    let mut analyzer = project(
        "import { inject } from 'vue'; interface Expected { mode: string }; const state = inject<Expected>('state', {} as Expected)",
        "partial",
        "import { provide } from 'vue'; interface Actual { count: number }; provide('state', {} as Actual)",
    );
    let result = analyzer.analyze();
    let mismatches = result
        .diagnostics
        .iter()
        .filter_map(|diagnostic| match &diagnostic.kind {
            CrossFileDiagnosticKind::ProvideInjectTypeMismatch {
                key,
                provided_type,
                injected_type,
            } => Some((key.as_str(), provided_type.as_str(), injected_type.as_str())),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(mismatches, vec![("state", "Actual", "Expected")]);
    assert_eq!(result.provide_inject_matches.len(), 1);
    assert_eq!(result.provide_inject_matches[0].type_match, Some(false));
}

#[test]
fn optional_inject_keeps_reactive_destructuring_errors() {
    let mut analyzer = project(
        "import { inject } from 'vue'; const { count } = inject('state', { count: 0 })",
        "partial",
        "import { provide, reactive } from 'vue'; const state = reactive({ count: 1 }); provide('state', state)",
    );
    let result = analyzer.analyze();
    let losses = result
        .diagnostics
        .iter()
        .filter_map(|diagnostic| match &diagnostic.kind {
            CrossFileDiagnosticKind::DestructuringBreaksReactivity {
                source_name,
                destructured_keys,
                suggestion,
            } => Some((
                source_name.as_str(),
                destructured_keys
                    .iter()
                    .map(|key| key.as_str())
                    .collect::<Vec<_>>(),
                suggestion.as_str(),
                diagnostic.severity,
            )),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        losses,
        vec![(
            "inject('state')",
            vec!["count"],
            "toRefs",
            DiagnosticSeverity::Error
        )]
    );
}
