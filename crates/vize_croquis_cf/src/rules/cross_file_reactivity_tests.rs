//! Tests for cross-file reactivity tracking.
#![expect(clippy::disallowed_macros, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]

use super::types::{ReactiveValueId, ReactivityFlow};
use super::{CrossFileReactivityAnalyzer, CrossFileReactivityIssue, CrossFileReactivityIssueKind};
use crate::diagnostics::{CrossFileDiagnosticKind, DiagnosticSeverity, DiagnosticSource};
use crate::graph::DependencyGraph;
use crate::registry::{FileId, ModuleRegistry};
use insta::assert_snapshot;
use vize_carton::{CompactString, append};

#[test]
fn documented_reactive_cycle_requires_the_same_tracked_reference_identities() {
    let registry = ModuleRegistry::new();
    let graph = DependencyGraph::new();
    let a = ReactiveValueId {
        file_id: FileId::new(1),
        name: CompactString::new("A"),
        offset: 10,
    };
    let b = ReactiveValueId {
        file_id: FileId::new(2),
        name: CompactString::new("B"),
        offset: 20,
    };
    let distinct_a = ReactiveValueId {
        offset: 11,
        ..a.clone()
    };
    let forward = ReactivityFlow {
        source: a.clone(),
        target: b.clone(),
    };
    let backward = ReactivityFlow {
        source: b.clone(),
        target: a.clone(),
    };
    let unrelated = ReactivityFlow {
        source: b,
        target: distinct_a,
    };
    for (flows, expected_cycle) in [
        (vec![forward.clone(), backward], true),
        (vec![forward.clone()], false),
        (vec![forward, unrelated], false),
    ] {
        let mut analyzer = CrossFileReactivityAnalyzer::new(&registry, &graph);
        analyzer.flows = flows;
        let (issues, diagnostics) = analyzer.analyze();
        assert_eq!(issues.len(), usize::from(expected_cycle));
        assert_eq!(diagnostics.len(), usize::from(expected_cycle));
        if expected_cycle {
            let cycle = vec![CompactString::new("A"), CompactString::new("B")];
            let issue = issues.first().expect("cycle issue");
            assert_eq!(
                issue.kind,
                CrossFileReactivityIssueKind::CircularReactiveDependency {
                    cycle: cycle.clone(),
                }
            );
            assert_eq!(issue.file_id, a.file_id);
            assert_eq!(issue.offset, a.offset);
            assert_eq!(issue.related_file, Some(FileId::new(2)));
            assert_eq!(issue.severity, DiagnosticSeverity::Warning);
            let diagnostic = diagnostics.first().expect("cycle diagnostic");
            assert_eq!(
                diagnostic.kind,
                CrossFileDiagnosticKind::CircularReactiveDependency { cycle }
            );
            assert_eq!(
                diagnostic.code(),
                "vize:croquis/cf/circular-reactive-dependency"
            );
            assert_eq!(diagnostic.severity, DiagnosticSeverity::Warning);
            assert_eq!(diagnostic.primary_file, a.file_id);
            assert_eq!(diagnostic.primary_source, DiagnosticSource::Unspecified);
            assert_eq!(diagnostic.primary_offset, a.offset);
            assert_eq!(diagnostic.primary_end_offset, a.offset);
            assert!(diagnostic.related_files.is_empty());
            assert_eq!(
                diagnostic.message,
                "Circular reactive dependency: A → B → ..."
            );
            assert_eq!(
                diagnostic.suggestion.as_deref(),
                Some("Break the cycle by using computed() or reorganizing data flow")
            );
        }
    }
}

#[test]
fn test_snapshot_cross_file_reactivity_issue() {
    let mut output = String::new();
    output.push_str("=== Cross-File Reactivity Issues ===\n\n");

    let issues = [
        CrossFileReactivityIssue {
            file_id: FileId::new(1),
            kind: CrossFileReactivityIssueKind::ComposableReturnDestructured {
                composable_name: CompactString::new("useCounter"),
                destructured_props: vec![
                    CompactString::new("count"),
                    CompactString::new("increment"),
                ],
            },
            offset: 100,
            related_file: Some(FileId::new(2)),
            severity: DiagnosticSeverity::Warning,
        },
        CrossFileReactivityIssue {
            file_id: FileId::new(3),
            kind: CrossFileReactivityIssueKind::StoreDestructured {
                store_name: CompactString::new("useUserStore"),
                destructured_props: vec![
                    CompactString::new("user"),
                    CompactString::new("isLoggedIn"),
                ],
            },
            offset: 150,
            related_file: Some(FileId::new(4)),
            severity: DiagnosticSeverity::Warning,
        },
        CrossFileReactivityIssue {
            file_id: FileId::new(5),
            kind: CrossFileReactivityIssueKind::PropsDestructured {
                destructured_props: vec![CompactString::new("count"), CompactString::new("name")],
            },
            offset: 30,
            related_file: None,
            severity: DiagnosticSeverity::Warning,
        },
        CrossFileReactivityIssue {
            file_id: FileId::new(6),
            kind: CrossFileReactivityIssueKind::InjectValueDestructured {
                key: CompactString::new("theme"),
                destructured_props: vec![CompactString::new("isDark")],
            },
            offset: 80,
            related_file: Some(FileId::new(1)),
            severity: DiagnosticSeverity::Warning,
        },
        CrossFileReactivityIssue {
            file_id: FileId::new(7),
            kind: CrossFileReactivityIssueKind::NonReactiveProvide {
                key: CompactString::new("config"),
            },
            offset: 50,
            related_file: None,
            severity: DiagnosticSeverity::Error,
        },
    ];

    for (i, issue) in issues.iter().enumerate() {
        append!(output, "Issue {} - {:?}\n", i + 1, issue.kind);
        append!(
            output,
            "  File: {:?}, offset={}\n",
            issue.file_id,
            issue.offset
        );
        if let Some(ref related) = issue.related_file {
            append!(output, "  Related file: {related:?}\n");
        }
        append!(output, "  Severity: {:?}\n\n", issue.severity);
    }

    assert_snapshot!(output);
}
