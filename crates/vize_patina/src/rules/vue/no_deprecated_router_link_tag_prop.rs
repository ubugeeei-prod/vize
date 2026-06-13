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
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_carton::dialect::VueDialect;
use vize_relief::{ElementNode, ExpressionNode, PropNode};

static META: RuleMeta = RuleMeta {
    name: "vue/no-deprecated-router-link-tag-prop",
    description: "Disallow the `tag` prop on <router-link>",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Error,
};

/// Disallow the `tag` prop on `<router-link>`.
pub struct NoDeprecatedRouterLinkTagProp;

impl Rule for NoDeprecatedRouterLinkTagProp {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        // Vue Router 4 (the router for the default Vue 3 dialect) removed `tag`.
        if ctx.dialect() != VueDialect::Vue {
            return;
        }

        // Match both the kebab-case and PascalCase spellings of the component.
        if element.tag != "router-link" && element.tag != "RouterLink" {
            return;
        }

        for prop in element.props.iter() {
            match prop {
                PropNode::Attribute(attr) if attr.name == "tag" => {
                    ctx.error_with_help(
                        ctx.t("vue/no-deprecated-router-link-tag-prop.message"),
                        &attr.loc,
                        ctx.t("vue/no-deprecated-router-link-tag-prop.help"),
                    );
                }
                PropNode::Directive(dir)
                    if dir.name == "bind"
                        && matches!(
                            &dir.arg,
                            Some(ExpressionNode::Simple(s)) if s.content == "tag"
                        ) =>
                {
                    ctx.error_with_help(
                        ctx.t("vue/no-deprecated-router-link-tag-prop.message"),
                        &dir.loc,
                        ctx.t("vue/no-deprecated-router-link-tag-prop.help"),
                    );
                }
                _ => {}
            }
        }
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
