use crate::diagnostic::Severity;
use crate::linter::{LintResult, Linter};
use crate::rule::{Rule, RuleRegistry};
use crate::rules::vue::NoBareStringsInTemplate;
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
fn no_bare_strings_fires_on_jsx_and_tsx_lowered_markup() {
    let linter = linter_with(Box::new(NoBareStringsInTemplate));
    let source = r#"const A = () => <div>Hello</div>;"#;
    let result = linter.lint_jsx(source, "test.jsx", JsxLang::Jsx);

    assert_eq!(
        diagnostic_rules(&result),
        vec!["vue/no-bare-strings-in-template"],
        "JSX bare text must flag through the lowered markup pass: {:?}",
        result.diagnostics
    );
    assert_eq!(result.error_count, 0);

    let diag = &result.diagnostics[0];
    let text_start = source.find("Hello").unwrap() as u32;
    assert_eq!(diag.start, text_start);
    assert_eq!(&source[diag.start as usize..diag.end as usize], "Hello");
    assert_eq!(diag.severity, Severity::Warning);
    assert_eq!(
        diag.help.as_deref(),
        Some(
            "Move the text into a translation function, e.g. {{ $t('key') }} for content or :title=\"$t('key')\" for an attribute."
        )
    );
    assert_eq!(diag.fix.as_ref().map(|_| "some"), None);

    let tsx = linter.lint_jsx(
        r#"const A = (): JSX.Element => <div>こんにちは</div>;"#,
        "test.tsx",
        JsxLang::Tsx,
    );
    assert_eq!(
        diagnostic_rules(&tsx),
        vec!["vue/no-bare-strings-in-template"],
        "TSX bare text must also flag through the lowered markup pass"
    );
}

#[test]
fn no_bare_strings_preserves_legacy_jsx_boundaries() {
    let linter = linter_with(Box::new(NoBareStringsInTemplate));
    for source in [
        r#"const A = () => <div>{label}</div>;"#,
        r#"const A = () => <div>{t('hello')}</div>;"#,
        r#"const A = () => <div>123</div>;"#,
        r#"const A = () => <div>-</div>;"#,
        r#"const A = () => <div>{'-'}</div>;"#,
        r#"const A = () => <img alt={caption} />;"#,
        r#"const A = () => <img title={title} />;"#,
        r#"const A = () => <img html:alt="a cat" />;"#,
        r#"const A = () => <div class="container"></div>;"#,
        r#"const A = () => <script>const label = "Hello";</script>;"#,
        r#"const A = () => <style>{`.x::before { content: "Hello"; }`}</style>;"#,
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
        r#"const A = () => <div>Hello</div>;"#,
        r#"const A = () => <div>こんにちは</div>;"#,
        r#"const A = () => <img alt="a cat" />;"#,
        r#"const A = () => <input placeholder="Search" />;"#,
        r#"const A = () => <div aria-label="Menu" />;"#,
        r#"const A = () => <button TITLE="Close" />;"#,
        r#"const A = () => <div>{'Hello'}</div>;"#,
        r#"const A = () => <div>{/* comment */}Hello</div>;"#,
        r#"const A = () => <MyPanel>Hello</MyPanel>;"#,
        r#"const A = () => <div>{cond && <span>Hello</span>}</div>;"#,
        r#"const A = () => <ul>{items.map((item) => <li>{'Hello'}</li>)}</ul>;"#,
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
fn no_bare_strings_reports_multiple_jsx_targets_in_source_order() {
    let linter = linter_with(Box::new(NoBareStringsInTemplate));
    let source = r#"const A = () => <button title="Close"><span>Save</span></button>;"#;
    let result = linter.lint_jsx(source, "test.jsx", JsxLang::Jsx);

    assert_eq!(
        diagnostic_rules(&result),
        vec![
            "vue/no-bare-strings-in-template",
            "vue/no-bare-strings-in-template"
        ]
    );
    assert_eq!(result.warning_count, 2);
    assert_eq!(result.error_count, 0);
    assert_eq!(
        diagnostic_slices(source, &result),
        vec![r#"title="Close""#, "Save"]
    );
}

#[test]
fn migrated_no_bare_strings_reports_once_not_per_backend() {
    let linter = linter_with(Box::new(NoBareStringsInTemplate));
    let result = linter.lint_jsx(
        r#"const A = () => <div>Hello</div>;"#,
        "test.jsx",
        JsxLang::Jsx,
    );

    assert_eq!(
        result.diagnostics.len(),
        1,
        "a migrated no-bare-strings rule must report once: {:?}",
        result.diagnostics
    );
    assert_eq!(result.warning_count, 1);
    assert_eq!(result.error_count, 0);
}

#[test]
fn no_bare_strings_sfc_offsets_template_diagnostics() {
    let source = r#"<script setup lang="ts">
const label = "Hello";
</script>

<template>
  <div>Hello</div>
</template>
"#;
    let linter = linter_with(Box::new(NoBareStringsInTemplate));
    let result = linter.lint_sfc(source, "test.vue");

    assert_eq!(
        diagnostic_rules(&result),
        vec!["vue/no-bare-strings-in-template"],
        "SFC template bare text must report once: {:?}",
        result.diagnostics
    );

    let diag = &result.diagnostics[0];
    let text_start = source.rfind("Hello").unwrap() as u32;
    assert_eq!(
        diag.start, text_start,
        "SFC diagnostic must be offset into file coordinates"
    );
    assert_eq!(&source[diag.start as usize..diag.end as usize], "Hello");
}

#[test]
fn no_bare_strings_sfc_does_not_lint_script_tsx() {
    let source = r#"<script setup lang="tsx">
const A = () => <div>Hello</div>;
</script>
"#;
    let linter = linter_with(Box::new(NoBareStringsInTemplate));
    let result = linter.lint_sfc(source, "test.vue");

    assert_eq!(
        result.warning_count, 0,
        "SFC no-bare-strings must not inspect JSX inside script blocks: {:?}",
        result.diagnostics
    );
    assert_eq!(result.error_count, 0);
}
