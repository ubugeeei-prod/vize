//! vue/no-custom-modifiers-on-v-model
//!
//! Vue 2 components only accept the native `v-model` modifiers `lazy`,
//! `number`, and `trim`. A custom modifier such as `v-model.foo` is reported
//! on a component. Native elements are left to `vue/valid-v-model`. The check
//! stays opt-in and is not part of the Vue 3 presets.

use super::require_v_for_key::markup_element_is_custom_component;
use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::markup::{MarkupContext, MarkupDirective, MarkupElement, MarkupRule, MarkupTraversal};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::{DirectiveNode, ElementNode};

use vize_l0::{String, cstr};

const RULE: &str = "vue/no-custom-modifiers-on-v-model";

pub(super) fn message(name: &str) -> String {
    cstr!("'v-model' directives don't support the modifier '{name}'.")
}

static META: RuleMeta = RuleMeta {
    name: RULE,
    description: "Disallow custom modifiers on `v-model` used on a component",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Disallow custom modifiers on component `v-model`.
pub struct NoCustomModifiersOnVModel;

fn is_builtin_modifier(name: &str) -> bool {
    matches!(name, "lazy" | "number" | "trim")
}

impl NoCustomModifiersOnVModel {
    fn check(
        ctx: &mut LintContext<'_>,
        element: &MarkupElement<'_>,
        directive: &MarkupDirective<'_>,
    ) {
        if !directive.name_eq("model") || !markup_element_is_custom_component(element) {
            return;
        }
        directive.walk_modifiers(&mut |modifier| {
            if !is_builtin_modifier(modifier) {
                ctx.warn_at(message(modifier), directive.range());
            }
        });
    }
}

impl MarkupRule for NoCustomModifiersOnVModel {
    fn name(&self) -> &'static str {
        META.name
    }

    fn traversal(&self) -> MarkupTraversal {
        MarkupTraversal::Directives
    }

    fn enter_directive<'a>(
        &self,
        ctx: &mut MarkupContext<'_, 'a>,
        element: &MarkupElement<'a>,
        directive: &MarkupDirective<'a>,
    ) {
        Self::check(ctx.lint(), element, directive);
    }
}

impl Rule for NoCustomModifiersOnVModel {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn as_markup_rule(&self) -> Option<&dyn MarkupRule> {
        Some(self)
    }

    fn check_directive<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        element: &ElementNode<'a>,
        directive: &DirectiveNode<'a>,
    ) {
        Self::check(
            ctx,
            &MarkupElement::new(element),
            &MarkupDirective::from_relief(directive),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::{NoCustomModifiersOnVModel, message};
    use crate::linter::Linter;
    use crate::preset::LintPreset;
    use crate::rule::RuleRegistry;
    use vize_l0::String;

    const RULE: &str = "vue/no-custom-modifiers-on-v-model";

    fn lint(source: &str) -> Vec<std::string::String> {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(NoCustomModifiersOnVModel));
        Linter::with_registry(registry)
            .lint_template(source, "test.vue")
            .diagnostics
            .into_iter()
            .map(|diagnostic| diagnostic.message.to_string())
            .collect()
    }

    #[test]
    fn reports_custom_modifiers_on_components() {
        assert_eq!(
            lint(r#"<MyComponent v-model.foo="bar" />"#),
            vec![message("foo")]
        );
        assert_eq!(
            lint(r#"<my-component v-model:value.foo="bar" />"#),
            vec![message("foo")]
        );
        assert_eq!(
            lint(r#"<MyComponent v-model.foo.bar="bar" />"#),
            vec![message("foo"), message("bar")]
        );
        assert_eq!(
            lint(r#"<MyComponent v-model.lazy.foo.trim="bar" />"#),
            vec![message("foo")]
        );
        assert_eq!(
            lint(r#"<div is="foo" v-model.bar="value"></div>"#),
            vec![message("bar")]
        );
        assert_eq!(
            lint(r#"<input :is="Foo" v-model.custom="value" />"#),
            vec![message("custom")]
        );
    }

    #[test]
    fn allows_builtin_modifiers_and_native_elements() {
        assert!(lint(r#"<MyComponent v-model="bar" />"#).is_empty());
        assert!(lint(r#"<MyComponent v-model.lazy="bar" />"#).is_empty());
        assert!(lint(r#"<MyComponent v-model.number.trim="bar" />"#).is_empty());
        assert!(lint(r#"<MyComponent v-model:value.lazy="bar" />"#).is_empty());
        assert!(lint(r#"<input v-model.foo="bar" />"#).is_empty());
        assert!(lint(r#"<input v-model.lazy.number.trim="bar" />"#).is_empty());
        assert!(lint(r#"<textarea v-model.foo="bar"></textarea>"#).is_empty());
        assert!(lint(r#"<div v-model.foo="bar"></div>"#).is_empty());
    }

    #[test]
    fn stays_opt_in_outside_vue3_presets() {
        assert!(!RuleRegistry::with_essential().has_rule(RULE));
        assert!(!RuleRegistry::with_happy_path().has_rule(RULE));
        assert!(RuleRegistry::with_opt_in_rules().has_rule(RULE));

        let result = Linter::with_preset(LintPreset::Incremental)
            .with_enabled_rules(Some(vec![String::from(RULE)]))
            .lint_template(r#"<MyComponent v-model.foo="bar" />"#, "page.vue");
        let reports: Vec<_> = result
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.rule_name == RULE)
            .map(|diagnostic| diagnostic.message.as_str())
            .collect();
        assert_eq!(reports, [message("foo").as_str()]);
    }
}
