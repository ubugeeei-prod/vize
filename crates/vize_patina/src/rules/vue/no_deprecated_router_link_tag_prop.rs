//! vue/no-deprecated-router-link-tag-prop
//!
//! Disallow the `tag` prop on `<router-link>` (removed in Vue Router 4).
//!
//! Vue Router 4 removed the `tag` prop (and the related `event` prop) in favour
//! of the scoped-slot (`v-slot`) API, which exposes `href`, `navigate`, and
//! `isActive` so the caller renders any element they like.
//!
//! The rule flags both a static `tag` attribute and a `v-bind:tag` binding on
//! the `<router-link>` / `<RouterLink>` element. It is an opt-in migration rule
//! and only fires for the default Vue 3 dialect.
//!
//! ## Examples
//!
//! ### Invalid (Vue 3)
//! ```vue
//! <template>
//!   <router-link to="/home" tag="button">Home</router-link>
//! </template>
//! ```
//!
//! ### Valid (Vue 3)
//! ```vue
//! <template>
//!   <router-link to="/home" v-slot="{ navigate }">
//!     <button @click="navigate">Home</button>
//!   </router-link>
//! </template>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::markup::{MarkupContext, MarkupDirective, MarkupElement, MarkupRule};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_l0::dialect::VueDialect;
use vize_relief::{DirectiveNode, ElementNode};

static META: RuleMeta = RuleMeta {
    name: "vue/no-deprecated-router-link-tag-prop",
    description: "Disallow the `tag` prop on <router-link>",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Error,
};

/// Disallow the `tag` prop on `<router-link>`.
pub struct NoDeprecatedRouterLinkTagProp;

impl NoDeprecatedRouterLinkTagProp {
    fn is_router_link(ctx: &LintContext<'_>, element: &MarkupElement<'_>) -> bool {
        ctx.dialect() == VueDialect::Vue
            && (element.is_unqualified_tag_exact("router-link")
                || element.is_unqualified_tag_exact("RouterLink"))
    }

    fn report(ctx: &mut LintContext<'_>, range: crate::ir::ByteRange) {
        ctx.error_at_with_help(
            ctx.t("vue/no-deprecated-router-link-tag-prop.message"),
            range,
            ctx.t("vue/no-deprecated-router-link-tag-prop.help"),
        );
    }

    fn check_element(ctx: &mut LintContext<'_>, element: &MarkupElement<'_>) {
        if !Self::is_router_link(ctx, element) {
            return;
        }

        element.walk_attributes(&mut |attribute| {
            if attribute.name() == "tag" {
                Self::report(ctx, attribute.name_range());
            }
        });
    }

    fn check_directive(
        ctx: &mut LintContext<'_>,
        element: &MarkupElement<'_>,
        directive: &MarkupDirective<'_>,
    ) {
        if !Self::is_router_link(ctx, element)
            || !directive.name_eq("bind")
            || directive.arg_name() != Some("tag")
        {
            return;
        }
        if let Some(range) = directive.argument_range() {
            Self::report(ctx, range);
        }
    }
}

impl MarkupRule for NoDeprecatedRouterLinkTagProp {
    fn name(&self) -> &'static str {
        META.name
    }

    fn enter_element<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, element: &MarkupElement<'a>) {
        Self::check_element(ctx.lint(), element);
    }

    fn enter_directive<'a>(
        &self,
        ctx: &mut MarkupContext<'_, 'a>,
        element: &MarkupElement<'a>,
        directive: &MarkupDirective<'a>,
    ) {
        Self::check_directive(ctx.lint(), element, directive);
    }
}

impl Rule for NoDeprecatedRouterLinkTagProp {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn as_markup_rule(&self) -> Option<&dyn MarkupRule> {
        Some(self)
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        Self::check_element(ctx, &MarkupElement::new(element));
    }

    fn check_directive<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        element: &ElementNode<'a>,
        directive: &DirectiveNode<'a>,
    ) {
        Self::check_directive(
            ctx,
            &MarkupElement::new(element),
            &MarkupDirective::from_relief(directive),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::NoDeprecatedRouterLinkTagProp;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(NoDeprecatedRouterLinkTagProp));
        Linter::with_registry(registry)
    }

    #[test]
    fn reports_static_tag_prop() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<router-link to="/home" tag="button">Home</router-link>"#,
            "App.vue",
        );
        assert_eq!(result.error_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn reports_static_tag_prop_pascal_case() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<RouterLink to="/home" tag="button">Home</RouterLink>"#,
            "App.vue",
        );
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn reports_bound_tag_prop() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<router-link to="/home" :tag="el">Home</router-link>"#,
            "App.vue",
        );
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn allows_router_link_without_tag() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<router-link to="/home" v-slot="{ navigate }"><button @click="navigate">Home</button></router-link>"#,
            "App.vue",
        );
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn allows_tag_on_other_elements() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<my-link tag="button">Home</my-link>"#, "App.vue");
        assert_eq!(result.error_count, 0);
    }
}
