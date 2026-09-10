//! a11y/no-redundant-roles
//!
//! Disallow redundant ARIA roles that match the element's implicit role.
//!
//! Some HTML elements have implicit ARIA roles. Adding a role attribute that
//! matches the implicit role is redundant and adds unnecessary noise.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <nav role="navigation">...</nav>
//! <button role="button">...</button>
//! ```
//!
//! ### Valid
//! ```vue
//! <nav>...</nav>
//! <div role="navigation">...</div>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::markup::{MarkupBindingKind, MarkupContext, MarkupElement, MarkupRule};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use lightningcss::declaration::DeclarationBlock;
use lightningcss::properties::list::ListStyleType;
use lightningcss::properties::{Property, PropertyId};
use lightningcss::rules::CssRule as LCssRule;
use lightningcss::selector::{Component, Selector};
use lightningcss::stylesheet::{ParserOptions, StyleSheet};
use vize_relief::{ElementNode, ElementType};
use vize_s0::FxHashSet;

use super::helpers::{
    get_implicit_role, get_implicit_role_by_attr, get_static_attribute_value,
    get_static_or_bound_literal_attribute_value,
};

static META: RuleMeta = RuleMeta {
    name: "a11y/no-redundant-roles",
    description: "Disallow redundant ARIA roles",
    category: RuleCategory::Accessibility,
    fixable: true,
    default_severity: Severity::Warning,
};

/// Disallow redundant ARIA roles
#[derive(Default)]
pub struct NoRedundantRoles;

impl NoRedundantRoles {
    fn first_static_attribute_value<'a>(
        element: &MarkupElement<'a>,
        name: &str,
    ) -> Option<&'a str> {
        let mut found = false;
        let mut value = None;
        element.walk_bindings(&mut |binding| {
            if !found
                && binding.kind() == MarkupBindingKind::Attribute
                && binding.is_static_unqualified_arg_exact(name)
            {
                found = true;
                value = binding.static_value();
            }
        });
        value
    }

    fn markup_implicit_role<'a>(element: &MarkupElement<'a>) -> Option<&'static str> {
        let tag = element.tag();
        if !element.is_unqualified_tag_exact(tag) {
            return None;
        }

        get_implicit_role_by_attr(tag, |name| {
            Self::first_static_attribute_value(element, name)
        })
    }

    fn keeps_markerless_list_role(
        ctx: &LintContext<'_>,
        element: &ElementNode<'_>,
        role: &str,
    ) -> bool {
        if role != "list" || !matches!(element.tag, "ol" | "ul") {
            return false;
        }

        let Some(class) = get_static_or_bound_literal_attribute_value(element, "class") else {
            return false;
        };
        let classes = class
            .split_ascii_whitespace()
            .filter(|class| !class.is_empty())
            .collect::<FxHashSet<_>>();
        let Some(descriptor) = ctx.sfc_descriptor() else {
            return false;
        };

        descriptor.styles.iter().any(|style| {
            style.src.is_none()
                && style
                    .lang
                    .as_deref()
                    .is_none_or(|lang| lang.eq_ignore_ascii_case("css"))
                && style_has_markerless_list_class(style.content.as_ref(), &classes)
        })
    }
}

fn style_has_markerless_list_class(source: &str, classes: &FxHashSet<&str>) -> bool {
    let Ok(sheet) = StyleSheet::parse(source, ParserOptions::default()) else {
        return false;
    };
    sheet
        .rules
        .0
        .iter()
        .any(|rule| css_rule_has_markerless_list_class(rule, classes))
}

fn css_rule_has_markerless_list_class(rule: &LCssRule, classes: &FxHashSet<&str>) -> bool {
    match rule {
        LCssRule::Style(rule) => {
            rule.selectors
                .0
                .iter()
                .any(|selector| selector_matches_class(selector, classes))
                && declarations_have_markerless_list(&rule.declarations)
        }
        LCssRule::Media(rule) => rule
            .rules
            .0
            .iter()
            .any(|rule| css_rule_has_markerless_list_class(rule, classes)),
        LCssRule::Supports(rule) => rule
            .rules
            .0
            .iter()
            .any(|rule| css_rule_has_markerless_list_class(rule, classes)),
        LCssRule::LayerBlock(rule) => rule
            .rules
            .0
            .iter()
            .any(|rule| css_rule_has_markerless_list_class(rule, classes)),
        _ => false,
    }
}

fn selector_matches_class(selector: &Selector, classes: &FxHashSet<&str>) -> bool {
    selector.iter().any(|component| {
        matches!(component, Component::Class(class) if classes.contains(class.0.as_ref()))
    })
}

fn declarations_have_markerless_list(declarations: &DeclarationBlock) -> bool {
    declarations
        .declarations
        .iter()
        .chain(declarations.important_declarations.iter())
        .any(property_has_markerless_list)
}

fn property_has_markerless_list(property: &Property) -> bool {
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

        if implicit != role_value {
            return;
        }

        let message = ctx.lint().t_fmt(
            "a11y/no-redundant-roles.message",
            &[("tag", element.tag()), ("role", role_value)],
        );
        let help = ctx.lint().t("a11y/no-redundant-roles.help");
        ctx.lint().warn_at_with_help(message, element.range(), help);
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
            && !Self::keeps_markerless_list_role(ctx, element, role_value)
        {
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

#[cfg(test)]
mod tests {
    use super::NoRedundantRoles;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(NoRedundantRoles));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_no_role() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<nav>Navigation</nav>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_different_role() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div role="navigation">Navigation</div>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_nav_navigation() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<nav role="navigation">Navigation</nav>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_button_button() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<button role="button">Click</button>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_main_main() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<main role="main">Content</main>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_valid_markerless_list_role_workaround() {
        let linter = create_linter();
        let result = linter.lint_sfc(
            r#"<template>
  <ul class="plain" role="list"><li>a</li></ul>
</template>
<style scoped>
.plain { list-style: none; }
</style>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_list_role_without_markerless_style() {
        let linter = create_linter();
        let result = linter.lint_sfc(
            r#"<template>
  <ul class="plain" role="list"><li>a</li></ul>
</template>
<style scoped>
.plain { list-style: disc; }
</style>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_valid_component_skipped() {
        let linter = create_linter();
        let result =
            linter.lint_template(r#"<MyNav role="navigation">Navigation</MyNav>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }
}
