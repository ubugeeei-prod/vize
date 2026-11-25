use super::super::NoEmptyPattern;
use super::reported;
use crate::{Linter, Severity};

#[test]
fn empty_pattern_reads_vue2_slot_scope_attributes() {
    assert_eq!(
        reported(
            Box::new(NoEmptyPattern),
            r#"<el-table-column><template slot-scope="{}"><i /></template><template scope="{ row }"><b /></template></el-table-column>"#,
        ),
        ["{}"]
    );
}

#[test]
fn empty_pattern_runs_with_full_registry_and_markup_facade_rules() {
    let source = r#"<template><el-table-column><template slot-scope="{}"><i /></template><template scope="[]"><b /></template><template slot-scope="{ row }"><img /></template></el-table-column></template>"#;
    let expected = ["{}", "[]"].map(|pattern| {
        let start = source.find(pattern).unwrap() as u32;
        (start, start + pattern.len() as u32)
    });

    // The full preset uses the cached legacy callback plan. Selecting a legacy
    // rule together with a markup facade rule builds the filtered plan instead.
    // A single-rule registry visits every callback and misses this regression.
    for linter in [
        Linter::with_all(),
        Linter::with_all().with_enabled_rules(Some(vec![
            "vue/no-empty-pattern".into(),
            "a11y/img-alt".into(),
        ])),
    ] {
        let result = linter.lint_sfc(source, "DragTable.vue");
        assert!(
            result
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.rule_name == "a11y/img-alt")
        );
        let diagnostics: Vec<_> = result
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.rule_name == "vue/no-empty-pattern")
            .collect();
        assert_eq!(diagnostics.len(), expected.len());
        for (diagnostic, (start, end)) in diagnostics.into_iter().zip(expected) {
            assert_eq!((diagnostic.start, diagnostic.end), (start, end));
            assert_eq!(diagnostic.severity, Severity::Warning);
            assert_eq!(
                diagnostic.message.as_str(),
                "Unexpected empty destructuring pattern"
            );
            assert!(diagnostic.fix.is_none());
        }
    }
}

#[test]
fn empty_pattern_reports_pinned_vue_element_admin_drag_table() {
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/_fixtures/differential/linter/no-empty-pattern-slot-scope-12336/original-drag-table.vue.txt"
    ));
    let result = Linter::with_all().lint_sfc(source, "src/views/table/drag-table.vue");
    let diagnostics: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.rule_name == "vue/no-empty-pattern")
        .collect();
    assert_eq!(diagnostics.len(), 1);
    let diagnostic = diagnostics[0];
    let prefix = &source[..diagnostic.start as usize];
    assert_eq!(prefix.bytes().filter(|byte| *byte == b'\n').count() + 1, 50);
    assert_eq!(prefix.rsplit('\n').next().unwrap().chars().count() + 1, 31);
    assert_eq!(
        &source[diagnostic.start as usize..diagnostic.end as usize],
        "{}"
    );
    assert_eq!(diagnostic.severity, Severity::Warning);
    assert_eq!(
        diagnostic.message.as_str(),
        "Unexpected empty destructuring pattern"
    );
}
