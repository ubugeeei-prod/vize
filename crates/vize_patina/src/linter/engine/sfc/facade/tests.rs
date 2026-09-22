//! The migrated rules preserve full diagnostics on the real SFC entry point.

use crate::diagnostic::Severity;
use crate::linter::Linter;
use crate::linter::engine::offset_result;
use vize_s0::ToCompactString;

fn linter() -> Linter {
    Linter {
        enabled_rules: Some(
            super::RULES
                .iter()
                .map(|rule| rule.to_compact_string())
                .collect(),
        ),
        ..Linter::default()
    }
}

fn compare_template(source: &str) {
    let prefix = "<script setup>const note = '日本語😀'</script>\r\n<template>";
    let sfc = format!("{prefix}{source}</template>");
    let linter = linter();
    let mut expected = linter.lint_template(source, "test.vue");
    // A raw template is the baseline: only its file offset is adjusted.
    offset_result(&mut expected, prefix.len() as u32);
    let actual = linter.lint_sfc(&sfc, "test.vue");
    assert_eq!(actual.error_count, expected.error_count, "{source}");
    assert_eq!(actual.warning_count, expected.warning_count, "{source}");
    assert_eq!(
        format!("{:?}", actual.diagnostics),
        format!("{:?}", expected.diagnostics),
        "{source}"
    );
}

#[test]
fn migrated_sfc_rules_preserve_diagnostics_and_fixes() {
    for source in [
        r#"<div  style="color:red"><textarea>{{ message }}</textarea><button>go</button></div>"#,
        r#"<a href="https://example.com" target="_blank">go</a><iframe src="/frame"/><input disabled="true"/><div contenteditable="invalid"/>"#,
        r#"<a href="https://example.com" target="_blank" rel="noopener noreferrer">go</a><iframe sandbox=""/><button type="button">go</button><input disabled/>"#,
        r#"<Comp><template #default><textarea>{{ value }}</textarea><button>go</button></template></Comp>"#,
        "<!-- eslint-disable-next-line vue/no-inline-style -->\n<div style=\"color:red\"/>",
        r#"<div v-pre><textarea>{{ raw }}</textarea></div>"#,
    ] {
        compare_template(source);
    }
}

#[test]
fn disabled_rules_and_severity_overrides_apply_to_the_batch() {
    let linter = linter()
        .with_disabled_rules(vec!["vue/no-inline-style".into()])
        .with_rule_severity_overrides(vec![("vue/no-textarea-mustache".into(), Severity::Warning)]);
    let source = "<template><textarea aria-label=\"Message\" style=\"color:red\">{{ value }}</textarea></template>";
    let result = linter.lint_sfc(source, "test.vue");
    assert_eq!(result.diagnostics.len(), 1, "{:?}", result.diagnostics);
    assert_eq!(result.diagnostics[0].rule_name, "vue/no-textarea-mustache");
    assert_eq!(result.diagnostics[0].severity, Severity::Warning);
}

#[test]
fn sfc_batch_does_not_inspect_script_jsx() {
    let result = linter().lint_sfc(
        r#"<script setup lang="tsx">
const view = <button style="color:red">go</button>;
</script>"#,
        "test.vue",
    );
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
}

mod battery;

struct FacadeOnly(&'static crate::rule::RuleMeta);

impl crate::markup::MarkupRule for FacadeOnly {
    fn name(&self) -> &'static str {
        self.0.name
    }
    fn enter_element<'a>(
        &self,
        ctx: &mut crate::markup::MarkupContext<'_, 'a>,
        _: &crate::markup::MarkupElement<'a>,
    ) {
        ctx.lint()
            .error_at("facade", crate::ir::ByteRange::new(0, 1));
    }
}

impl crate::rule::Rule for FacadeOnly {
    fn meta(&self) -> &'static crate::rule::RuleMeta {
        self.0
    }
    fn as_markup_rule(&self) -> Option<&dyn crate::markup::MarkupRule> {
        Some(self)
    }
    fn enter_element<'a>(
        &self,
        _: &mut crate::context::LintContext<'a>,
        _: &vize_relief::ElementNode<'a>,
    ) {
        panic!("an admitted SFC rule reached the legacy element visitor");
    }
}

#[test]
fn every_admitted_rule_reaches_only_the_s2_facade() {
    let all = crate::rule::RuleRegistry::with_all();
    for name in super::RULES {
        let meta = all
            .rules()
            .iter()
            .find(|rule| rule.meta().name == *name)
            .unwrap()
            .meta();
        let mut registry = crate::rule::RuleRegistry::new();
        registry.register(Box::new(FacadeOnly(meta)));
        let result =
            Linter::with_registry(registry).lint_sfc("<template><div/></template>", "test.vue");
        assert_eq!(result.diagnostics.len(), 1, "{name}");
        assert_eq!(result.diagnostics[0].rule_name, *name);
        assert_eq!(result.diagnostics[0].message, "facade");
    }
}
