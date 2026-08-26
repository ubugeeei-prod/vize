//! vue/no-use-v-else-with-v-for
//!
//! Disallow `v-else-if` or `v-else` on the same element as `v-for`.
//!
//! Vue accepts this shape and gives the conditional directive precedence, but
//! the mixed control flow is difficult to read. Put the conditional on a
//! wrapper element and keep `v-for` on the repeated element instead.

#[cfg(test)]
mod tests;

use crate::context::LintContext;
use crate::diagnostic::{LintDiagnostic, Severity};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::{ElementNode, PropNode, SourceLocation, TemplateChildNode};
use vize_s0::ensure_sufficient_stack;

static META: RuleMeta = RuleMeta {
    name: "vue/no-use-v-else-with-v-for",
    description: "Disallow using `v-else-if` or `v-else` on the same element as `v-for`",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Disallow conditional fallback branches on the same element as `v-for`.
pub struct NoUseVElseWithVFor;

impl Rule for NoUseVElseWithVFor {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        if !has_directive(element, "for") {
            return;
        }

        // Match upstream precedence when malformed markup contains both.
        let directive = if has_directive(element, "else-if") {
            "v-else-if"
        } else if has_directive(element, "else") {
            "v-else"
        } else {
            return;
        };

        let loc = full_element_loc(ctx.source, element);
        let message = ctx.t_fmt(
            "vue/no-use-v-else-with-v-for.message",
            &[("directive", directive)],
        );
        ctx.report(
            LintDiagnostic::warn(META.name, message, loc.span.start, loc.span.end)
                .with_help(ctx.t("vue/no-use-v-else-with-v-for.help").as_ref()),
        );
    }
}

#[inline]
fn has_directive(element: &ElementNode<'_>, name: &str) -> bool {
    element
        .props
        .iter()
        .any(|prop| matches!(prop, PropNode::Directive(directive) if directive.name == name))
}

/// Relief stores the start tag in `ElementNode::loc`. Recover the closing tag
/// after the complete subtree so the diagnostic matches vue-eslint-parser's
/// whole-element span.
fn full_element_loc(source: &str, element: &ElementNode<'_>) -> SourceLocation {
    let mut loc = element.loc.clone();
    if element.is_self_closing {
        return loc;
    }

    let search_from = element
        .children
        .iter()
        .fold(loc.span.end as usize, |end, child| {
            let child_end = match child {
                TemplateChildNode::Element(child) => {
                    ensure_sufficient_stack(|| full_element_loc(source, child).span.end as usize)
                }
                other => other.loc().span.end as usize,
            };
            end.max(child_end)
        });
    let bytes = source.as_bytes();
    let tag = element.tag.as_bytes();
    let mut cursor = search_from;

    while cursor + tag.len() + 3 <= bytes.len() {
        let Some(relative) = bytes[cursor..].iter().position(|&byte| byte == b'<') else {
            break;
        };
        let start = cursor + relative;
        let name_start = start + 2;
        let name_end = name_start + tag.len();
        if bytes.get(start + 1) == Some(&b'/')
            && bytes
                .get(name_start..name_end)
                .is_some_and(|name| name.eq_ignore_ascii_case(tag))
            && bytes
                .get(name_end)
                .is_some_and(|byte| byte.is_ascii_whitespace() || *byte == b'>')
            && let Some(end_relative) = bytes[name_end..].iter().position(|&byte| byte == b'>')
        {
            loc.span.end = (name_end + end_relative + 1) as u32;
            return loc;
        }
        cursor = start + 1;
    }

    loc
}
