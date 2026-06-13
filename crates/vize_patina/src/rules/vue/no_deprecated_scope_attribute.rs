//! vue/no-deprecated-scope-attribute
//!
//! Disallow the deprecated `scope` attribute on `<template>` (removed in Vue 3).
//!
//! Vue 2.5 deprecated the `scope` attribute on `<template>` in favour of
//! `slot-scope` (and later `v-slot`), and Vue 3 removed it entirely.
//!
//! The rule only inspects `<template scope>`: the `scope` attribute is valid
//! HTML on other elements (notably `<th scope="col">`), so flagging it
//! elsewhere would be a false positive.
//!
//! This mirrors eslint-plugin-vue's `vue/no-deprecated-scope-attribute`. It is
//! an opt-in migration rule and only fires for the default Vue 3 dialect.
//!
//! ## Examples
//!
//! ### Invalid (Vue 3)
//! ```vue
//! <Foo>
//!   <template scope="props">{{ props.msg }}</template>
//! </Foo>
//! ```
//!
//! ### Valid (Vue 3)
//! ```vue
//! <Foo>
//!   <template v-slot="props">{{ props.msg }}</template>
//! </Foo>
//! <table><th scope="col">Name</th></table>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_carton::dialect::VueDialect;
use vize_relief::{ElementNode, PropNode};

static META: RuleMeta = RuleMeta {
    name: "vue/no-deprecated-scope-attribute",
    description: "Disallow the deprecated `scope` attribute on <template>",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Error,
};

/// Disallow the deprecated `scope` attribute on `<template>`.
pub struct NoDeprecatedScopeAttribute;

impl Rule for NoDeprecatedScopeAttribute {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        if ctx.dialect() != VueDialect::Vue {
            return;
        }

        // `scope` is only the deprecated slot syntax on `<template>`; on other
        // elements (e.g. `<th scope="col">`) it is valid HTML.
        if element.tag != "template" {
            return;
        }

        for prop in element.props.iter() {
            if let PropNode::Attribute(attr) = prop
                && attr.name == "scope"
            {
                ctx.error_with_help(
                    ctx.t("vue/no-deprecated-scope-attribute.message"),
                    &attr.loc,
                    ctx.t("vue/no-deprecated-scope-attribute.help"),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::NoDeprecatedScopeAttribute;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(NoDeprecatedScopeAttribute));
        Linter::with_registry(registry)
    }

    #[test]
    fn reports_scope_attribute_on_template() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<Foo><template scope="props">x</template></Foo>"#,
            "App.vue",
        );
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn allows_scope_on_th() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<table><th scope="col">Name</th></table>"#, "App.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn allows_v_slot() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<Foo><template v-slot="props">x</template></Foo>"#,
            "App.vue",
        );
        assert_eq!(result.error_count, 0);
    }
}
