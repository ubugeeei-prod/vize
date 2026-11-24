//! `vue/no-restricted-v-on`.

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_l0::config::{RestrictedVOn, RestrictedVOnSpec};
use vize_l0::{String, cstr};
use vize_relief::{DirectiveNode, ElementNode, ExpressionNode, PropNode, RootNode};

use super::report_template;
use super::template_walk::walk;

static META: RuleMeta = RuleMeta {
    name: "vue/no-restricted-v-on",
    description: "disallow specific argument in `v-on`",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// `vue/no-restricted-v-on`.
#[derive(Default)]
pub struct NoRestrictedVOn {
    /// In authored order. The first restriction a listener meets reports it.
    restrictions: Vec<RestrictedVOnSpec>,
}

impl NoRestrictedVOn {
    /// Build the rule from `linter.ruleOptions["vue/no-restricted-v-on"]`.
    pub(crate) fn from_options(options: &RestrictedVOn) -> Self {
        Self {
            restrictions: options
                .restrictions()
                .into_iter()
                .filter(|restriction| !restriction.argument.is_empty())
                // eslint-plugin-vue reads an empty message as no message.
                .map(|restriction| RestrictedVOnSpec {
                    message: restriction.message.filter(|message| !message.is_empty()),
                    ..restriction
                })
                .collect(),
        }
    }
}

/// Whether `restriction` covers the listener `name` on `element`: the same
/// event, on the named element if one is named, carrying every listed modifier.
fn covers(
    restriction: &RestrictedVOnSpec,
    element: &ElementNode<'_>,
    directive: &DirectiveNode<'_>,
    name: &str,
) -> bool {
    restriction.argument.as_str() == name
        && restriction
            .element
            .as_ref()
            .is_none_or(|tag| tag.as_str() == element.tag)
        && restriction.modifiers.iter().all(|wanted| {
            directive
                .modifiers
                .iter()
                .any(|modifier| modifier.content == wanted.as_str())
        })
}

/// The message of a restriction that carries none: the event with the
/// restricted modifiers, and the element when the restriction names one.
fn default_message(restriction: &RestrictedVOnSpec, name: &str) -> String {
    let mut listener = cstr!("@{name}");
    for modifier in &restriction.modifiers {
        listener.push('.');
        listener.push_str(modifier);
    }
    match &restriction.element {
        Some(tag) => cstr!("Using `{listener}` is not allowed on this <{tag}>."),
        None => cstr!("Using `{listener}` is not allowed on this element."),
    }
}

impl Rule for NoRestrictedVOn {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_template<'a>(&self, ctx: &mut LintContext<'a>, root: &RootNode<'a>) {
        if self.restrictions.is_empty() {
            return;
        }
        let mut findings = Vec::new();
        walk(&root.children, 1, &mut |element, _depth| {
            for prop in &element.props {
                let PropNode::Directive(directive) = prop else {
                    continue;
                };
                if directive.name != "on" {
                    continue;
                }
                let Some(ExpressionNode::Simple(argument)) = &directive.arg else {
                    continue;
                };
                if !argument.is_static {
                    continue;
                }
                let name = argument.content;
                let Some(restriction) = self
                    .restrictions
                    .iter()
                    .find(|restriction| covers(restriction, element, directive, name))
                else {
                    continue;
                };
                findings.push((
                    match &restriction.message {
                        Some(message) => message.clone(),
                        None => default_message(restriction, name),
                    },
                    directive.loc.span.start,
                    directive.loc.span.end,
                ));
            }
        });
        report_template(ctx, META.name, findings);
    }
}
