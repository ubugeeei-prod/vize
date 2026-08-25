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
            match dir.name {
                // Conditional rendering toggles the element's presence.
                "if" | "else" | "else-if" | "show" => return true,
                // A bound `:key` (`v-bind:key`) forces a re-mount on change.
                "bind" => {
                    if let Some(ExpressionNode::Simple(arg)) = &dir.arg
                        && arg.content == "key"
                    {
                        return true;
                    }
                }
                _ => {}
            }
        }
        false
    }

    /// `<Transition appear>` animates its initial mount without the child being
    /// toggled by `v-if`, `v-show`, or a changing key.
    fn has_appear(element: &ElementNode) -> bool {
        element.props.iter().any(|prop| match prop {
            PropNode::Attribute(attr) => attr.name == "appear",
            PropNode::Directive(dir) => {
                dir.name == "bind"
                    && matches!(
                        dir.arg.as_ref(),
                        Some(ExpressionNode::Simple(arg)) if arg.content == "appear"
                    )
            }
        })
    }
}

impl Rule for RequireToggleInsideTransition {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        // Built-in `<transition>` / `<Transition>` wrapper (case-insensitive,
        // matching how Vue resolves the built-in component name).
        if !element.tag.eq_ignore_ascii_case("transition") {
            return;
        }

        if Self::has_appear(element) {
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
        if child.tag == "component" {
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
mod tests;
