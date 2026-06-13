//! vue/no-deprecated-slot-attribute
//!
//! Disallow the deprecated `slot` attribute (removed in Vue 3).
//!
//! Vue 2.6 deprecated the `slot="name"` attribute in favour of `v-slot`, and
//! Vue 3 removed it entirely. A lingering `slot` attribute is no longer
//! interpreted as a named-slot assignment.
//!
//! This mirrors eslint-plugin-vue's `vue/no-deprecated-slot-attribute`. It is an
//! opt-in migration rule and only fires for the default Vue 3 dialect.
//!
//! ## Examples
//!
//! ### Invalid (Vue 3)
//! ```vue
//! <template>
//!   <Foo>
//!     <template slot="header"><h1>Title</h1></template>
//!   </Foo>
//! </template>
//! ```
//!
//! ### Valid (Vue 3)
//! ```vue
//! <template>
//!   <Foo>
//!     <template v-slot:header><h1>Title</h1></template>
//!   </Foo>
//! </template>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_carton::dialect::VueDialect;
use vize_relief::{ElementNode, PropNode};

static META: RuleMeta = RuleMeta {
    name: "vue/no-deprecated-slot-attribute",
    description: "Disallow the deprecated `slot` attribute",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Error,
};

/// Disallow the deprecated `slot` attribute.
pub struct NoDeprecatedSlotAttribute;

impl Rule for NoDeprecatedSlotAttribute {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        // Only the default Vue 3 dialect removed the `slot` attribute.
        if ctx.dialect() != VueDialect::Vue {
            return;
        }

        for prop in element.props.iter() {
            if let PropNode::Attribute(attr) = prop
                && attr.name == "slot"
            {
                ctx.error_with_help(
                    ctx.t("vue/no-deprecated-slot-attribute.message"),
                    &attr.loc,
                    ctx.t("vue/no-deprecated-slot-attribute.help"),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::NoDeprecatedSlotAttribute;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(NoDeprecatedSlotAttribute));
        Linter::with_registry(registry)
    }

    #[test]
    fn reports_slot_attribute() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<Foo><template slot="header">x</template></Foo>"#,
            "App.vue",
        );
        assert_eq!(result.error_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn allows_v_slot() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<Foo><template v-slot:header>x</template></Foo>"#,
            "App.vue",
        );
        assert_eq!(result.error_count, 0);
    }
}
