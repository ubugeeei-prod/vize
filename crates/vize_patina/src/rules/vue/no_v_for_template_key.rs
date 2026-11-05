//! vue/no-v-for-template-key
//!
//! Disallow `key` on `<template v-for>`.
//!
//! Vue 2 ignores a key on `<template v-for>` and expects it on a real child.
//! Vue 3 requires the key on the `<template>`, so this check stays opt-in and
//! is not part of the Vue 3 presets. `vue/no-template-key` still rejects a key
//! on a `<template>` that has no `v-for`.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <template v-for="item in items" :key="item.id">
//!   <div />
//! </template>
//! ```
//!
//! ### Valid
//! ```vue
//! <template v-for="item in items">
//!   <div :key="item.id" />
//! </template>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::{ElementNode, ExpressionNode, PropNode};

const MESSAGE: &str = "'<template v-for>' cannot be keyed. Place the key on real elements instead.";

static META: RuleMeta = RuleMeta {
    name: "vue/no-v-for-template-key",
    description: "Disallow `key` on `<template v-for>`",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Error,
};

/// Disallow `key` on `<template v-for>`.
pub struct NoVForTemplateKey;

fn has_v_for(element: &ElementNode<'_>) -> bool {
    element
        .props
        .iter()
        .any(|prop| matches!(prop, PropNode::Directive(directive) if directive.name == "for"))
}

fn is_key(prop: &PropNode<'_>) -> bool {
    match prop {
        PropNode::Attribute(attribute) => attribute.name == "key",
        PropNode::Directive(directive) => {
            directive.name == "bind"
                && matches!(
                    directive.arg.as_ref(),
                    Some(ExpressionNode::Simple(argument)) if argument.content == "key"
                )
        }
    }
}

impl Rule for NoVForTemplateKey {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        if element.tag != "template" || !has_v_for(element) {
            return;
        }
        if let Some(prop) = element.props.iter().find(|prop| is_key(prop)) {
            ctx.error(MESSAGE, prop.loc());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{MESSAGE, NoVForTemplateKey};
    use crate::linter::Linter;
    use crate::preset::LintPreset;
    use crate::rule::RuleRegistry;
    use vize_l0::String;

    const RULE: &str = "vue/no-v-for-template-key";

    fn lint(source: &str) -> Vec<std::string::String> {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(NoVForTemplateKey));
        Linter::with_registry(registry)
            .lint_template(source, "test.vue")
            .diagnostics
            .into_iter()
            .map(|diagnostic| format!("{}: {}", diagnostic.rule_name, diagnostic.message))
            .collect()
    }

    #[test]
    fn reports_bound_and_static_keys_on_template_v_for() {
        let expected = vec![format!("{RULE}: {MESSAGE}")];
        assert_eq!(
            lint(r#"<template v-for="item in items" :key="item.id"><div /></template>"#),
            expected
        );
        assert_eq!(
            lint(r#"<template v-for="item in items" v-bind:key="item.id"><div /></template>"#),
            expected
        );
        assert_eq!(
            lint(r#"<template v-for="item in items" key="id"><div /></template>"#),
            expected
        );
    }

    #[test]
    fn allows_the_key_on_a_real_child_and_on_element_v_for() {
        assert!(
            lint(r#"<template v-for="item in items"><div :key="item.id" /></template>"#).is_empty()
        );
        assert!(lint(r#"<div v-for="item in items" :key="item.id" />"#).is_empty());
        assert!(lint(r#"<template v-for="item in items"><div /></template>"#).is_empty());
        assert!(lint(r#"<template :key="section"><div /></template>"#).is_empty());
    }

    #[test]
    fn stays_opt_in_outside_vue3_presets() {
        assert!(!RuleRegistry::with_essential().has_rule(RULE));
        assert!(!RuleRegistry::with_happy_path().has_rule(RULE));
        assert!(RuleRegistry::with_opt_in_rules().has_rule(RULE));

        let result = Linter::with_preset(LintPreset::Incremental)
            .with_enabled_rules(Some(vec![String::from(RULE)]))
            .lint_template(
                r#"<template v-for="item in items" :key="item.id"><div /></template>"#,
                "page.vue",
            );
        let reports: Vec<_> = result
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.rule_name == RULE)
            .map(|diagnostic| diagnostic.message.as_str())
            .collect();
        assert_eq!(reports, [MESSAGE]);
    }
}
