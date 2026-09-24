//! vue/no-src-attribute
//!
//! Discourage external `src` attributes on top-level Vue SFC blocks.
//! Keeping the template, script, and styles in one file makes a component
//! easier to inspect and better supported by build tools and editors.
//!
//! Ordinary HTML documents may load external scripts and styles; only
//! parsed SFC blocks are subject to this rule.

use crate::context::LintContext;
use crate::diagnostic::{LintDiagnostic, Severity};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_atelier_sfc::{SfcParseOptions, parse_sfc};
use vize_s0::profile;

static META: RuleMeta = RuleMeta {
    name: "vue/no-src-attribute",
    description: "Discourage src attribute on SFC blocks",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// No src attribute rule.
#[derive(Default)]
pub struct NoSrcAttribute;

impl Rule for NoSrcAttribute {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_sfc<'a>(&self, ctx: &mut LintContext<'a>) {
        // Reuse the engine's parsed descriptor so that only top-level SFC
        // blocks are inspected. Parse lazily for callers without one.
        let owned_descriptor;
        let descriptor = if let Some(descriptor) = ctx.sfc_descriptor() {
            descriptor
        } else {
            owned_descriptor = match profile!(
                "patina.rule.no_src_attribute.parse_sfc",
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

        let mut spans = Vec::new();
        if let Some(template) = &descriptor.template
            && (template.src.is_some() || template.attrs.contains_key(":src"))
        {
            spans.push((template.loc.tag_start as u32, template.loc.start as u32));
        }
        for script in [&descriptor.script, &descriptor.script_setup]
            .into_iter()
            .flatten()
        {
            if script.src.is_some() || script.attrs.contains_key(":src") {
                spans.push((script.loc.tag_start as u32, script.loc.start as u32));
            }
        }
        for style in &descriptor.styles {
            if style.src.is_some() || style.attrs.contains_key(":src") {
                spans.push((style.loc.tag_start as u32, style.loc.start as u32));
            }
        }

        for (start, end) in spans {
            ctx.report(
                LintDiagnostic::warn(
                    META.name,
                    "Avoid using src attribute on SFC blocks",
                    start,
                    end,
                )
                .with_help(
                    "Keep all component code in the same .vue file for better maintainability",
                ),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::NoSrcAttribute;
    use crate::linter::Linter;
    use crate::preset::LintPreset;
    use crate::rule::RuleRegistry;

    fn linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(NoSrcAttribute));
        Linter::with_registry(registry)
    }

    #[test]
    fn allows_external_assets_in_standalone_html() {
        let source = r#"<html><head>
<script type="module" src="/src/main.ts"></script>
<style src="/src/site.css"></style>
</head><body></body></html>"#;
        let result =
            Linter::with_preset(LintPreset::Opinionated).lint_standalone_html(source, "index.html");
        assert!(
            result
                .diagnostics
                .iter()
                .all(|diagnostic| diagnostic.rule_name != "vue/no-src-attribute"),
            "{:?}",
            result.diagnostics
        );
    }

    #[test]
    fn reports_only_top_level_sfc_blocks_with_src() {
        let source = r#"<template src="./view.html"></template>
<script src="./logic.ts"></script>
<style src="./theme.css"></style>"#;
        let result = linter().lint_sfc(source, "Example.vue");
        let starts: Vec<_> = result
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.rule_name == "vue/no-src-attribute")
            .map(|diagnostic| diagnostic.start as usize)
            .collect();
        assert_eq!(
            starts,
            [
                source.find("<template").unwrap(),
                source.find("<script").unwrap(),
                source.find("<style").unwrap(),
            ]
        );
    }

    #[test]
    fn ignores_external_scripts_inside_sfc_templates() {
        let source = r#"<template>
  <script src="/widget.js"></script>
  <div>Widget</div>
</template>"#;
        let result = linter().lint_sfc(source, "Example.vue");
        assert!(
            result
                .diagnostics
                .iter()
                .all(|diagnostic| diagnostic.rule_name != "vue/no-src-attribute"),
            "{:?}",
            result.diagnostics
        );
    }
}
