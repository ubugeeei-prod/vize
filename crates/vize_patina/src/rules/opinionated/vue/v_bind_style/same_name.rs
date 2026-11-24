//! `sameNameShorthand`: one spelling for a binding that reads its own argument.
//!
//! `always` requires `:foo` where `:foo="foo"` is written, and `never` requires
//! the written value. The policy is its own rule object, so `vue/v-bind-style`
//! without the option runs exactly the code it ran before the option existed.

use super::{META, VBindStyle};
use crate::context::LintContext;
use crate::diagnostic::{Fix, LintDiagnostic, TextEdit};
use crate::markup::{MarkupBinding, SameNameSpelling};
use crate::rule::{Rule, RuleHooks, RuleMeta};
use vize_l0::cstr;
use vize_relief::{DirectiveNode, ElementNode};

/// `vue/v-bind-style` with `sameNameShorthand` set to `always` or `never`.
pub(crate) struct VBindStyleSameName {
    style: VBindStyle,
    required: SameNameSpelling,
}

impl VBindStyleSameName {
    /// Require `required` of every same-name binding, on top of `style`.
    pub(crate) const fn new(style: VBindStyle, required: SameNameSpelling) -> Self {
        Self { style, required }
    }

    fn check_same_name(&self, ctx: &mut LintContext<'_>, directive: &DirectiveNode<'_>) {
        let Some(same_name) = MarkupBinding::from_relief_directive(directive).same_name_binding()
        else {
            return;
        };
        if same_name.spelling == self.required {
            return;
        }
        let (start, end) = (directive.loc.span.start, directive.loc.span.end);
        let (message, help, fix) = match self.required {
            SameNameSpelling::Shorthand => (
                "Expected same-name shorthand.",
                "Use `:attr` instead of `:attr=\"attr\"`",
                same_name_shorthand_fix(ctx.source, start, end),
            ),
            SameNameSpelling::Explicit => (
                "Unexpected same-name shorthand.",
                "Use `:attr=\"attr\"` instead of `:attr`",
                Some(Fix::new(
                    "Write the bound value",
                    TextEdit::insert(end, cstr!("=\"{}\"", same_name.operand())),
                )),
            ),
        };
        let mut diagnostic = LintDiagnostic::warn(META.name, message, start, end).with_help(help);
        if let Some(fix) = fix {
            diagnostic = diagnostic.with_fix(fix);
        }
        ctx.report(diagnostic);
    }
}

impl Rule for VBindStyleSameName {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn template_hooks(&self) -> RuleHooks {
        RuleHooks::DIRECTIVE
    }

    fn check_directive<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        element: &ElementNode<'a>,
        directive: &DirectiveNode<'a>,
    ) {
        self.check_same_name(ctx, directive);
        self.style.check_directive(ctx, element, directive);
    }
}

/// Delete `="value"` from `:user-name="userName"`, leaving `:user-name`.
pub(in crate::rules::opinionated::vue) fn same_name_shorthand_fix(
    source: &str,
    start: u32,
    end: u32,
) -> Option<Fix> {
    let start = start as usize;
    let end = end as usize;
    let slice = source.get(start..end)?;
    let eq = slice.find('=')?;
    let mut trim = eq;
    while trim > 0
        && slice
            .as_bytes()
            .get(trim - 1)
            .is_some_and(u8::is_ascii_whitespace)
    {
        trim -= 1;
    }
    if trim == 0 {
        return None;
    }
    Some(Fix::new(
        "Use shorthand prop syntax",
        TextEdit::delete((start + trim) as u32, end as u32),
    ))
}

#[cfg(test)]
mod tests;
