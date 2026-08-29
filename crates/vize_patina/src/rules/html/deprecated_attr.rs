//! html/deprecated-attr
//!
//! Warn when deprecated HTML attributes are used. Suggests CSS alternatives.
//! Based on markuplint's `deprecated-attr` rule.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <template>
//!   <div align="center">text</div>
//!   <table bgcolor="#fff" cellpadding="5">...</table>
//! </template>
//! ```
//!
//! ### Valid
//! ```vue
//! <template>
//!   <div style="text-align: center">text</div>
//!   <table style="background-color: #fff">...</table>
//! </template>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::markup::{MarkupBinding, MarkupBindingKind, MarkupContext, MarkupElement, MarkupRule};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::{ElementNode, ElementType, PropNode};

use super::helpers::{deprecated_attr_suggestion, deprecated_attr_suggestion_by_tag};

static META: RuleMeta = RuleMeta {
    name: "html/deprecated-attr",
    description: "Disallow deprecated HTML attributes",
    category: RuleCategory::HtmlConformance,
    fixable: false,
    default_severity: Severity::Warning,
};

#[derive(Default)]
pub struct DeprecatedAttr;

impl DeprecatedAttr {
    fn deprecated_markup_attr<'a>(
        element: &MarkupElement<'a>,
        binding: &MarkupBinding<'a>,
    ) -> Option<(&'a str, &'static str)> {
        if binding.kind() != MarkupBindingKind::Attribute {
            return None;
        }

        let name = binding.arg_name()?;
        if !binding.is_static_unqualified_arg_exact(name) {
            return None;
        }

        deprecated_attr_suggestion_by_tag(name, |expected| {
            element.is_unqualified_tag_exact(expected)
        })
        .map(|suggestion| (name, suggestion))
    }
}

/// Markup-IR entry point for `html/deprecated-attr`.
///
/// Mirrors the legacy template visitor exactly: only static, unqualified
/// attributes on non-component elements are checked. Bound attributes
/// (`:align` / `align={...}`), case-different names, and JSX namespaced props
/// stay outside the rule; tag-specific exceptions use exact unqualified tag
/// matching so JSX namespaced tags keep the old lowered-tag behavior.
impl MarkupRule for DeprecatedAttr {
    fn name(&self) -> &'static str {
        META.name
    }

    fn enter_binding<'a>(
        &self,
        ctx: &mut MarkupContext<'_, 'a>,
        element: &MarkupElement<'a>,
        binding: &MarkupBinding<'a>,
    ) {
        if element.is_component() {
            return;
        }

        let tag = element.tag();
        let Some((name, suggestion)) = Self::deprecated_markup_attr(element, binding) else {
            return;
        };

        let message = ctx.lint().t_fmt(
            "html/deprecated-attr.message",
            &[("attr", name), ("tag", tag)],
        );
        let help = ctx
            .lint()
            .t_fmt("html/deprecated-attr.help", &[("suggestion", suggestion)]);
        ctx.lint().warn_at_with_help(message, binding.range(), help);
    }
}

impl Rule for DeprecatedAttr {
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

        let tag = element.tag;

        for prop in &element.props {
            if let PropNode::Attribute(attr) = prop {
                let name = attr.name;
                if let Some(suggestion) = deprecated_attr_suggestion(tag, name) {
                    let message = ctx.t_fmt(
                        "html/deprecated-attr.message",
                        &[("attr", name), ("tag", tag)],
                    );
                    let help =
                        ctx.t_fmt("html/deprecated-attr.help", &[("suggestion", suggestion)]);
                    ctx.warn_with_help(message, &attr.loc, help);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::DeprecatedAttr;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(DeprecatedAttr));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_no_deprecated() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div class="center">text</div>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_table_border() {
        let linter = create_linter();
        // border on table is NOT deprecated
        let result = linter.lint_template(r#"<table border="1"></table>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_align() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div align="center">text</div>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_bgcolor() {
        let linter = create_linter();
        let result = linter.lint_template(r##"<table bgcolor="#fff"></table>"##, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_cellpadding() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<table cellpadding="5"></table>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_td_valign() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<table><tr><td valign="top">text</td></tr></table>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_br_clear() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<br clear="all">"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_multiple() {
        let linter = create_linter();
        let result = linter.lint_template(
            r##"<div align="center" bgcolor="#fff">text</div>"##,
            "test.vue",
        );
        assert_eq!(result.warning_count, 2);
    }
}
