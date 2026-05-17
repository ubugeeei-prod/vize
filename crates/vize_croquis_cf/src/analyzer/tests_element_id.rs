use super::{CrossFileAnalyzer, CrossFileOptions};
use crate::diagnostics::{CrossFileDiagnosticKind, DiagnosticSeverity};
use std::path::Path;
use vize_armature::parse;
use vize_carton::Bump;
use vize_croquis::{Analyzer, AnalyzerOptions, Croquis};

fn analyze_template(template: &str) -> Croquis {
    let allocator = Bump::new();
    let (root, errors) = parse(&allocator, template);
    assert!(errors.is_empty(), "template should parse cleanly");

    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_template(&root);
    analyzer.finish()
}

#[test]
fn duplicate_static_ids_are_reported_across_components() {
    let mut analyzer = CrossFileAnalyzer::new(CrossFileOptions::default().with_unique_ids(true));

    analyzer.add_file_with_analysis(
        Path::new("FormA.vue"),
        "",
        analyze_template(r#"<label for="email">Email</label><input id="email" />"#),
    );
    analyzer.add_file_with_analysis(
        Path::new("FormB.vue"),
        "",
        analyze_template(r#"<input id="email" />"#),
    );

    let result = analyzer.analyze();
    let diagnostic = result
        .diagnostics
        .iter()
        .find(|diagnostic| {
            matches!(
                &diagnostic.kind,
                CrossFileDiagnosticKind::DuplicateElementId { id, .. } if id == "email"
            )
        })
        .expect("duplicate element id should be reported");

    assert_eq!(result.unique_id_issues.len(), 1);
    assert_eq!(diagnostic.severity, DiagnosticSeverity::Warning);
    assert_eq!(diagnostic.related_files.len(), 1);
}

#[test]
fn static_ids_in_v_for_are_errors() {
    let mut analyzer = CrossFileAnalyzer::new(CrossFileOptions::default().with_unique_ids(true));
    analyzer.add_file_with_analysis(
        Path::new("List.vue"),
        "",
        analyze_template(r#"<div v-for="item in items" id="row">{{ item.name }}</div>"#),
    );

    let result = analyzer.analyze();
    let diagnostic = result
        .diagnostics
        .iter()
        .find(|diagnostic| {
            matches!(
                &diagnostic.kind,
                CrossFileDiagnosticKind::NonUniqueIdInLoop { id_expression } if id_expression == "row"
            )
        })
        .expect("static id inside v-for should be reported");

    assert_eq!(diagnostic.severity, DiagnosticSeverity::Error);
}

#[test]
fn dynamic_ids_in_v_for_must_look_unique() {
    let mut analyzer = CrossFileAnalyzer::new(CrossFileOptions::default().with_unique_ids(true));
    analyzer.add_file_with_analysis(
        Path::new("List.vue"),
        "",
        analyze_template(
            r#"<div>
  <div v-for="item in items" :id="item.name">{{ item.name }}</div>
  <div v-for="item in items" :id="item.id">{{ item.name }}</div>
</div>"#,
        ),
    );

    let result = analyzer.analyze();
    assert!(result.diagnostics.iter().any(|diagnostic| {
        matches!(
            &diagnostic.kind,
            CrossFileDiagnosticKind::NonUniqueIdInLoop { id_expression } if id_expression == "item.name"
        )
    }));
    assert!(result.diagnostics.iter().all(|diagnostic| {
        !matches!(
            &diagnostic.kind,
            CrossFileDiagnosticKind::NonUniqueIdInLoop { id_expression } if id_expression == "item.id"
        )
    }));
}

#[test]
fn descendant_static_ids_inside_v_for_are_errors() {
    let mut analyzer = CrossFileAnalyzer::new(CrossFileOptions::default().with_unique_ids(true));
    analyzer.add_file_with_analysis(
        Path::new("NestedList.vue"),
        "",
        analyze_template(
            r#"<ul>
  <li v-for="item in items">
    <span id="row-label">{{ item.name }}</span>
  </li>
</ul>"#,
        ),
    );

    let result = analyzer.analyze();
    assert!(result.diagnostics.iter().any(|diagnostic| {
        diagnostic.severity == DiagnosticSeverity::Error
            && matches!(
                &diagnostic.kind,
                CrossFileDiagnosticKind::NonUniqueIdInLoop { id_expression } if id_expression == "row-label"
            )
    }));
}

#[test]
fn bound_static_string_ids_are_deduplicated_with_plain_static_ids() {
    let mut analyzer = CrossFileAnalyzer::new(CrossFileOptions::default().with_unique_ids(true));
    analyzer.add_file_with_analysis(
        Path::new("Plain.vue"),
        "",
        analyze_template(r#"<input id="email" />"#),
    );
    analyzer.add_file_with_analysis(
        Path::new("Bound.vue"),
        "",
        analyze_template(r#"<input :id="'email'" />"#),
    );

    let result = analyzer.analyze();
    let diagnostic = result
        .diagnostics
        .iter()
        .find(|diagnostic| {
            matches!(
                &diagnostic.kind,
                CrossFileDiagnosticKind::DuplicateElementId { id, .. } if id == "email"
            )
        })
        .expect("bound static string id should be grouped with plain id");

    assert_eq!(diagnostic.severity, DiagnosticSeverity::Warning);
    assert_eq!(result.unique_id_issues.len(), 1);
}

#[test]
fn id_references_do_not_count_as_duplicate_definitions() {
    let mut analyzer = CrossFileAnalyzer::new(CrossFileOptions::default().with_unique_ids(true));
    analyzer.add_file_with_analysis(
        Path::new("LabelA.vue"),
        "",
        analyze_template(r#"<label for="email">Email</label>"#),
    );
    analyzer.add_file_with_analysis(
        Path::new("LabelB.vue"),
        "",
        analyze_template(r#"<label for="email">Email again</label>"#),
    );

    let result = analyzer.analyze();
    assert!(result.unique_id_issues.is_empty());
    assert!(result.diagnostics.iter().all(|diagnostic| {
        !matches!(
            &diagnostic.kind,
            CrossFileDiagnosticKind::DuplicateElementId { id, .. } if id == "email"
        )
    }));
}
