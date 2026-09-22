//! a11y/mouse-events-have-key-events
//!
//! Require focus/blur events with mouseenter/mouseleave events.
//!
//! Mouse-only event handlers exclude keyboard and touch users.
//! `@mouseenter`/`@mouseover` should be paired with `@focus`,
//! and `@mouseleave`/`@mouseout` should be paired with `@blur`.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <div @mouseenter="show">...</div>
//! ```
//!
//! ### Valid
//! ```vue
//! <div @mouseenter="show" @focus="show">...</div>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::markup::{MarkupBindingKind, MarkupContext, MarkupElement, MarkupHooks, MarkupRule};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::{ElementNode, ElementType};

use super::helpers::has_event_handler;

static META: RuleMeta = RuleMeta {
    name: "a11y/mouse-events-have-key-events",
    description: "Require focus/blur events with mouse events",
    category: RuleCategory::Accessibility,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Require focus/blur events with mouse events
#[derive(Default)]
pub struct MouseEventsHaveKeyEvents;

/// The static event handlers an element carries, read in one binding pass.
#[derive(Default)]
struct Handlers {
    mouse_enter: bool,
    mouse_leave: bool,
    focus: bool,
    blur: bool,
}

impl MouseEventsHaveKeyEvents {
    fn handlers(element: &MarkupElement<'_>) -> Handlers {
        let mut handlers = Handlers::default();
        element.walk_bindings(&mut |binding| {
            if binding.kind() != MarkupBindingKind::On {
                return;
            }
            let is = |event: &str| binding.is_static_unqualified_arg_exact(event);
            handlers.mouse_enter |= is("mouseenter") || is("mouseover");
            handlers.mouse_leave |= is("mouseleave") || is("mouseout");
            handlers.focus |= is("focus");
            handlers.blur |= is("blur");
        });
        handlers
    }
}

impl MarkupRule for MouseEventsHaveKeyEvents {
    fn name(&self) -> &'static str {
        META.name
    }

    fn hooks(&self) -> MarkupHooks {
        MarkupHooks::ELEMENT
    }

    fn enter_element<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, element: &MarkupElement<'a>) {
        if element.is_component() {
            return;
        }

        let handlers = Self::handlers(element);
        if handlers.mouse_enter && !handlers.focus {
            let message = ctx
                .lint()
                .t("a11y/mouse-events-have-key-events.message_enter");
            let help = ctx.lint().t("a11y/mouse-events-have-key-events.help");
            ctx.lint().warn_at_with_help(message, element.range(), help);
        }

        if handlers.mouse_leave && !handlers.blur {
            let message = ctx
                .lint()
                .t("a11y/mouse-events-have-key-events.message_leave");
            let help = ctx.lint().t("a11y/mouse-events-have-key-events.help");
            ctx.lint().warn_at_with_help(message, element.range(), help);
        }
    }
}

impl Rule for MouseEventsHaveKeyEvents {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn as_markup_rule(&self) -> Option<&dyn MarkupRule> {
        Some(self)
    }

    fn jsx_needs_lowering(&self) -> bool {
        true
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        if element.tag_type == ElementType::Component {
            return;
        }

        // mouseenter/mouseover require @focus
        if (has_event_handler(element, "mouseenter") || has_event_handler(element, "mouseover"))
            && !has_event_handler(element, "focus")
        {
            ctx.warn_with_help(
                ctx.t("a11y/mouse-events-have-key-events.message_enter"),
                &element.loc,
                ctx.t("a11y/mouse-events-have-key-events.help"),
            );
        }

        // mouseleave/mouseout require @blur
        if (has_event_handler(element, "mouseleave") || has_event_handler(element, "mouseout"))
            && !has_event_handler(element, "blur")
        {
            ctx.warn_with_help(
                ctx.t("a11y/mouse-events-have-key-events.message_leave"),
                &element.loc,
                ctx.t("a11y/mouse-events-have-key-events.help"),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::MouseEventsHaveKeyEvents;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(MouseEventsHaveKeyEvents));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_mouseenter_with_focus() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div @mouseenter="show" @focus="show">Content</div>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_mouseleave_with_blur() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div @mouseleave="hide" @blur="hide">Content</div>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_mouseenter_without_focus() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div @mouseenter="show">Content</div>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_mouseleave_without_blur() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div @mouseleave="hide">Content</div>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_both_missing() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div @mouseenter="show" @mouseleave="hide">Content</div>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 2);
    }

    #[test]
    fn test_invalid_mouseenter_with_dynamic_focus_argument() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div @mouseenter="show" @[focus]="show">Content</div>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_valid_dynamic_mouseenter_argument() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div @[mouseenter]="show">Content</div>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_component_skipped() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<MyTooltip @mouseenter="show">Tip</MyTooltip>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }
}
