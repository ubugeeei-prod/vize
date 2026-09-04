//! vue/no-deprecated-inline-template
//!
//! Disallow the `inline-template` attribute (removed in Vue 3).
//!
//! Vue 2 let a component's own template be written inside its parent by adding
//! `inline-template` to the component usage. Vue 3 removed that mode; the child
//! component now renders its own template and the nested markup is ordinary slot
//! content. A lingering `inline-template` therefore silently changes component
//! boundaries during migration.
//!
//! This mirrors eslint-plugin-vue's `vue/no-deprecated-inline-template`. It is
//! an opt-in migration rule and only fires for the default Vue 3 dialect.

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::{ElementNode, PropNode};
use vize_s0::dialect::VueDialect;

static META: RuleMeta = RuleMeta {
    name: "vue/no-deprecated-inline-template",
    description: "Disallow the deprecated `inline-template` attribute",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Error,
};

/// Disallow the deprecated `inline-template` attribute.
pub struct NoDeprecatedInlineTemplate;

impl Rule for NoDeprecatedInlineTemplate {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        // Only the default Vue 3 dialect removed inline-template. Non-Vue
        // dialects use their own compatibility rules.
        if ctx.dialect() != VueDialect::Vue {
            return;
        }

        for prop in element.props.iter() {
            if let PropNode::Attribute(attr) = prop
                && attr.name.eq_ignore_ascii_case("inline-template")
            {
                ctx.error_with_help(
                    ctx.t("vue/no-deprecated-inline-template.message"),
                    &attr.loc,
                    ctx.t("vue/no-deprecated-inline-template.help"),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::NoDeprecatedInlineTemplate;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(NoDeprecatedInlineTemplate));
        Linter::with_registry(registry)
    }

    /// Wrap markup in a petite-vue document so `ctx.dialect()` resolves to
    /// petite-vue and the rule gates itself off.
    fn petite_doc(markup: &str) -> String {
        format!(
            r#"<!DOCTYPE html>
<html>
  <body>
    <div v-scope="{{ count: 0 }}">
{markup}
    </div>
    <script src="https://unpkg.com/petite-vue" init></script>
  </body>
</html>"#
        )
    }

    #[test]
    fn reports_inline_template_on_component() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<MyComponent inline-template><div>Content</div></MyComponent>"#,
            "App.vue",
        );
        assert_eq!(result.error_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn reports_inline_template_on_kebab_component() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<my-component inline-template><div>Content</div></my-component>"#,
            "App.vue",
        );
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn reports_inline_template_on_native_element() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div inline-template>Content</div>"#, "App.vue");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn allows_slot_content() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<MyComponent><template #default><div>Content</div></template></MyComponent>"#,
            "App.vue",
        );
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn allows_bound_attribute_with_same_name() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<MyComponent :inline-template="value" />"#, "App.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn allows_regular_component() {
        let linter = create_linter();
        let result =
            linter.lint_template(r#"<MyComponent :prop="value"></MyComponent>"#, "App.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn ignores_petite_vue_dialect() {
        let linter = create_linter();
        let source = petite_doc(r#"<my-component inline-template></my-component>"#);
        let result = linter.lint_standalone_html(&source, "index.html");
        assert_eq!(result.error_count, 0);
    }
}
