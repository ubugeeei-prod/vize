//! `sameNameShorthand`: one spelling for a binding that reads its own argument.
//!
//! `always` requires `:foo` where `:foo="foo"` is written, and `never` requires
//! the written value.

use super::META;
use crate::context::LintContext;
use crate::diagnostic::{Fix, LintDiagnostic, TextEdit};
use crate::markup::{MarkupBinding, SameNameSpelling};
use vize_l0::cstr;
use vize_relief::DirectiveNode;

/// Report `directive` when it is a same-name binding not written as `required`.
pub(super) fn check(
    ctx: &mut LintContext<'_>,
    directive: &DirectiveNode<'_>,
    required: SameNameSpelling,
) {
    // Only the spelling the policy rejects is looked for.
    let rejected = match required {
        SameNameSpelling::Shorthand => SameNameSpelling::Explicit,
        SameNameSpelling::Explicit => SameNameSpelling::Shorthand,
    };
    let Some(same_name) =
        MarkupBinding::from_relief_directive(directive).same_name_binding_written(rejected)
    else {
        return;
    };
    let (start, end) = (directive.loc.span.start, directive.loc.span.end);
    let (message, help, fix) = match required {
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
