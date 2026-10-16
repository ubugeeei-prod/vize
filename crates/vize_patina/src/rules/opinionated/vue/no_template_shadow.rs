//! vue/no-template-shadow
//!
//! Disallow template declarations that shadow another template binding.
//!
//! v-for and modern/legacy scoped-slot declarations use their authored identifier
//! ranges and lexical template scopes. Script/component exposure is separate.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <!-- Nested v-for with same variable name -->
//! <div v-for="item in items">
//!   <div v-for="item in item.children">{{ item }}</div>
//! </div>
//! ```
//!
//! ### Valid
//! ```vue
//! <div v-for="item in items">
//!   <div v-for="child in item.children">{{ child }}</div>
//! </div>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::RootNode;

mod bindings;
mod frozen;
mod scopes;

static META: RuleMeta = RuleMeta {
    name: "vue/no-template-shadow",
    description: "Disallow variable names that shadow variables in outer scope",
    category: RuleCategory::StronglyRecommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Disallow template variable shadowing
pub struct NoTemplateShadow;

impl Rule for NoTemplateShadow {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_template<'a>(&self, ctx: &mut LintContext<'a>, root: &RootNode<'a>) {
        scopes::check(ctx, &root.children);
    }
}

#[cfg(test)]
mod tests {
    use super::NoTemplateShadow;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(NoTemplateShadow));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_no_shadow() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div v-for="item in items" :key="item.id"><span v-for="child in item.children" :key="child.id">{{ child }}</span></div>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_nested_shadow() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div v-for="item in items" :key="item.id"><span v-for="item in item.children" :key="item.id">{{ item }}</span></div>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn test_valid_different_names() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div v-for="(item, index) in items" :key="index"><span v-for="(child, childIndex) in item.children" :key="childIndex">{{ child }}</span></div>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }
}
