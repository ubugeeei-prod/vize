//! vue/no-preprocessor-lang
//!
//! Discourage use of CSS preprocessors (sass, scss, less, stylus) in Vue SFCs.

use crate::context::LintContext;
use crate::diagnostic::{LintDiagnostic, Severity};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_atelier_sfc::{SfcParseOptions, parse_sfc};
use vize_l0::profile;

static META: RuleMeta = RuleMeta {
    name: "vue/no-preprocessor-lang",
    description: "Discourage CSS preprocessor usage in favor of modern CSS",
    category: RuleCategory::Recommended,
    // Removing lang without translating the block body corrupts authored
    // SCSS/Less/Stylus, so this policy diagnostic has no automatic edit.
    fixable: false,
    default_severity: Severity::Warning,
};

/// No preprocessor lang rule.
#[derive(Default)]
pub struct NoPreprocessorLang;

impl Rule for NoPreprocessorLang {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_sfc<'a>(&self, ctx: &mut LintContext<'a>) {
        let owned_descriptor;
        let descriptor = if let Some(descriptor) = ctx.sfc_descriptor() {
            descriptor
        } else {
            owned_descriptor = match profile!(
                "patina.rule.no_preprocessor_lang.parse_sfc",
                parse_sfc(
                    ctx.source,
                    SfcParseOptions {
                        filename: ctx.filename.into(),
                        ..Default::default()
                    },
                )
            ) {
                Ok(descriptor) => descriptor,
                Err(_) => return,
            };
            &owned_descriptor
        };

        let spans: Vec<_> = descriptor
            .styles
            .iter()
            .filter(|style| {
                style.lang.as_deref().is_some_and(|lang| {
                    matches!(
                        lang.to_ascii_lowercase().as_str(),
                        "sass" | "scss" | "less" | "stylus" | "styl"
                    )
                })
            })
            .map(|style| (style.loc.tag_start as u32, style.loc.start as u32))
            .collect();

        for (start, end) in spans {
            ctx.report_in_sfc(
                LintDiagnostic::warn(
                    META.name,
                    "Consider using modern CSS instead of preprocessor",
                    start,
                    end,
                )
                .with_help(
                    "Modern CSS supports nesting, variables, and more. Consider migrating to plain CSS.",
                ),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::NoPreprocessorLang;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(NoPreprocessorLang));
        Linter::with_registry(registry)
    }

    #[test]
    fn reports_each_preprocessor_style_without_an_unsafe_fix() {
        let source = r#"<template><div /></template>
<style lang='scss' scoped>$gap: 4px; .box { margin: $gap; }</style>
<style lang="less">@gap: 4px;</style>"#;
        let result = linter().lint_sfc(source, "Box.vue");
        assert_eq!(result.warning_count, 2, "{:?}", result.diagnostics);
        assert!(result.diagnostics.iter().all(|diagnostic| {
            diagnostic.rule_name == "vue/no-preprocessor-lang" && diagnostic.fix.is_none()
        }));
        assert_eq!(
            result.diagnostics[0].start as usize,
            source.find("<style lang='scss'").unwrap()
        );
    }

    #[test]
    fn allows_plain_css_and_non_style_text() {
        let source = r#"<template><pre>&lt;style lang="scss"&gt;</pre></template>
<style scoped>.box { color: red; }</style>"#;
        assert_eq!(linter().lint_sfc(source, "Box.vue").warning_count, 0);
    }
}
