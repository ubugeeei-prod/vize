//! vapor/no-inline-template
//!
//! Disallow inline-template attribute (deprecated in Vue 3).
//!
//! The inline-template attribute was deprecated in Vue 3 and is not
//! supported in Vapor mode. Use slots or separate component files instead.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <my-component inline-template>
//!   <div>Content</div>
//! </my-component>
//! ```
//!
//! ### Valid
//! ```vue
//! <my-component>
//!   <template #default>
//!     <div>Content</div>
//!   </template>
//! </my-component>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::ast::{ElementNode, PropNode};

static META: RuleMeta = RuleMeta {
    name: "vapor/no-inline-template",
    description: "Disallow deprecated inline-template attribute",
    category: RuleCategory::Vapor,
    fixable: false,
    default_severity: Severity::Error,
};

/// Disallow inline-template
pub struct NoInlineTemplate;

impl Rule for NoInlineTemplate {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        for prop in element.props.iter() {
            if let PropNode::Attribute(attr) = prop {
                if attr.name.as_str().eq_ignore_ascii_case("inline-template") {
                    ctx.error_with_help(
                        ctx.t("vapor/no-inline-template.message"),
                        &attr.loc,
                        ctx.t("vapor/no-inline-template.help"),
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::NoInlineTemplate;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(NoInlineTemplate));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_invalid_inline_template() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<MyComponent inline-template><div>Content</div></MyComponent>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 1);
        assert!(result.diagnostics[0].message.contains("inline-template"));
    }

    #[test]
    fn test_valid_slot() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<MyComponent><template #default><div>Content</div></template></MyComponent>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_regular_component() {
        let linter = create_linter();
        let result =
            linter.lint_template(r#"<MyComponent :prop="value"></MyComponent>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }
}
