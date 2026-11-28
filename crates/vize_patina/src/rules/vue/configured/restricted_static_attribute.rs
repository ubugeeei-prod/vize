//! `vue/no-restricted-static-attribute`.

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::ir::TemplateSyntax;
use crate::markup::{MarkupContext, MarkupDocument, MarkupRule, MarkupTraversal};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_l0::config::RestrictedNames;
use vize_l0::{String, cstr};
use vize_relief::RootNode;

use super::{hits, listed, report_template};

static META: RuleMeta = RuleMeta {
    name: "vue/no-restricted-static-attribute",
    description: "disallow specific attribute",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// `vue/no-restricted-static-attribute`.
#[derive(Default)]
pub struct NoRestrictedStaticAttribute {
    names: Vec<String>,
}

impl NoRestrictedStaticAttribute {
    /// Build the rule from `linter.ruleOptions["vue/no-restricted-static-attribute"]`.
    pub(crate) fn from_options(options: &RestrictedNames) -> Self {
        Self {
            names: listed(options),
        }
    }
}

impl Rule for NoRestrictedStaticAttribute {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_template<'a>(&self, ctx: &mut LintContext<'a>, root: &RootNode<'a>) {
        self.check_document(ctx, &MarkupDocument::new(root, TemplateSyntax::Vue));
    }

    fn as_markup_rule(&self) -> Option<&dyn MarkupRule> {
        Some(self)
    }

    fn jsx_needs_lowering(&self) -> bool {
        true
    }
}

impl NoRestrictedStaticAttribute {
    fn check_document(&self, ctx: &mut LintContext<'_>, document: &MarkupDocument<'_>) {
        if self.names.is_empty() {
            return;
        }
        let mut findings = Vec::new();
        document.walk_lint_headers(&mut |element| {
            element.walk_static_attributes(ctx.source, &mut |attribute| {
                if !hits(&self.names, attribute.name) {
                    return;
                }
                let span = if attribute.name_range.end > attribute.name_range.start {
                    attribute.name_range
                } else {
                    attribute.range
                };
                let name = attribute.name;
                findings.push((
                    cstr!("Using `{name}` is not allowed."),
                    span.start,
                    span.end,
                ));
            });
        });
        report_template(ctx, META.name, findings);
    }
}

impl MarkupRule for NoRestrictedStaticAttribute {
    fn name(&self) -> &'static str {
        META.name
    }

    fn traversal(&self) -> MarkupTraversal {
        MarkupTraversal::Document
    }

    fn enter_document(&self, ctx: &mut MarkupContext<'_, '_>, document: &MarkupDocument) {
        self.check_document(ctx.lint(), document);
    }
}
