use super::{AttributeHyphenation, HyphenationStyle};
use crate::{Linter, rule::RuleRegistry};
use vize_l0::ToCompactString;

fn configured(tags: &[&str], style: HyphenationStyle) -> Linter {
    let mut registry = RuleRegistry::new();
    let mut rule = AttributeHyphenation::new(style);
    rule.ignore_tags = tags.iter().map(|tag| tag.to_compact_string()).collect();
    registry.register(Box::new(rule));
    Linter::with_registry(registry)
}

#[test]
fn literal_tags_match_the_complete_authored_case_sensitive_name() {
    let source = r#"<MyWidget fooBar="value" :activeKey="key" v-model:pageSize="size" />"#;
    for tags in [
        &[][..],
        &["Widget"][..],
        &["mywidget"][..],
        &["", " ", ":exit"][..],
    ] {
        assert_eq!(
            configured(tags, HyphenationStyle::Always)
                .lint_template(source, "test.vue")
                .warning_count,
            3
        );
    }
    let report =
        configured(&["MyWidget"], HyphenationStyle::Always).lint_template(source, "test.vue");
    assert!(report.diagnostics.is_empty());
    assert_eq!(report.warning_count, 0);
    let source = r#"<MyWidget foo-bar="value" :active-key="key" v-model:page-size="size" />"#;
    assert_eq!(
        configured(&["MyWidget"], HyphenationStyle::Never)
            .lint_template(source, "test.vue")
            .warning_count,
        0
    );
}

#[test]
fn ignoring_a_parent_does_not_disable_children_siblings_or_literal_dot_neighbors() {
    let source = r#"<Ignored fooBar="x"><MyWidget :activeKey="key" /></Ignored><MyWidget fooBar="x" /><My.Widget fooBar="x" /><MyXWidget fooBar="x" />"#;
    let report = configured(&["Ignored", "My.Widget"], HyphenationStyle::Always)
        .lint_template(source, "test.vue");
    assert_eq!(report.warning_count, 3);
    let spans: Vec<_> = report
        .diagnostics
        .iter()
        .map(|item| &source[item.start as usize..item.end as usize])
        .collect();
    assert_eq!(
        spans,
        [r#":activeKey="key""#, r#"fooBar="x""#, r#"fooBar="x""#]
    );
}

#[test]
fn customized_builtins_match_authored_tag_without_changing_default_classification() {
    let source = r#"<div is="vue:MyRow" :rowData="row" /><div rowData="native" />"#;
    assert_eq!(
        configured(&[], HyphenationStyle::Always)
            .lint_template(source, "test.vue")
            .warning_count,
        1
    );
    assert_eq!(
        configured(&["MyRow"], HyphenationStyle::Always)
            .lint_template(source, "test.vue")
            .warning_count,
        1
    );
    assert_eq!(
        configured(&["div"], HyphenationStyle::Always)
            .lint_template(source, "test.vue")
            .warning_count,
        0
    );
}

#[test]
fn literal_tag_configuration_does_not_register_an_unselected_rule() {
    let report = Linter::with_registry(RuleRegistry::new())
        .with_attribute_hyphenation_policy(
            HyphenationStyle::Always,
            vec![],
            vec!["Other".to_compact_string()],
        )
        .lint_template(r#"<MyWidget fooBar="x" />"#, "test.vue");
    assert!(report.diagnostics.is_empty());
}
