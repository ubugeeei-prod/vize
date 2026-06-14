//! vue/require-toggle-inside-transition
//!
//! Require a toggle on the element wrapped by `<transition>`.
//!
//! A `<transition>` animates a single child as it enters and leaves the DOM.
//! That enter/leave only happens when the child is actually toggled. If the
//! wrapped element is unconditionally present — it has no `v-if`, `v-show`,
//! `v-else`, `v-else-if`, no `:key` that changes, and is not a dynamic
//! `<component :is>` — the transition never plays and the `<transition>` is
//! dead weight.
//!
//! This mirrors eslint-plugin-vue's `vue/require-toggle-inside-transition`
//! (the Vue 3 `essential` preset).
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <transition>
//!   <div>content</div>
//! </transition>
//! ```
//!
//! ### Valid
//! ```vue
//! <transition>
//!   <div v-if="show">content</div>
//! </transition>
//! ```
//!
//! ```vue
//! <transition>
//!   <div v-show="show">content</div>
//! </transition>
//! ```
//!
//! ```vue
//! <transition>
//!   <component :is="view" />
//! </transition>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::{ElementNode, ElementType, ExpressionNode, PropNode, TemplateChildNode};

static META: RuleMeta = RuleMeta {
    name: "vue/require-toggle-inside-transition",
    description: "Require a toggle on the element wrapped by `<transition>`",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Error,
};

/// Require a toggle directive on the element wrapped by `<transition>`.
pub struct RequireToggleInsideTransition;

impl RequireToggleInsideTransition {
    /// A whitespace-only text node or a comment is layout noise between the
    /// transition tags and is ignored when locating the wrapped element.
    fn is_ignorable(child: &TemplateChildNode) -> bool {
        match child {
            TemplateChildNode::Comment(_) => true,
            TemplateChildNode::Text(text) => text.content.trim().is_empty(),
            _ => false,
        }
    }

    /// Return the sole element child of `<transition>`, or `None` when the
    /// transition wraps zero, several, or non-element content.
    ///
    /// `<transition>` is only meaningful around a single element, so anything
    /// else (an empty transition, a `v-if`/`v-else` pair forming multiple roots,
    /// or bare text) is outside this rule's scope.
    fn sole_element_child<'e, 'a>(element: &'e ElementNode<'a>) -> Option<&'e ElementNode<'a>> {
        let mut sole: Option<&ElementNode<'a>> = None;
        for child in &element.children {
            if Self::is_ignorable(child) {
                continue;
            }
            match child {
                TemplateChildNode::Element(el) => {
                    if sole.is_some() {
                        // More than one element child: outside the single-child
                        // shape this rule targets.
                        return None;
                    }
                    sole = Some(el);
                }
                // Meaningful non-element content (interpolation, ...) means the
                // wrapped node is not a lone element.
                _ => return None,
            }
        }
        sole
    }

    /// Whether `element` carries something that makes it enter/leave: a
    /// conditional-render directive, a `v-show`, or a bound `:key`.
    fn has_toggle(element: &ElementNode) -> bool {
        for prop in &element.props {
            let PropNode::Directive(dir) = prop else {
                continue;
            };
            match dir.name.as_str() {
                // Conditional rendering toggles the element's presence.
                "if" | "else" | "else-if" | "show" => return true,
                // A bound `:key` (`v-bind:key`) forces a re-mount on change.
                "bind" => {
                    if let Some(ExpressionNode::Simple(arg)) = &dir.arg
                        && arg.content.as_str() == "key"
                    {
                        return true;
                    }
                }
                _ => {}
            }
        }
        false
    }
}

impl Rule for RequireToggleInsideTransition {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        // Built-in `<transition>` / `<Transition>` wrapper (case-insensitive,
        // matching how Vue resolves the built-in component name).
        if !element.tag.as_str().eq_ignore_ascii_case("transition") {
            return;
        }

        let Some(child) = Self::sole_element_child(element) else {
            return;
        };

        // A custom component may toggle itself internally, and `<slot>` content
        // is supplied by the parent — neither can be judged here, so both are
        // left alone (matching eslint-plugin-vue).
        if child.tag_type == ElementType::Component || child.tag_type == ElementType::Slot {
            return;
        }

        // `<component :is="...">` is a dynamic component: swapping `is` is itself
        // an enter/leave, so it always animates.
        if child.tag.as_str() == "component" {
            return;
        }

        if Self::has_toggle(child) {
            return;
        }

        ctx.error_with_help(
            ctx.t("vue/require-toggle-inside-transition.message"),
            &child.loc,
            ctx.t("vue/require-toggle-inside-transition.help"),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::RequireToggleInsideTransition;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(RequireToggleInsideTransition));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_invalid_plain_element() {
        let linter = create_linter();
        let result =
            linter.lint_template(r#"<transition><div>content</div></transition>"#, "test.vue");
        assert_eq!(result.error_count, 1);
        assert_eq!(
            result.diagnostics[0].rule_name,
            "vue/require-toggle-inside-transition"
        );
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn test_invalid_pascal_case_transition() {
        let linter = create_linter();
        // `<Transition>` (PascalCase) resolves to the same built-in.
        let result = linter.lint_template(r#"<Transition><p>x</p></Transition>"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_static_key_only() {
        let linter = create_linter();
        // A *static* `key` attribute does not change, so it does not trigger an
        // enter/leave the way a bound `:key` does.
        let result = linter.lint_template(
            r#"<transition><div key="static">x</div></transition>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_unrelated_binding() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<transition><div :class="cls">x</div></transition>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_valid_v_if() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<transition><div v-if="show">x</div></transition>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_v_show() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<transition><div v-show="show">x</div></transition>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_v_else() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<transition><div v-else>x</div></transition>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_v_else_if() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<transition><div v-else-if="b">x</div></transition>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_bound_key() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<transition><div :key="k">x</div></transition>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_v_bind_key_longhand() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<transition><div v-bind:key="k">x</div></transition>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_dynamic_component() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<transition><component :is="view" /></transition>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_custom_component_child() {
        let linter = create_linter();
        // A custom component may toggle itself internally; do not flag it.
        let result = linter.lint_template(r#"<transition><MyModal /></transition>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_slot_child() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<transition><slot /></transition>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_empty_transition() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<transition></transition>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_multiple_children() {
        let linter = create_linter();
        // A v-if/v-else pair is two element children; the toggle lives on the
        // pair, not a single wrapped element, so the rule does not apply.
        let result = linter.lint_template(
            r#"<transition><div v-if="a">a</div><div v-else>b</div></transition>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_invalid_ignores_surrounding_comment_and_whitespace() {
        let linter = create_linter();
        // Comments and whitespace around the wrapped element are ignored when
        // locating the single child, so this still reports.
        let result = linter.lint_template(
            "<transition>\n  <!-- wrap --><div>x</div>\n</transition>",
            "test.vue",
        );
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_valid_non_transition_element() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div><span>x</span></div>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }
}
