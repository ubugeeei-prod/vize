//! `vue/max-template-depth`.

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::ir::TemplateSyntax;
use crate::markup::{MarkupContext, MarkupDocument, MarkupRule, MarkupTraversal};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_l0::config::MaxTemplateDepthOptions;
use vize_l0::cstr;
use vize_relief::RootNode;

use super::report_template;

static META: RuleMeta = RuleMeta {
    name: "vue/max-template-depth",
    description: "enforce maximum depth of template",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// `vue/max-template-depth`.
#[derive(Default)]
pub struct MaxTemplateDepth {
    limit: Option<u32>,
}

impl MaxTemplateDepth {
    /// Build the rule from `linter.ruleOptions["vue/max-template-depth"]`.
    pub(crate) fn from_options(options: &MaxTemplateDepthOptions) -> Self {
        Self {
            limit: options.max_depth.filter(|value| *value >= 1),
        }
    }
}

impl Rule for MaxTemplateDepth {
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

impl MaxTemplateDepth {
    fn check_document(&self, ctx: &mut LintContext<'_>, document: &MarkupDocument<'_>) {
        let Some(limit) = self.limit else {
            return;
        };
        let mut findings = Vec::new();
        document.walk_lint_elements(true, &mut |element| {
            let depth = element.depth;
            if depth > limit {
                let range = element.range();
                findings.push((
                    cstr!(
                        "Element is nested too deeply (depth of {depth}, maximum allowed is {limit})."
                    ),
                    range.start,
                    range.end,
                ));
            }
        });
        report_template(ctx, META.name, findings);
    }
}

impl MarkupRule for MaxTemplateDepth {
    fn name(&self) -> &'static str {
        META.name
    }

    fn traversal(&self) -> MarkupTraversal {
        MarkupTraversal::Document
    }

    fn requires_original_element_syntax(&self) -> bool {
        self.limit.is_some()
    }

    fn enter_document(&self, ctx: &mut MarkupContext<'_, '_>, document: &MarkupDocument) {
        self.check_document(ctx.lint(), document);
    }
}
