use super::{
    ElementNode, ElementType, Fix, LintContext, LintDiagnostic, ListStyleType, META, MarkupContext,
    MarkupElement, MarkupRule, NoRedundantRoles, PropNode, Property, PropertyId, Rule, RuleMeta,
    TextEdit, get_implicit_role, get_static_attribute_value,
    get_static_or_bound_literal_attribute_value,
};

fn relief_role_attribute_span(element: &ElementNode<'_>) -> Option<(u32, u32)> {
    for prop in &element.props {
        let PropNode::Attribute(attr) = prop else {
            continue;
        };
        if attr.name != "role" {
            continue;
        }
        return attr
            .value
            .as_ref()
            .map(|_| (attr.loc.span.start, attr.loc.span.end));
    }
    None
}

fn report_redundant_role(
    ctx: &mut LintContext<'_>,
    tag: &str,
    role: &str,
    diagnostic_start: u32,
    diagnostic_end: u32,
    attribute_start: u32,
    attribute_end: u32,
) {
    let message = ctx.t_fmt(
        "a11y/no-redundant-roles.message",
        &[("tag", tag), ("role", role)],
    );
    let help = ctx.t("a11y/no-redundant-roles.help");
    let mut diagnostic =
        LintDiagnostic::warn(ctx.current_rule, message, diagnostic_start, diagnostic_end);
    if let Some(processed) = ctx.help_level.process(help.as_ref()) {
        diagnostic = diagnostic.with_help(processed);
    }
    diagnostic = diagnostic.with_fix(Fix::new(
        help.as_ref(),
        remove_attribute_edit(ctx.source, attribute_start, attribute_end),
    ));
    ctx.report(diagnostic);
}

/// Delete the role attribute, including the horizontal whitespace before it.
fn remove_attribute_edit(source: &str, start: u32, end: u32) -> TextEdit {
    let bytes = source.as_bytes();
    let end = (end as usize).min(bytes.len());
    let mut start = (start as usize).min(end);
    while start > 0
        && bytes
            .get(start - 1)
            .is_some_and(|byte| matches!(*byte, b' ' | b'\t'))
    {
        start -= 1;
    }
    TextEdit::delete(start as u32, end as u32)
}

pub(super) fn property_has_markerless_list(property: &Property) -> bool {
    matches!(property, Property::ListStyleType(ListStyleType::None))
        || property
            .longhand(&PropertyId::ListStyleType)
            .is_some_and(|longhand| {
                matches!(longhand, Property::ListStyleType(ListStyleType::None))
            })
}

/// Markup-IR entry point for `a11y/no-redundant-roles`.
///
/// The role table mirrors the legacy `ElementNode` helper, including its
/// static-attribute-only `href` / `type` / `alt` probes and first-attribute
/// behavior. Exact unqualified tag / attribute checks keep direct JSX/TSX
/// projection inside the same visible boundary the old lowering fallback had.
impl MarkupRule for NoRedundantRoles {
    fn name(&self) -> &'static str {
        META.name
    }

    fn enter_element<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, element: &MarkupElement<'a>) {
        if element.is_component() {
            return;
        }

        let Some(role_value) = Self::first_static_attribute_value(element, "role") else {
            return;
        };

        let Some(implicit) = Self::markup_implicit_role(element) else {
            return;
        };

        if implicit != role_value
            || Self::keeps_markerless_list_role(
                ctx.lint(),
                element.tag(),
                super::super::markup_helpers::get_static_or_bound_literal_markup_value(
                    element, "class",
                ),
                role_value,
            )
        {
            return;
        }

        let diagnostic_range = element.range();
        let Some(attribute_range) = Self::redundant_role_attribute_range(element) else {
            let message = ctx.lint().t_fmt(
                "a11y/no-redundant-roles.message",
                &[("tag", element.tag()), ("role", role_value)],
            );
            let help = ctx.lint().t("a11y/no-redundant-roles.help");
            ctx.lint()
                .warn_at_with_help(message, diagnostic_range, help);
            return;
        };
        report_redundant_role(
            ctx.lint(),
            element.tag(),
            role_value,
            diagnostic_range.start,
            diagnostic_range.end,
            attribute_range.start,
            attribute_range.end,
        );
    }
}

impl Rule for NoRedundantRoles {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn as_markup_rule(&self) -> Option<&dyn MarkupRule> {
        Some(self)
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        if element.tag_type == ElementType::Component {
            return;
        }

        let role_value = match get_static_attribute_value(element, "role") {
            Some(r) => r,
            None => return,
        };

        let implicit_role = get_implicit_role(element.tag, element);

        if let Some(implicit) = implicit_role
            && implicit == role_value
            && !Self::keeps_markerless_list_role(
                ctx,
                element.tag,
                get_static_or_bound_literal_attribute_value(element, "class"),
                role_value,
            )
        {
            if let Some((attribute_start, attribute_end)) = relief_role_attribute_span(element) {
                report_redundant_role(
                    ctx,
                    element.tag,
                    role_value,
                    element.loc.span.start,
                    element.loc.span.end,
                    attribute_start,
                    attribute_end,
                );
            } else {
                ctx.warn_with_help(
                    ctx.t_fmt(
                        "a11y/no-redundant-roles.message",
                        &[("tag", element.tag), ("role", role_value)],
                    ),
                    &element.loc,
                    ctx.t("a11y/no-redundant-roles.help"),
                );
            }
        }
    }
}
