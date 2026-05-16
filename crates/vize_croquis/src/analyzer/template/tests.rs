//! Tests for template analysis.

use super::super::{Analyzer, AnalyzerOptions};

#[test]
fn test_vif_guard_in_template() {
    use vize_armature::parse;
    use vize_carton::Bump;

    let allocator = Bump::new();
    let template = r#"<div>
            <p v-if="todo.description">{{ unwrapDescription(todo.description) }}</p>
            <span>{{ todo.title }}</span>
        </div>"#;

    let (root, errors) = parse(&allocator, template);
    assert!(errors.is_empty(), "Template should parse without errors");

    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();

    // Find the interpolation expressions
    let expressions: Vec<_> = summary
        .template_expressions
        .iter()
        .filter(|e| {
            matches!(
                e.kind,
                crate::analysis::TemplateExpressionKind::Interpolation
            )
        })
        .collect();

    insta::assert_debug_snapshot!(expressions);
}
