//! Tests for cross-file reactivity tracking.
#![expect(clippy::disallowed_macros, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]

use super::{CrossFileReactivityIssue, CrossFileReactivityIssueKind};
use crate::diagnostics::DiagnosticSeverity;
use crate::registry::FileId;
use insta::assert_snapshot;
use vize_carton::{CompactString, append};

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
