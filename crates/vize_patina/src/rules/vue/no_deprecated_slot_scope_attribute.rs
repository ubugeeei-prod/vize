//! vue/no-deprecated-slot-scope-attribute
//!
//! Disallow the deprecated `slot-scope` attribute (removed in Vue 3).
//!
//! Vue 2.6 deprecated `slot-scope` in favour of `v-slot`, and Vue 3 removed it
//! entirely.
//!
//! This mirrors eslint-plugin-vue's `vue/no-deprecated-slot-scope-attribute`. It
//! only fires for the default Vue 3 dialect.
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
use crate::markup::{MarkupAttribute, MarkupContext, MarkupElement, MarkupRule, MarkupTraversal};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_l0::dialect::VueDialect;
use vize_relief::ElementNode;

static META: RuleMeta = RuleMeta {
    name: "vue/no-deprecated-slot-scope-attribute",
    description: "Disallow the deprecated `slot-scope` attribute",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Error,
};

mod native;

/// Disallow the deprecated `slot-scope` attribute.
pub struct NoDeprecatedSlotScopeAttribute;

impl NoDeprecatedSlotScopeAttribute {
    fn check_attribute(ctx: &mut LintContext<'_>, attribute: &MarkupAttribute<'_>) {
        if ctx.dialect() != VueDialect::Vue || attribute.name() != "slot-scope" {
            return;
        }

        ctx.error_at_with_help(
            ctx.t("vue/no-deprecated-slot-scope-attribute.message"),
            attribute.name_range(),
            ctx.t("vue/no-deprecated-slot-scope-attribute.help"),
        );
    }

    fn check_element(ctx: &mut LintContext<'_>, element: &MarkupElement<'_>) {
        element.walk_attributes(&mut |attribute| {
            Self::check_attribute(ctx, &attribute);
        });
    }
}

impl MarkupRule for NoDeprecatedSlotScopeAttribute {
    fn name(&self) -> &'static str {
        META.name
    }

    fn traversal(&self) -> MarkupTraversal {
        MarkupTraversal::Elements
    }

    fn enter_element<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, element: &MarkupElement<'a>) {
        Self::check_element(ctx.lint(), element);
    }
}

impl Rule for NoDeprecatedSlotScopeAttribute {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn as_native_sfc_template_rule(
        &self,
    ) -> Option<&dyn crate::native::sfc::NativeSfcTemplateRule> {
        Some(self)
    }

    fn as_markup_rule(&self) -> Option<&dyn MarkupRule> {
        Some(self)
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        Self::check_element(ctx, &MarkupElement::new(element));
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
