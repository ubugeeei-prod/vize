//! vue/valid-v-pre
//!
//! Enforce valid `v-pre` directives.
//!
//! `v-pre` is a bare marker directive: it must not have an argument, a
//! modifier, or a value.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <div v-pre:arg></div>
//! <div v-pre.mod></div>
//! <div v-pre="foo"></div>
//! ```
//!
//! ### Valid
//! ```vue
//! <div v-pre>{{ raw }}</div>
//! ```

mod open_tag;

use crate::context::LintContext;
use crate::diagnostic::{LintDiagnostic, Severity};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use open_tag::{AuthoredAttribute, authored_attributes};
use vize_relief::{ElementNode, RootNode, TemplateChildNode};

static META: RuleMeta = RuleMeta {
    name: "vue/valid-v-pre",
    description: "Enforce valid `v-pre` directives",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Error,
};

/// Enforce valid v-pre directives
pub struct ValidVPre;

impl Rule for ValidVPre {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    // The parser consumes `v-pre` while it freezes the element, so the
    // directive never reaches `check_directive`. Read it from the authored
    // opening tag instead, and stop at the first `v-pre`: inside the raw
    // subtree the same spelling is an ordinary attribute.
    fn run_on_template<'a>(&self, ctx: &mut LintContext<'a>, root: &RootNode<'a>) {
        if ctx.source.contains("v-pre") {
            check_children(ctx, &root.children);
        }
    }
}

fn check_children(ctx: &mut LintContext, children: &[TemplateChildNode]) {
    for child in children {
        if let TemplateChildNode::Element(element) = child
            && !check_opening_tag(ctx, element)
        {
            check_children(ctx, &element.children);
        }
    }
}

/// Report an invalid `v-pre` on `element`; returns whether it carries one.
fn check_opening_tag(ctx: &mut LintContext, element: &ElementNode) -> bool {
    let span = &element.loc.span;
    let Some(tag_source) = ctx.source.get(span.start as usize..span.end as usize) else {
        return false;
    };
    if !tag_source.contains("v-pre") {
        return false;
    }
    let Some(v_pre) = authored_attributes(tag_source).find_map(VPre::parse) else {
        return false;
    };

    let mut report = |key: &str, range: (usize, usize)| {
        let (Ok(start), Ok(end)) = (u32::try_from(range.0), u32::try_from(range.1)) else {
            return;
        };
        ctx.report(
            LintDiagnostic::warn(
                META.name,
                ctx.t(key).as_ref(),
                span.start.saturating_add(start),
                span.start.saturating_add(end),
            )
            .with_help(ctx.t("vue/valid-v-pre.help").as_ref()),
        );
    };
    if let Some(argument) = v_pre.argument {
        report("vue/valid-v-pre.unexpected_argument", argument);
    }
    if let Some(modifiers) = v_pre.modifiers {
        report("vue/valid-v-pre.unexpected_modifier", modifiers);
    }
    if let Some(value) = v_pre.value {
        report("vue/valid-v-pre.unexpected_value", value);
    }
    true
}

/// The pieces of one authored `v-pre` attribute, as byte ranges in the tag.
struct VPre {
    argument: Option<(usize, usize)>,
    modifiers: Option<(usize, usize)>,
    value: Option<(usize, usize)>,
}

impl VPre {
    fn parse(attribute: AuthoredAttribute<'_>) -> Option<Self> {
        let rest = attribute.name.strip_prefix("v-pre")?;
        if !(rest.is_empty() || rest.starts_with([':', '.'])) {
            return None;
        }
        let rest_start = attribute.name_start + "v-pre".len();
        let (argument, modifiers_at) = match rest.strip_prefix(':') {
            Some(after_colon) => {
                let len = after_colon.find('.').unwrap_or(after_colon.len());
                let start = rest_start + 1;
                (Some((start, start + len)), start + len)
            }
            None => (None, rest_start),
        };
        let name_end = attribute.name_start + attribute.name.len();
        // Skip the dot that introduces the first modifier.
        let modifiers = (modifiers_at < name_end).then_some((modifiers_at + 1, name_end));
        Some(Self {
            argument: argument.filter(|(start, end)| start < end),
            modifiers: modifiers.filter(|(start, end)| start < end),
            value: attribute.value,
        })
    }
}

#[cfg(test)]
mod tests;
