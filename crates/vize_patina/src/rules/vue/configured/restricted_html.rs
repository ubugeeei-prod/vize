//! `vue/no-restricted-html-elements`.

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::ir::TemplateSyntax;
use crate::markup::{MarkupContext, MarkupDocument, MarkupRule, MarkupTraversal};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_l0::config::RestrictedHtmlElements;
use vize_l0::{String, cstr};
use vize_relief::RootNode;

use super::report_template;

static META: RuleMeta = RuleMeta {
    name: "vue/no-restricted-html-elements",
    description: "disallow specific elements",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// `vue/no-restricted-html-elements`.
#[derive(Default)]
pub struct NoRestrictedHtmlElements {
    /// Each restricted name with the message its entry carries, in authored
    /// order. The first entry that names an element decides its message.
    restrictions: Vec<(String, Option<String>)>,
}

impl NoRestrictedHtmlElements {
    /// Build the rule from `linter.ruleOptions["vue/no-restricted-html-elements"]`.
    pub(crate) fn from_options(options: &RestrictedHtmlElements) -> Self {
        Self {
            restrictions: options
                .restrictions()
                .into_iter()
                .filter(|(name, _)| !name.is_empty())
                // eslint-plugin-vue reads an empty message as no message.
                .map(|(name, message)| (name, message.filter(|message| !message.is_empty())))
                .collect(),
        }
    }
}

impl Rule for NoRestrictedHtmlElements {
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

impl NoRestrictedHtmlElements {
    fn check_document(&self, ctx: &mut LintContext<'_>, document: &MarkupDocument<'_>) {
        if self.restrictions.is_empty() {
            return;
        }
        let mut findings = Vec::new();
        document.walk_lint_elements(false, &mut |element| {
            if !element.is_plain() {
                return;
            }
            let Some((_, message)) = self
                .restrictions
                .iter()
                .find(|(name, _)| name.as_str() == element.tag())
            else {
                return;
            };
            let range = element.range();
            findings.push((
                match message {
                    Some(message) => message.clone(),
                    None => cstr!("Unexpected use of forbidden element {}.", element.tag()),
                },
                range.start,
                range.end,
            ));
        });
        report_template(ctx, META.name, findings);
    }
}

impl MarkupRule for NoRestrictedHtmlElements {
    fn name(&self) -> &'static str {
        META.name
    }

    fn traversal(&self) -> MarkupTraversal {
        MarkupTraversal::Document
    }

    fn requires_original_element_syntax(&self) -> bool {
        !self.restrictions.is_empty()
    }

    fn enter_document(&self, ctx: &mut MarkupContext<'_, '_>, document: &MarkupDocument) {
        self.check_document(ctx.lint(), document);
    }
}
