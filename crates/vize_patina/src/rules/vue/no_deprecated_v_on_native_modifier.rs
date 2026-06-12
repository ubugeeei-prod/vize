//! vue/no-deprecated-v-on-native-modifier
//!
//! Disallow the `.native` modifier on `v-on` (deprecated and removed in Vue 3).
//!
//! In Vue 2, `@click.native` was required to listen to a *native* DOM event on
//! the root element of a child component, because component event listeners and
//! native listeners lived in separate channels. Vue 3 removed the `.native`
//! modifier entirely: any listener that is not declared as a component
//! `emits`/prop event falls through to the root element automatically. A
//! lingering `.native` modifier in a Vue 3 template is silently ignored, so
//! the listener may not behave as the author intended.
//!
//! This mirrors eslint-plugin-vue's `vue/no-deprecated-v-on-native-modifier`.
//!
//! ## Dialect gating
//!
//! The rule fires only for the default Vue 3 dialect. petite-vue does not model
//! component native listeners (its `@` always binds a real DOM listener), and
//! the legacy Vue 2 / 2.7 dialect still compiles `.native` behind the `legacy`
//! feature, so neither should be flagged here.
//!
//! ## Examples
//!
//! ### Invalid (Vue 3)
//! ```vue
//! <MyComponent @click.native="handler" />
//! <MyComponent v-on:click.native="handler" />
//! <MyComponent @click.native.stop="handler" />
//! ```
//!
//! ### Valid (Vue 3)
//! ```vue
//! <MyComponent @click="handler" />
//! <MyComponent @click.stop="handler" />
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_carton::dialect::VueDialect;
use vize_relief::{DirectiveNode, ElementNode};

static META: RuleMeta = RuleMeta {
    name: "vue/no-deprecated-v-on-native-modifier",
    description: "Disallow the deprecated `.native` modifier on `v-on`",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Error,
};

/// Disallow the deprecated `.native` modifier on `v-on`.
pub struct NoDeprecatedVOnNativeModifier;

impl Rule for NoDeprecatedVOnNativeModifier {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn check_directive<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        _element: &ElementNode<'a>,
        directive: &DirectiveNode<'a>,
    ) {
        // Only the default Vue 3 dialect removed `.native`. petite-vue and the
        // legacy Vue 2 dialect treat `.native` differently, so leave them alone.
        if ctx.dialect() != VueDialect::Vue {
            return;
        }

        // Only `v-on` / `@` carries the `.native` modifier.
        if directive.name.as_str() != "on" {
            return;
        }

        let Some(native) = directive
            .modifiers
            .iter()
            .find(|m| m.content.as_str() == "native")
        else {
            return;
        };

        // Report on the `.native` modifier itself for a precise underline.
        ctx.error_with_help(
            ctx.t("vue/no-deprecated-v-on-native-modifier.message"),
            &native.loc,
            ctx.t("vue/no-deprecated-v-on-native-modifier.help"),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::NoDeprecatedVOnNativeModifier;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(NoDeprecatedVOnNativeModifier));
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
    fn reports_native_modifier_shorthand() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<MyComponent @click.native="handler" />"#, "App.vue");
        assert_eq!(result.error_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn reports_native_modifier_full_syntax() {
        let linter = create_linter();
        let result =
            linter.lint_template(r#"<MyComponent v-on:click.native="handler" />"#, "App.vue");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn reports_native_combined_with_other_modifiers() {
        let linter = create_linter();
        let result =
            linter.lint_template(r#"<MyComponent @click.native.stop="handler" />"#, "App.vue");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn allows_v_on_without_native_modifier() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<MyComponent @click="handler" @keyup.enter="onEnter" />"#,
            "App.vue",
        );
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn ignores_native_substring_modifier() {
        // A custom modifier merely starting with "native" must not be flagged.
        let linter = create_linter();
        let result =
            linter.lint_template(r#"<MyComponent @click.nativeish="handler" />"#, "App.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn ignores_native_modifier_in_petite_vue() {
        // petite-vue handles `.native` differently; never flag it there.
        let linter = create_linter();
        let result = linter.lint_standalone_html(
            &petite_doc(r#"<MyComponent @click.native="handler"></MyComponent>"#),
            "index.html",
        );
        assert_eq!(result.error_count, 0);
    }
}
