//! Tests for template analysis.

use super::super::{Analyzer, AnalyzerOptions};

/// Collect the `vif_guard` attached to each template interpolation, keyed by
/// expression text. Used to pin sibling-aware `v-if` / `v-else` narrowing.
fn interpolation_guards(template: &str) -> Vec<(std::string::String, Option<std::string::String>)> {
    use vize_armature::parse;
    use vize_carton::Bump;

    let allocator = Bump::new();
    let (root, _errors) = parse(&allocator, template);
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();

    summary
        .template_expressions
        .iter()
        .filter(|e| {
            matches!(
                e.kind,
                crate::analysis::TemplateExpressionKind::Interpolation
            )
        })
        .map(|e| {
            (
                e.content.to_string(),
                e.vif_guard.as_ref().map(|g| g.to_string()),
            )
        })
        .collect()
}

#[test]
fn flat_v_else_branch_gets_negated_guard() {
    // Regression for vuejs/language-tools#5850 / #3787-style narrowing: when the
    // parser keeps `v-if` / `v-else` as sibling elements (no `IfNode` grouping),
    // the `v-else` branch must still receive the negated guard so that
    // discriminated-union narrowing flows into it. Previously the else branch
    // had `vif_guard: None`, producing a false TS2339 in the template.
    let guards = interpolation_guards(
        r#"<div>
  <div v-if="props.data.kind === 'a'">{{ props.data.x }}</div>
  <div v-else>{{ props.data.y }}</div>
</div>"#,
    );

    let x = guards.iter().find(|(c, _)| c == "props.data.x").unwrap();
    let y = guards.iter().find(|(c, _)| c == "props.data.y").unwrap();
    assert_eq!(x.1.as_deref(), Some("(props.data.kind === 'a')"));
    assert_eq!(y.1.as_deref(), Some("!(props.data.kind === 'a')"));
}

#[test]
fn flat_v_else_if_chain_accumulates_negated_guards() {
    // A three-way flat `v-if` / `v-else-if` / `v-else` chain negates every
    // preceding condition for the later branches.
    let guards = interpolation_guards(
        r#"<div>
  <div v-if="s === 'a'">{{ a }}</div>
  <div v-else-if="s === 'b'">{{ b }}</div>
  <div v-else>{{ c }}</div>
</div>"#,
    );

    let g = |name: &str| guards.iter().find(|(c, _)| c == name).unwrap().1.clone();
    assert_eq!(g("a").as_deref(), Some("(s === 'a')"));
    assert_eq!(g("b").as_deref(), Some("!(s === 'a') && (s === 'b')"));
    assert_eq!(g("c").as_deref(), Some("!(s === 'a') && !(s === 'b')"));
}

#[test]
fn non_conditional_sibling_breaks_v_if_chain() {
    // A plain element between `v-if` and `v-else` is invalid Vue, but the
    // analyzer must not leak the earlier condition into the trailing element:
    // an element with no conditional directive resets the chain, so the second
    // `v-if` opens a fresh (un-negated) guard.
    let guards = interpolation_guards(
        r#"<div>
  <div v-if="a">{{ x }}</div>
  <div>{{ y }}</div>
  <div v-if="b">{{ z }}</div>
</div>"#,
    );

    let g = |name: &str| guards.iter().find(|(c, _)| c == name).unwrap().1.clone();
    assert_eq!(g("x").as_deref(), Some("(a)"));
    assert_eq!(g("y"), None);
    assert_eq!(g("z").as_deref(), Some("(b)"));
}

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
