//! vue/no-template-lang
//!
//! Discourage use of template preprocessors such as Pug in Vue SFCs.

use crate::context::LintContext;
use crate::diagnostic::{LintDiagnostic, Severity};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_atelier_sfc::{SfcParseOptions, parse_sfc};
use vize_l0::profile;

static META: RuleMeta = RuleMeta {
    name: "vue/no-template-lang",
    description: "Discourage lang attribute on template block",
    category: RuleCategory::Recommended,
    // Deleting the attribute leaves preprocessor syntax behind and changes the
    // meaning of the file. Converting the body requires a dedicated formatter.
    fixable: false,
    default_severity: Severity::Warning,
};

/// No template lang rule.
#[derive(Default)]
pub struct NoTemplateLang;

impl Rule for NoTemplateLang {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_sfc<'a>(&self, ctx: &mut LintContext<'a>) {
        let owned_descriptor;
        let descriptor = if let Some(descriptor) = ctx.sfc_descriptor() {
            descriptor
        } else {
            owned_descriptor = match profile!(
                "patina.rule.no_template_lang.parse_sfc",
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

        let Some(template) = descriptor.template.as_ref() else {
            return;
        };
        let is_preprocessor = template.lang.as_deref().is_some_and(|lang| {
            matches!(
                lang.to_ascii_lowercase().as_str(),
                "pug" | "jade" | "slm" | "haml"
            )
        });
        if !is_preprocessor {
            return;
        }

        ctx.report_in_sfc(
            LintDiagnostic::warn(
                META.name,
                "Avoid using template preprocessor for better tooling support",
                template.loc.tag_start as u32,
                template.loc.start as u32,
            )
            .with_help("Use native HTML templates for better IDE support and maintainability"),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::NoTemplateLang;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(NoTemplateLang));
        Linter::with_registry(registry)
    }

    #[test]
    fn reports_pug_without_an_unsafe_fix() {
        let source = "<template lang='pug'>\ndiv.box hello\n</template>";
        let result = linter().lint_sfc(source, "PugBox.vue");
        let diagnostics: Vec<_> = result
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.rule_name == "vue/no-template-lang")
            .collect();
        assert_eq!(diagnostics.len(), 1, "{:?}", result.diagnostics);
        assert_eq!(diagnostics[0].start, 0);
        assert_eq!(diagnostics[0].end as usize, source.find('>').unwrap() + 1);
        assert!(diagnostics[0].fix.is_none());
    }

    #[test]
    fn allows_plain_html_templates() {
        assert_eq!(
            linter()
                .lint_sfc("<template><div>Hello</div></template>", "Box.vue")
                .warning_count,
            0
        );
    }
}
