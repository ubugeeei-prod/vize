//! Frozen pre-migration implementation used as an independent oracle.

//! `vue/no-restricted-html-elements`.

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_l0::config::RestrictedHtmlElements;
use vize_l0::{String, cstr};
use vize_relief::{ElementType, RootNode};

use super::report_template;
use super::template_walk::walk;

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
        if self.restrictions.is_empty() {
            return;
        }
        let mut findings = Vec::new();
        walk(&root.children, 1, &mut |element, _depth| {
            if element.tag_type != ElementType::Element {
                return;
            }
            let Some((_, message)) = self
                .restrictions
                .iter()
                .find(|(name, _)| name.as_str() == element.tag)
            else {
                return;
            };
            findings.push((
                match message {
                    Some(message) => message.clone(),
                    None => cstr!("Unexpected use of forbidden element {}.", element.tag),
                },
                element.loc.span.start,
                element.loc.span.end,
            ));
        });
        report_template(ctx, META.name, findings);
    }
}
