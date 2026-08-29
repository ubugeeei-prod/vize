//! html/no-duplicate-dt
//!
//! Detect duplicate `<dt>` term definitions within a `<dl>` element.
//! Based on markuplint's `no-duplicate-dt` rule.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <template>
//!   <dl>
//!     <dt>Term A</dt>
//!     <dd>Definition 1</dd>
//!     <dt>Term A</dt>
//!     <dd>Definition 2</dd>
//!   </dl>
//! </template>
//! ```
//!
//! ### Valid
//! ```vue
//! <template>
//!   <dl>
//!     <dt>Term A</dt>
//!     <dd>Definition A</dd>
//!     <dt>Term B</dt>
//!     <dd>Definition B</dd>
//!   </dl>
//! </template>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::markup::{MarkupContext, MarkupElement, MarkupNode, MarkupRule};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::ElementNode;
use vize_s0::FxHashMap;
use vize_s0::String;
use vize_s0::ToCompactString;

static META: RuleMeta = RuleMeta {
    name: "html/no-duplicate-dt",
    description: "Disallow duplicate <dt> names in <dl>",
    category: RuleCategory::HtmlConformance,
    fixable: false,
    default_severity: Severity::Warning,
};

#[derive(Default)]
pub struct NoDuplicateDt;

impl NoDuplicateDt {
    fn check_element(ctx: &mut LintContext<'_>, element: &MarkupElement<'_>) {
        if element.is_component() || !element.is_unqualified_tag_exact("dl") {
            return;
        }

        let mut seen: FxHashMap<String, u32> = FxHashMap::default();

        element.walk_children(&mut |child| {
            if let MarkupNode::Element(el) = child
                && el.is_unqualified_tag_exact("dt")
            {
                let text = el.direct_text_content();
                let normalized = text.trim().to_compact_string();
                if normalized.is_empty() {
                    return;
                }

                if let std::collections::hash_map::Entry::Vacant(entry) =
                    seen.entry(normalized.clone())
                {
                    entry.insert(el.range().start);
                } else {
                    let message = ctx.t_fmt(
                        "html/no-duplicate-dt.message",
                        &[("term", normalized.as_str())],
                    );
                    let help = ctx.t("html/no-duplicate-dt.help");
                    ctx.warn_at_with_help(message, el.range(), help);
                }
            }
        });
    }
}

impl MarkupRule for NoDuplicateDt {
    fn name(&self) -> &'static str {
        META.name
    }

    fn enter_element<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, element: &MarkupElement<'a>) {
        Self::check_element(ctx.lint(), element);
    }
}

impl Rule for NoDuplicateDt {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn as_markup_rule(&self) -> Option<&dyn MarkupRule> {
        Some(self)
    }

    fn jsx_needs_lowering(&self) -> bool {
        true
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        Self::check_element(ctx, &MarkupElement::new(element));
    }
}

#[cfg(test)]
mod tests {
    use super::NoDuplicateDt;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(NoDuplicateDt));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_unique_dt() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<dl><dt>A</dt><dd>def A</dd><dt>B</dt><dd>def B</dd></dl>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_no_dt() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<dl></dl>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_not_dl() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div>text</div>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_duplicate_dt() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<dl><dt>A</dt><dd>def 1</dd><dt>A</dt><dd>def 2</dd></dl>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_triple_duplicate() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<dl><dt>X</dt><dd>1</dd><dt>X</dt><dd>2</dd><dt>X</dt><dd>3</dd></dl>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 2);
    }
}
