//! Allowed style-block attribute modes.

use crate::context::LintContext;
use crate::diagnostic::{LintDiagnostic, Severity};
use crate::rule::{Rule, RuleCategory, RuleMeta};

static META: RuleMeta = RuleMeta {
    name: "vue/enforce-style-attribute",
    description: "Enforce allowed style-block attributes",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Authored style-block modes accepted by `vue/enforce-style-attribute`.
pub enum StyleAttribute {
    /// A plain `<style>` block without `scoped` or `module`.
    Plain,
    /// A `<style scoped>` block.
    Scoped,
    /// A `<style module>` block.
    Module,
}

/// `vue/enforce-style-attribute`; upstream defaults to `scoped` only.
pub struct EnforceStyleAttribute {
    allow_plain: bool,
    allow_scoped: bool,
    allow_module: bool,
}

impl Default for EnforceStyleAttribute {
    fn default() -> Self {
        Self {
            allow_plain: false,
            allow_scoped: true,
            allow_module: false,
        }
    }
}

impl EnforceStyleAttribute {
    /// Build the rule from the style modes accepted by the project.
    pub fn new(allowed: &[StyleAttribute]) -> Self {
        Self {
            allow_plain: allowed.contains(&StyleAttribute::Plain),
            allow_scoped: allowed.contains(&StyleAttribute::Scoped),
            allow_module: allowed.contains(&StyleAttribute::Module),
        }
    }
}

impl Rule for EnforceStyleAttribute {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_sfc<'a>(&self, ctx: &mut LintContext<'a>) {
        let finding = ctx.sfc_descriptor().and_then(|descriptor| {
            descriptor.styles.iter().find_map(|style| {
                let allowed = if style.scoped {
                    self.allow_scoped
                } else if style.module.is_some() {
                    self.allow_module
                } else {
                    self.allow_plain
                };
                (!allowed).then_some((style.loc.tag_start as u32, style.loc.start as u32))
            })
        });
        if let Some((start, end)) = finding {
            ctx.report_in_sfc(
                LintDiagnostic::warn(
                    META.name,
                    "This style block uses a disallowed style attribute mode",
                    start,
                    end,
                )
                .with_help("Use one of the style modes allowed by this rule's configuration"),
            );
        }
    }
}
