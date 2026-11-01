use super::{AttributeHyphenation, HyphenationStyle};
use crate::linter::Linter;
use crate::rule::RuleRegistry;

fn create_linter() -> Linter {
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(AttributeHyphenation::default()));
    Linter::with_registry(registry)
}

fn warning_count(template: &str) -> usize {
    create_linter()
        .lint_template(template, "test.vue")
        .warning_count
}

fn never_warning_count(template: &str) -> usize {
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(AttributeHyphenation::new(HyphenationStyle::Never)));
    Linter::with_registry(registry)
        .lint_template(template, "test.vue")
        .warning_count
}

fn fixed(linter: &Linter, source: &str) -> String {
    let result = linter.lint_template(source, "test.vue");
    let mut fixed = source.to_owned();
    let mut edits: Vec<_> = result
        .diagnostics
        .iter()
        .flat_map(|diagnostic| diagnostic.fix.as_ref().unwrap().edits.iter())
        .collect();
    edits.sort_by_key(|edit| std::cmp::Reverse(edit.start));
    for edit in edits {
        fixed.replace_range(edit.start as usize..edit.end as usize, &edit.new_text);
    }
    fixed
}

#[test]
fn test_valid_hyphenated() {
    assert_eq!(warning_count(r#"<MyComponent my-prop="value" />"#), 0);
}

#[test]
fn test_invalid_camel_case() {
    assert_eq!(warning_count(r#"<MyComponent myProp="value" />"#), 1);
}

#[test]
fn test_invalid_uppercase_initial_and_lowercase_custom_component() {
    assert_eq!(
        warning_count(r#"<div><MyComponent DOMId="value" /><draggable itemKey="id" /></div>"#,),
        2,
    );
}

#[test]
fn test_invalid_bound_camel_case_argument() {
    assert_eq!(warning_count(r#"<MyComponent :activeKey="value" />"#), 1);
}

#[test]
fn test_invalid_model_camel_case_argument() {
    assert_eq!(
        warning_count(r#"<MyComponent v-model:activeKey.trim="value" />"#,),
        1,
    );
}

#[test]
fn test_invalid_on_prefixed_prop() {
    assert_eq!(warning_count(r#"<MyComponent :onUndo="undo" />"#), 1);
}

#[test]
fn test_valid_event_argument() {
    assert_eq!(warning_count(r#"<MyComponent @onUndo="undo" />"#), 0);
}

#[test]
fn test_valid_dynamic_argument() {
    assert_eq!(warning_count(r#"<MyComponent :[myProp]="value" />"#), 0);
}

#[test]
fn test_valid_dynamic_model_argument() {
    assert_eq!(
        warning_count(r#"<MyComponent v-model:[activeKey]="value" />"#),
        0
    );
}

#[test]
fn test_valid_svg_weird_case_attributes() {
    assert_eq!(
        warning_count(
            r#"<MyIcon viewBox="0 0 16 16" :preserveAspectRatio="ratio" customCamel="x" />"#,
        ),
        1,
    );
}

#[test]
fn test_invalid_customized_builtin() {
    assert_eq!(
        warning_count(r#"<div is="vue:MyRow" :rowData="row" /><div :is="MyRow" rowData />"#),
        2,
    );
}

#[test]
fn test_valid_html_element() {
    assert_eq!(warning_count(r#"<div onClick="handler"></div>"#), 0);
}

#[test]
fn test_valid_data_attribute() {
    assert_eq!(warning_count(r#"<MyComponent data-testId="123" />"#), 0);
}

#[test]
fn test_never_invalid_hyphenated_attribute() {
    assert_eq!(never_warning_count(r#"<MyComponent my-prop="value" />"#), 1);
    assert_eq!(
        never_warning_count(r#"<MyComponent :active-key="value" />"#),
        1
    );
}

#[test]
fn test_never_valid_camel_case_attribute_and_native_elements() {
    assert_eq!(
        never_warning_count(r#"<MyComponent myProp="value" /><div data-test="value" />"#),
        0
    );
}

#[test]
fn test_camel_case_aria_and_data_props_are_allowed() {
    // Hyphenated `aria-*` / `data-*` names bypass prop type checking in
    // vue-tsc, so camelCase is the only spelling that type-checks a required
    // `ariaLabel` prop.
    assert_eq!(
        warning_count(
            r#"<div><IconButton ariaLabel="Close" /><MyComponent :ariaLabelledby="id" :dataSource="rows" /></div>"#,
        ),
        0,
    );
    assert_eq!(
        warning_count(r#"<IconButton aria-label="Close" data-testid="x" />"#),
        0
    );
}

#[test]
fn test_names_that_only_start_like_aria_or_data_still_need_hyphens() {
    assert_eq!(
        warning_count(r#"<MyComponent dataset="x" ariadne="y" :databaseName="n" />"#),
        1,
    );
}

#[test]
fn fixes_authored_static_and_bound_attribute_names() {
    let linter = create_linter();
    let source = r#"<UserBadge displayName="a" :activeKey="key" v-model:pageSize="size" />"#;
    let fixed = fixed(&linter, source);
    assert_eq!(
        fixed,
        r#"<UserBadge display-name="a" :active-key="key" v-model:page-size="size" />"#
    );
    assert_eq!(linter.lint_template(&fixed, "test.vue").warning_count, 0);
    let report = linter.lint_template(source, "test.vue");
    let spans: Vec<_> = report
        .diagnostics
        .iter()
        .filter_map(|d| source.get(d.start as usize..d.end as usize))
        .collect();
    assert_eq!(
        spans.join(" "),
        r#"displayName="a" :activeKey="key" v-model:pageSize="size""#
    );
}

#[test]
fn never_fixes_authored_attribute_names_to_camel_case() {
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(AttributeHyphenation::new(HyphenationStyle::Never)));
    let linter = Linter::with_registry(registry);
    let source = r#"<UserBadge display-name="a" :active-key="key" />"#;
    let fixed = fixed(&linter, source);
    assert_eq!(fixed, r#"<UserBadge displayName="a" :activeKey="key" />"#);
    assert_eq!(linter.lint_template(&fixed, "test.vue").warning_count, 0);
}
