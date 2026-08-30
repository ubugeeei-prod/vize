use crate::diagnostic::Severity;
use crate::linter::{LintResult, Linter};
use crate::rule::{Rule, RuleRegistry};
use crate::rules::a11y::UseList;
use vize_atelier_jsx::JsxLang;

fn linter_with(rule: Box<dyn Rule>) -> Linter {
    let mut registry = RuleRegistry::new();
    registry.register(rule);
    Linter::with_registry(registry)
}

fn diagnostic_rules(result: &LintResult) -> Vec<&str> {
    result
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.rule_name.as_ref())
        .collect()
}

fn diagnostic_slices<'a>(source: &'a str, result: &LintResult) -> Vec<&'a str> {
    result
        .diagnostics
        .iter()
        .map(|diagnostic| &source[diagnostic.start as usize..diagnostic.end as usize])
        .collect()
}

#[test]
fn use_list_fires_on_jsx_and_tsx_lowered_markup() {
    let linter = linter_with(Box::new(UseList));
    let source = r#"const A = () => <p>- Item one</p>;"#;
    let result = linter.lint_jsx(source, "test.jsx", JsxLang::Jsx);

    assert_eq!(
        diagnostic_rules(&result),
        vec!["a11y/use-list"],
        "JSX bullet text must flag through the lowered markup pass: {:?}",
        result.diagnostics
    );
    assert_eq!(result.error_count, 0);

    let diag = &result.diagnostics[0];
    let text_start = source.find("- Item one").unwrap() as u32;
    assert_eq!(diag.start, text_start);
    assert_eq!(
        &source[diag.start as usize..diag.end as usize],
        "- Item one",
        "range must cover the authored JSX text node"
    );
    assert_eq!(diag.severity, Severity::Warning);
    assert_eq!(
        diag.help.as_deref(),
        Some(
            "Screen readers cannot identify bullet characters as list items. Use semantic <ul>/<ol> with <li> elements instead."
        )
    );
    assert_eq!(diag.fix.as_ref().map(|_| "some"), None);

    let tsx = linter.lint_jsx(
        r#"const A = <T,>(props: { x: T }) => <section><p>• Item</p></section>;"#,
        "test.tsx",
        JsxLang::Tsx,
    );
    assert_eq!(
        diagnostic_rules(&tsx),
        vec!["a11y/use-list"],
        "TSX bullet text must also flag through the lowered markup pass"
    );
}

#[test]
fn use_list_preserves_legacy_jsx_boundaries() {
    let linter = linter_with(Box::new(UseList));
    for source in [
        r#"const A = () => <p>Normal text</p>;"#,
        r#"const A = () => <p>-word</p>;"#,
        r#"const A = () => <ul><li>- Item</li></ul>;"#,
        r#"const A = () => <ol><li>* Item</li></ol>;"#,
        r#"const A = () => <li>+ Item</li>;"#,
        r#"const A = () => <pre>- markdown content</pre>;"#,
        r#"const A = () => <code>- flag</code>;"#,
        r#"const A = () => <p>{lead} - Item</p>;"#,
        r#"const A = () => <p><span />- Item</p>;"#,
        r#"const A = () => <Panel>- Item</Panel>;"#,
        r#"const A = () => <Icons.Panel>- Item</Icons.Panel>;"#,
        r#"const A = () => <ul><li><span>- Item</span></li></ul>;"#,
        r#"const A = () => <ul>{items.map((item) => <span>- Item</span>)}</ul>;"#,
        r#"const A = () => <pre><span>- literal</span></pre>;"#,
        r#"const A = () => <code><span>- flag</span></code>;"#,
    ] {
        let result = linter.lint_jsx(source, "test.jsx", JsxLang::Jsx);
        assert_eq!(
            result.warning_count, 0,
            "must stay clean for {source}: {:?}",
            result.diagnostics
        );
        assert_eq!(result.error_count, 0, "must not error for {source}");
    }

    for source in [
        r#"const A = () => <p>- Item</p>;"#,
        r#"const A = () => <span>* Item</span>;"#,
        r#"const A = () => <div>+ Item</div>;"#,
        r#"const A = () => <p>   - Item</p>;"#,
        r#"const A = () => <section>- Item</section>;"#,
        r#"const A = () => <panel>- Item</panel>;"#,
        r#"const A = () => <my-panel>- Item</my-panel>;"#,
        r#"const A = () => <List.ul><p>- Item</p></List.ul>;"#,
        r#"const A = () => <p>{'- Item'}</p>;"#,
        r#"const A = () => <p>{/* comment */}- Item</p>;"#,
        r#"const A = () => <p><>- Item</></p>;"#,
        r#"const A = () => <p>{cond && <span>- Item</span>}</p>;"#,
        r#"const A = () => <p>{items.map((item) => <span>* Item</span>)}</p>;"#,
    ] {
        let result = linter.lint_jsx(source, "test.jsx", JsxLang::Jsx);
        assert_eq!(
            result.warning_count, 1,
            "must keep warning for {source}: {:?}",
            result.diagnostics
        );
        assert_eq!(result.error_count, 0, "must not error for {source}");
    }
}

#[test]
fn use_list_reports_multiple_text_nodes_in_source_order() {
    let linter = linter_with(Box::new(UseList));
    let source = r#"const A = () => <div><p>- First</p><span>* Second</span></div>;"#;
    let result = linter.lint_jsx(source, "test.jsx", JsxLang::Jsx);

    assert_eq!(
        diagnostic_rules(&result),
        vec!["a11y/use-list", "a11y/use-list"]
    );
    assert_eq!(result.warning_count, 2);
    assert_eq!(result.error_count, 0);
    assert_eq!(
        diagnostic_slices(source, &result),
        vec!["- First", "* Second"]
    );
}

#[test]
fn migrated_use_list_reports_once_not_per_backend() {
    let linter = linter_with(Box::new(UseList));
    let result = linter.lint_jsx(
        r#"const A = () => <p>- Item one</p>;"#,
        "test.jsx",
        JsxLang::Jsx,
    );

    assert_eq!(
        result.diagnostics.len(),
        1,
        "a migrated use-list rule must report once: {:?}",
        result.diagnostics
    );
    assert_eq!(result.warning_count, 1);
    assert_eq!(result.error_count, 0);
}

#[test]
fn use_list_sfc_offsets_template_diagnostics() {
    let source = r#"<script setup lang="ts">
const bullet = "- Item";
</script>

<template>
  <p>- Item one</p>
</template>
"#;
    let linter = linter_with(Box::new(UseList));
    let result = linter.lint_sfc(source, "test.vue");

    assert_eq!(
        diagnostic_rules(&result),
        vec!["a11y/use-list"],
        "SFC template bullet text must report once: {:?}",
        result.diagnostics
    );

    let diag = &result.diagnostics[0];
    let text_start = source.find("- Item one").unwrap() as u32;
    assert_eq!(
        diag.start, text_start,
        "SFC diagnostic must be offset into file coordinates"
    );
    assert_eq!(
        &source[diag.start as usize..diag.end as usize],
        "- Item one",
        "range must cover the bullet text in the full SFC"
    );
}

#[test]
fn use_list_sfc_does_not_lint_script_tsx() {
    let source = r#"<script setup lang="tsx">
const A = () => <p>- Item one</p>;
</script>
"#;
    let linter = linter_with(Box::new(UseList));
    let result = linter.lint_sfc(source, "test.vue");

    assert_eq!(
        result.warning_count, 0,
        "SFC use-list must not inspect JSX inside script blocks: {:?}",
        result.diagnostics
    );
    assert_eq!(result.error_count, 0);
}
