//! Source heuristics that decide whether ecosystem-specific template rules
//! can be skipped for a given source.

use super::tag_scan::{find_tag_end, tag_name_at};

pub(super) fn source_may_contain_ecosystem_template_rule(
    template_source: &str,
    sfc_source: Option<&str>,
) -> bool {
    let template_bytes = template_source.as_bytes();
    if template_may_contain_ecosystem_element(template_bytes, sfc_source) {
        return true;
    }

    sfc_source.is_some_and(|source| source.contains("<i18n"))
        && template_may_call_i18n(template_bytes)
}

fn template_may_contain_ecosystem_element(bytes: &[u8], sfc_source: Option<&str>) -> bool {
    let imports_void_vue = sfc_source.is_some_and(|source| source.contains("@void/vue"));
    let mut cursor = 0;
    while let Some(relative) = bytes
        .get(cursor..)
        .and_then(|rest| memchr::memchr(b'<', rest))
    {
        let tag_start = cursor + relative;
        let Some((tag_name, name_end)) = tag_name_at(bytes, tag_start) else {
            cursor = tag_start + 1;
            continue;
        };

        if matches!(
            tag_name,
            b"RouterLink" | b"router-link" | b"NuxtLink" | b"nuxt-link"
        ) {
            return true;
        }
        if tag_name == b"Link" && imports_void_vue {
            return true;
        }

        let Some(tag_end) = find_tag_end(bytes, name_end) else {
            return false;
        };
        cursor = tag_end + 1;
    }
    false
}

fn template_may_call_i18n(bytes: &[u8]) -> bool {
    memchr::memmem::find(bytes, b"$t(").is_some()
        || memchr::memmem::find(bytes, b"$te(").is_some()
        || memchr::memmem::find(bytes, b"$tm(").is_some()
        || memchr::memmem::find(bytes, b"t(").is_some()
        || memchr::memmem::find(bytes, b"te(").is_some()
        || memchr::memmem::find(bytes, b"tm(").is_some()
}

#[cfg(test)]
mod tests {
    #![expect(
        clippy::disallowed_macros,
        reason = "compare complete diagnostic Debug packets"
    )]

    use super::source_may_contain_ecosystem_template_rule;
    use crate::context::LintContext;
    use crate::diagnostic::{Fix, LintDiagnostic, Severity, TextEdit};
    use crate::rule::{Rule, RuleCategory, RuleMeta, RuleRegistry};
    use crate::{LintPreset, Linter};
    use vize_relief::ElementNode;

    #[test]
    fn ecosystem_template_hint_ignores_plain_internal_anchor() {
        assert!(!source_may_contain_ecosystem_template_rule(
            r#"<a href = "/docs">Docs</a>"#,
            None
        ));
    }

    #[test]
    fn ecosystem_template_hint_ignores_bound_href_with_script_path_strings() {
        let template = r#"<a :href="link.url">Docs</a>"#;
        let sfc = r#"<template><a :href="link.url">Docs</a></template>
<script setup>
const links = [{ url: '/docs' }]
</script>"#;
        assert!(!source_may_contain_ecosystem_template_rule(
            template,
            Some(sfc)
        ));
    }

    static CUSTOM_META: RuleMeta = RuleMeta {
        name: "custom/button",
        description: "Check host buttons",
        category: RuleCategory::Ecosystem,
        fixable: true,
        default_severity: Severity::Error,
    };

    struct CustomButtonRule(&'static RuleMeta);

    impl Rule for CustomButtonRule {
        fn meta(&self) -> &'static RuleMeta {
            self.0
        }

        fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
            if element.tag != "button" {
                return;
            }
            let start = element.loc.span.start;
            let end = start + 7;
            ctx.report(
                LintDiagnostic::error(self.meta().name, "Host button rule", start, end)
                    .with_help("Keep the host-provided rule active.")
                    .with_label("Button", start, end)
                    .with_fix(Fix::new(
                        "Rename button",
                        TextEdit::replace(start + 1, end, "input"),
                    )),
            );
        }
    }

    #[test]
    fn ecosystem_source_pruning_preserves_custom_rules() {
        enum Input {
            Template,
            Sfc,
            Html,
        }

        for (source, input) in [
            ("<button>Save</button>", Input::Template),
            ("<button>Save</button>", Input::Html),
            ("<template><button>Save</button></template>", Input::Sfc),
            ("<button>Save</button><RouterLink />", Input::Template),
            (
                "<template><button>Save</button><RouterLink /></template>",
                Input::Sfc,
            ),
        ] {
            let expected =
                Linter::with_ecosystem().with_rule(Box::new(CustomButtonRule(&CUSTOM_META)));
            let actual = Linter::with_preset(LintPreset::Ecosystem)
                .with_rule(Box::new(CustomButtonRule(&CUSTOM_META)));
            let lint = |linter: &Linter| match input {
                Input::Sfc => linter.lint_sfc(source, "Custom.vue"),
                Input::Html => linter.lint_standalone_html(source, "Custom.html"),
                Input::Template => linter.lint_template(source, "Custom.vue"),
            };
            let expected = lint(&expected);
            let actual = lint(&actual);
            assert_eq!(
                actual
                    .diagnostics
                    .iter()
                    .filter(|d| d.rule_name == "custom/button")
                    .count(),
                1,
                "{source}: {:?}",
                actual.diagnostics,
            );
            assert_eq!(
                format!("{:?}", actual.diagnostics),
                format!("{:?}", expected.diagnostics),
                "{source}"
            );
            assert_eq!(
                (actual.error_count, actual.warning_count),
                (expected.error_count, expected.warning_count)
            );
        }
    }

    #[test]
    fn ecosystem_source_pruning_keeps_the_pristine_builtin_prefix() {
        let prefix = RuleRegistry::with_happy_path().rules().len();
        for (linter, expected_count) in [
            (Linter::with_preset(LintPreset::Ecosystem), Some(prefix)),
            (Linter::with_ecosystem(), None),
            (Linter::with_registry(RuleRegistry::with_ecosystem()), None),
        ] {
            assert_eq!(
                linter.template_rule_count_for_source("<div />", None),
                expected_count.unwrap_or(linter.registry().rules().len()),
            );
            assert_eq!(
                linter.template_rule_count_for_source("<RouterLink />", None),
                linter.registry().rules().len(),
            );
            let result = linter.lint_template("<RouterLink />", "Custom.vue");
            assert!(
                result
                    .diagnostics
                    .iter()
                    .any(|d| d.rule_name == "ecosystem/router-link-require-to")
            );
        }
    }

    #[test]
    fn ecosystem_source_pruning_preserves_replaced_rule_instances() {
        static REPLACEMENT_META: RuleMeta = RuleMeta {
            name: "ecosystem/router-link-require-to",
            description: "Host replacement checks buttons",
            category: RuleCategory::Ecosystem,
            fixable: true,
            default_severity: Severity::Error,
        };
        let mut registry = RuleRegistry::with_ecosystem();
        registry.replace(Box::new(CustomButtonRule(&REPLACEMENT_META)));
        assert!(registry.ecosystem_template_prefix().is_none());
        let linter = Linter::with_registry(registry);
        let result = linter.lint_template("<button>Save</button>", "Custom.vue");
        let diagnostic = result
            .diagnostics
            .iter()
            .find(|d| d.rule_name == REPLACEMENT_META.name)
            .expect("host replacement must run on its own source shape");
        assert_eq!((diagnostic.start, diagnostic.end), (0, 7));
        assert_eq!(diagnostic.message, "Host button rule");
        assert!(diagnostic.fix.is_some());

        let clean = linter.lint_template("<div></div>", "Custom.vue");
        assert!(
            clean
                .diagnostics
                .iter()
                .all(|d| d.rule_name != REPLACEMENT_META.name)
        );
    }
}
