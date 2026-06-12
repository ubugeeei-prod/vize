//! a11y/img-alt
//!
//! Require alt attribute on <img> elements for accessibility.
//!
//! Images must have an alt attribute for screen readers.
//! Decorative images should have an empty alt attribute.
//!
//! Based on eslint-plugin-vuejs-accessibility alt-text rule.

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::markup::{MarkupBindingKind, MarkupContext, MarkupElement, MarkupRule};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::ElementNode;

static META: RuleMeta = RuleMeta {
    name: "a11y/img-alt",
    description: "Require alt attribute on images for accessibility",
    category: RuleCategory::Accessibility,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Require alt attribute on images
#[derive(Default)]
pub struct ImgAlt;

/// Markup-IR entry point for `a11y/img-alt`.
///
/// An HTML-shaped rule: it only inspects the tag name and whether *some* `alt`
/// binding exists (static `alt="…"`, dynamic `:alt`, or JSX `alt={…}`). Because
/// the [`MarkupElement`] facade answers those questions over both backends, the
/// same rule runs unchanged on a Vue `<img>` and a JSX `<img />` projected
/// directly from the OXC AST — no synthetic template AST in between.
impl MarkupRule for ImgAlt {
    fn name(&self) -> &'static str {
        META.name
    }

    fn enter_element<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, element: &MarkupElement<'a>) {
        if !element.is_tag("img") {
            return;
        }

        // An `alt` exists if there is any binding whose argument is `alt`,
        // whether it is a plain attribute (`alt="x"`), a `v-bind`/`:alt`, or a
        // JSX `alt={…}`.
        let mut has_alt = false;
        element.walk_bindings(&mut |binding| {
            if matches!(
                binding.kind(),
                MarkupBindingKind::Attribute | MarkupBindingKind::Bind
            ) && binding.arg_name_eq("alt")
            {
                has_alt = true;
            }
        });

        if !has_alt {
            let message = ctx.lint().t("a11y/img-alt.message");
            let help = ctx.lint().t("a11y/img-alt.help");
            ctx.lint().warn_at_with_help(message, element.range(), help);
        }
    }
}

impl Rule for ImgAlt {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn as_markup_rule(&self) -> Option<&dyn MarkupRule> {
        Some(self)
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        if element.tag != "img" {
            return;
        }

        // Check for alt attribute (static or dynamic)
        let has_alt = element.props.iter().any(|prop| match prop {
            vize_relief::PropNode::Attribute(attr) => attr.name == "alt",
            vize_relief::PropNode::Directive(dir) => {
                if dir.name == "bind" {
                    matches!(
                        &dir.arg,
                        Some(vize_relief::ExpressionNode::Simple(s)) if s.content == "alt"
                    )
                } else {
                    false
                }
            }
        });

        if !has_alt {
            ctx.warn_with_help(
                ctx.t("a11y/img-alt.message"),
                &element.loc,
                ctx.t("a11y/img-alt.help"),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ImgAlt;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(ImgAlt));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_with_alt() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<img src="/photo.jpg" alt="Photo" />"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_with_empty_alt() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<img src="/decoration.svg" alt="" />"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_no_alt() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<img src="/photo.jpg" />"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }
}
