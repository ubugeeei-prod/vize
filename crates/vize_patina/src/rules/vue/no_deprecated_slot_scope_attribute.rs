//! vue/no-deprecated-slot-scope-attribute
//!
//! Disallow the deprecated `slot-scope` attribute (removed in Vue 3).
//!
//! Vue 2.6 deprecated `slot-scope` in favour of `v-slot`, and Vue 3 removed it
//! entirely.
//!
//! This mirrors eslint-plugin-vue's `vue/no-deprecated-slot-scope-attribute`. It
//! is an opt-in migration rule and only fires for the default Vue 3 dialect.
//!
//! ## Examples
//!
//! ### Invalid (Vue 3)
//! ```vue
//! <Foo>
//!   <template slot-scope="props">{{ props.msg }}</template>
//! </Foo>
//! ```
//!
//! ### Valid (Vue 3)
//! ```vue
//! <Foo>
//!   <template v-slot="props">{{ props.msg }}</template>
//! </Foo>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_carton::dialect::VueDialect;
use vize_relief::{ElementNode, PropNode};

static META: RuleMeta = RuleMeta {
    name: "vue/no-deprecated-slot-scope-attribute",
    description: "Disallow the deprecated `slot-scope` attribute",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Error,
};

/// Disallow the deprecated `slot-scope` attribute.
pub struct NoDeprecatedSlotScopeAttribute;

impl Rule for NoDeprecatedSlotScopeAttribute {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        if ctx.dialect() != VueDialect::Vue {
            return;
        }

        for prop in element.props.iter() {
            if let PropNode::Attribute(attr) = prop
                && attr.name == "slot-scope"
            {
                ctx.error_with_help(
                    ctx.t("vue/no-deprecated-slot-scope-attribute.message"),
                    &attr.loc,
                    ctx.t("vue/no-deprecated-slot-scope-attribute.help"),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::NoDeprecatedSlotScopeAttribute;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(NoDeprecatedSlotScopeAttribute));
        Linter::with_registry(registry)
    }

    #[test]
    fn reports_slot_scope_attribute() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<Foo><template slot-scope="props">x</template></Foo>"#,
            "App.vue",
        );
        assert_eq!(result.error_count, 1);
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
