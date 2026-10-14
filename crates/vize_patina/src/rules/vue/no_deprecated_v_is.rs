//! vue/no-deprecated-v-is
//!
//! Disallow the deprecated `v-is` directive.
//!
//! Vue 3.1 deprecated `v-is` in favor of the `is` attribute with the `vue:`
//! prefix.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <tr v-is="'MyRow'"></tr>
//! ```
//!
//! ### Valid
//! ```vue
//! <tr is="vue:MyRow"></tr>
//! <component :is="MyRow" />
//! ```

use crate::context::LintContext;
use crate::diagnostic::{LintDiagnostic, Severity};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_l0::dialect::VueDialect;
use vize_relief::{DirectiveNode, ElementNode, ExpressionNode};

static META: RuleMeta = RuleMeta {
    name: "vue/no-deprecated-v-is",
    description: "Disallow the deprecated `v-is` directive",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Error,
};

/// Disallow the deprecated `v-is` directive.
pub struct NoDeprecatedVIs;

impl Rule for NoDeprecatedVIs {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn check_directive<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        _element: &ElementNode<'a>,
        directive: &DirectiveNode<'a>,
    ) {
        // Only the default Vue 3 dialect ever shipped `v-is` as a built-in.
        if directive.name != "is" || ctx.dialect() != VueDialect::Vue {
            return;
        }

        // Report the directive key (`v-is`, with any argument and modifiers)
        // rather than its value, as eslint-plugin-vue does.
        let start = directive.loc.span.start;
        let end = key_end(directive).unwrap_or(directive.loc.span.end);
        ctx.report(
            LintDiagnostic::warn(
                META.name,
                ctx.t("vue/no-deprecated-v-is.message").as_ref(),
                start,
                end,
            )
            .with_help(ctx.t("vue/no-deprecated-v-is.help").as_ref()),
        );
    }
}

/// End offset of the directive key: the last modifier, else the argument,
/// else the bare `v-is` name.
fn key_end(directive: &DirectiveNode) -> Option<u32> {
    if let Some(last) = directive.modifiers.last() {
        return Some(last.loc.span.end);
    }
    if let Some(arg) = &directive.arg {
        return Some(match arg {
            ExpressionNode::Simple(simple) => simple.loc.span.end,
            ExpressionNode::Compound(compound) => compound.loc.span.end,
        });
    }
    let name_len = u32::try_from(directive.raw_name?.len()).ok()?;
    directive.loc.span.start.checked_add(name_len)
}

#[cfg(test)]
mod tests;
