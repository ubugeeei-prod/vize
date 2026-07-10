//! vue/valid-template-root
//!
//! Enforce a valid `<template>` root for Vue 3 fragment semantics.
//!
//! Vue 3 templates compile to a fragment, so multiple root nodes and
//! `v-if` / `v-else-if` / `v-else` roots are all valid and are **not** flagged.
//!
//! Under Vue 3 fragment semantics a root `<slot>` renders the slot content (the
//! canonical Nuxt `<template><slot/></template>` layout) and a root `<template>`
//! carrying a structural directive (`v-if`/`v-else-if`/`v-else`/`v-for`) is a
//! valid fragment. Both render and are accepted by the compiler, so neither is
//! flagged. The one construct this rule still reports is a *bare* `<template>`
//! root — a directive-less wrapper that only nests its children one level
//! deeper for no effect.
//!
//! ## Scope note
//!
//! eslint's `vue/valid-template-root` also reports an empty `<template>` and a
//! `v-for` on a root node. Those checks require knowing that the node being
//! linted is genuinely the *entire* SFC `<template>` root. In this codebase a
//! template root is linted the same way whether it is a full SFC root or an
//! isolated fragment/snippet (an empty fragment and a single `v-for` root are
//! both valid inputs to lint), so flagging them here would be unsound and
//! produce false positives. This rule therefore implements the sound subset:
//! the root-element-kind check, which holds in either case.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <template>
//!   <template>content</template>
//! </template>
//! ```
//!
//! ### Valid
//! ```vue
//! <template>
//!   <div>content</div>
//! </template>
//! ```
//! ```vue
//! <template>
//!   <slot />
//! </template>
//! ```
//! ```vue
//! <template>
//!   <template v-if="ok"><div>a</div></template>
//! </template>
//! ```
//! ```vue
//! <template>
//!   <header>a</header>
//!   <main>b</main>
//! </template>
//! ```
//! ```vue
//! <template>
//!   <div v-if="ok">a</div>
//!   <div v-else>b</div>
//! </template>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::{ElementNode, PropNode, RootNode, TemplateChildNode};

/// Structural directives that turn a `<template>` root into a real Vue 3
/// fragment (its children render in its place).
fn has_fragment_directive(element: &ElementNode) -> bool {
    element.props.iter().any(|prop| {
        matches!(
            prop,
            PropNode::Directive(directive)
                if matches!(directive.name.as_str(), "if" | "else-if" | "else" | "for")
        )
    })
}

static META: RuleMeta = RuleMeta {
    name: "vue/valid-template-root",
    description: "Enforce a valid `<template>` root for Vue 3 fragment semantics",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Error,
};

/// Enforce a valid `<template>` root.
pub struct ValidTemplateRoot;

impl Rule for ValidTemplateRoot {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_template<'a>(&self, ctx: &mut LintContext<'a>, root: &RootNode<'a>) {
        for child in &root.children {
            // Under Vue 3 fragment semantics a `<slot>` root renders the slot
            // content (the canonical Nuxt `<template><slot/></template>` layout)
            // and a `<template>` root carrying a structural directive
            // (`v-if`/`v-else-if`/`v-else`/`v-for`) is a valid fragment. Both
            // render and are accepted by the compiler, so neither is flagged.
            //
            // The one residual case is a *bare* `<template>` root with no
            // structural directive: an inert wrapper that only nests its
            // children one level deeper for no effect.
            if let TemplateChildNode::Element(element) = child {
                let tag = element.tag.as_str();
                if tag == "template" && !has_fragment_directive(element) {
                    ctx.error_with_help(
                        ctx.t_fmt("vue/valid-template-root.disallowed_root", &[("tag", tag)]),
                        &element.loc,
                        ctx.t("vue/valid-template-root.help"),
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ValidTemplateRoot;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(ValidTemplateRoot));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_single_root() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div>content</div>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_multiple_roots() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<header>a</header><main>b</main>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_v_if_v_else_roots() {
        let linter = create_linter();
        let result =
            linter.lint_template(r#"<div v-if="ok">a</div><div v-else>b</div>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_text_root() {
        let linter = create_linter();
        let result = linter.lint_template(r#"hello {{ name }}"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_nested_template() {
        // A `<template>` that is *not* a root (it has a wrapping element) is the
        // job of `vue/no-lone-template`, not this rule.
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div><template v-if="ok"><span>a</span></template></div>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_nested_slot() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div><slot /></div>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_slot_root() {
        // `<template><slot/></template>` is the canonical Nuxt layout and renders
        // the slot content under Vue 3 fragment semantics.
        let linter = create_linter();
        let result = linter.lint_template(r#"<slot />"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_slot_root_among_siblings() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div>a</div><slot />"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_template_fragment_root() {
        // A `<template>` root carrying a structural directive is a valid Vue 3
        // fragment (e.g. the common `<template v-if>` / `<template v-else>` pair).
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<template v-if="ok"><div>a</div></template><template v-else><div>b</div></template>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_template_for_root() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<template v-for="x in xs" :key="x"><li>{{ x }}</li></template>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_invalid_bare_template_root() {
        // A bare `<template>` root with no structural directive is an inert
        // wrapper and is still reported.
        let linter = create_linter();
        let result = linter.lint_template(r#"<template><div>content</div></template>"#, "test.vue");
        assert_eq!(result.error_count, 1);
        assert_eq!(result.diagnostics[0].rule_name, "vue/valid-template-root");
    }
}
