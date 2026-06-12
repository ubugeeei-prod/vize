//! vue/no-deprecated-v-bind-sync
//!
//! Disallow the `.sync` modifier on `v-bind` (deprecated and removed in Vue 3).
//!
//! In Vue 2, `:title.sync="title"` was sugar for a two-way binding that emitted
//! an `update:title` event back to the parent. Vue 3 removed the `.sync`
//! modifier entirely and folded its behaviour into argumented `v-model`
//! (`v-model:title="title"`). A lingering `.sync` modifier in a Vue 3 template
//! is no longer compiled as a two-way binding, so the prop update never reaches
//! the parent and the binding silently breaks.
//!
//! This mirrors eslint-plugin-vue's `vue/no-deprecated-v-bind-sync`.
//!
//! ## Dialect gating
//!
//! The rule fires only for the default Vue 3 dialect. petite-vue does not model
//! `.sync` two-way bindings at all, and the legacy Vue 2 / 2.7 dialect still
//! compiles `.sync` behind the `legacy` feature, so neither should be flagged
//! here.
//!
//! ## Examples
//!
//! ### Invalid (Vue 3)
//! ```vue
//! <MyComponent :title.sync="title" />
//! <MyComponent v-bind:title.sync="title" />
//! <MyComponent :title.sync.camel="title" />
//! ```
//!
//! ### Valid (Vue 3)
//! ```vue
//! <MyComponent :title="title" />
//! <MyComponent v-model:title="title" />
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_carton::dialect::VueDialect;
use vize_relief::{DirectiveNode, ElementNode};

static META: RuleMeta = RuleMeta {
    name: "vue/no-deprecated-v-bind-sync",
    description: "Disallow the deprecated `.sync` modifier on `v-bind`",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Error,
};

/// Disallow the deprecated `.sync` modifier on `v-bind`.
pub struct NoDeprecatedVBindSync;

impl Rule for NoDeprecatedVBindSync {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn check_directive<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        _element: &ElementNode<'a>,
        directive: &DirectiveNode<'a>,
    ) {
        // Only the default Vue 3 dialect removed `.sync`. petite-vue and the
        // legacy Vue 2 dialect treat `.sync` differently, so leave them alone.
        if ctx.dialect() != VueDialect::Vue {
            return;
        }

        // Only `v-bind` / `:` carries the `.sync` modifier.
        if directive.name.as_str() != "bind" {
            return;
        }

        let Some(sync) = directive
            .modifiers
            .iter()
            .find(|m| m.content.as_str() == "sync")
        else {
            return;
        };

        // Report on the `.sync` modifier itself for a precise underline.
        ctx.error_with_help(
            ctx.t("vue/no-deprecated-v-bind-sync.message"),
            &sync.loc,
            ctx.t("vue/no-deprecated-v-bind-sync.help"),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::NoDeprecatedVBindSync;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(NoDeprecatedVBindSync));
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
    fn reports_sync_modifier_shorthand() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<MyComponent :title.sync="title" />"#, "App.vue");
        assert_eq!(result.error_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn reports_sync_modifier_full_syntax() {
        let linter = create_linter();
        let result =
            linter.lint_template(r#"<MyComponent v-bind:title.sync="title" />"#, "App.vue");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn reports_sync_combined_with_other_modifiers() {
        let linter = create_linter();
        let result =
            linter.lint_template(r#"<MyComponent :title.sync.camel="title" />"#, "App.vue");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn allows_v_bind_without_sync_modifier() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<MyComponent :title="title" :name.camel="name" />"#,
            "App.vue",
        );
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn ignores_sync_substring_modifier() {
        // A custom modifier merely starting with "sync" must not be flagged.
        let linter = create_linter();
        let result = linter.lint_template(r#"<MyComponent :title.synced="title" />"#, "App.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn ignores_sync_modifier_in_petite_vue() {
        // petite-vue handles `.sync` differently; never flag it there.
        let linter = create_linter();
        let result = linter.lint_standalone_html(
            &petite_doc(r#"<MyComponent :title.sync="title"></MyComponent>"#),
            "index.html",
        );
        assert_eq!(result.error_count, 0);
    }
}
