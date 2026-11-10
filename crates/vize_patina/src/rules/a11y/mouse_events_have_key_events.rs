//! a11y/mouse-events-have-key-events
//!
//! Require focus/blur events with mouseenter/mouseleave events.
//!
//! Mouse-only event handlers exclude keyboard and touch users.
//! `@mouseenter`/`@mouseover`/`@hover` should be paired with `@focus` or `@focusin`,
//! and `@mouseleave`/`@mouseout` should be paired with `@blur` or `@focusout`.
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
use crate::markup::{MarkupBindingKind, MarkupContext, MarkupElement, MarkupRule};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::ElementNode;

use super::markup_helpers;
use super::vuejs_accessibility::{
    has_on_directive, has_on_directives, is_jsx_filename, is_vue_template,
};

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

impl MouseEventsHaveKeyEvents {
    fn has_static_event_handler(element: &MarkupElement<'_>, event_name: &str) -> bool {
        let mut found = false;
        element.walk_bindings(&mut |binding| {
            if (binding.kind() == MarkupBindingKind::On
                && binding.is_static_unqualified_arg_exact(event_name))
                || (element.is_component()
                    && binding.kind() == MarkupBindingKind::Bind
                    && binding.is_static_event_handler_for(event_name))
            {
                found = true;
            }
        });
        found
    }

    /// Presentation/none and `aria-hidden="true"` opt the element out, the same
    /// way the click and static-element rules do.
    fn is_markup_opt_out(element: &MarkupElement<'_>) -> bool {
        if matches!(
            markup_helpers::get_static_markup_attribute_value(element, "role"),
            Some("presentation" | "none")
        ) {
            return true;
        }
        if markup_helpers::get_static_or_bound_literal_markup_value(element, "aria-hidden")
            == Some("true")
        {
            return true;
        }
        let mut hidden = false;
        element.walk_bindings(&mut |binding| {
            if !hidden
                && binding.kind() == MarkupBindingKind::Bind
                && binding.is_static_unqualified_arg_exact("aria-hidden")
                && binding
                    .expression()
                    .is_some_and(|expression| expression.trim() == "true")
            {
                hidden = true;
            }
        });
        hidden
    }
}

impl MarkupRule for MouseEventsHaveKeyEvents {
    fn name(&self) -> &'static str {
        META.name
    }

    fn traversal(&self) -> crate::markup::MarkupTraversal {
        crate::markup::MarkupTraversal::Elements
    }

    fn enter_element<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, element: &MarkupElement<'a>) {
        if is_vue_template(ctx) {
            Self::check_template(ctx.lint(), element);
        } else {
            Self::check_jsx(ctx, element);
        }
    }
}

impl MouseEventsHaveKeyEvents {
    fn report(ctx: &mut LintContext<'_>, element: &MarkupElement<'_>, message: &str) {
        ctx.warn_at_with_help(
            ctx.t(message),
            element.range(),
            ctx.t("a11y/mouse-events-have-key-events.help"),
        );
    }

    /// The plugin checks every element, components included: an undeclared
    /// listener falls through to the component root.
    fn check_template(ctx: &mut LintContext<'_>, element: &MarkupElement<'_>) {
        let over = has_on_directives(element, &["mouseover", "mouseenter"], true);
        if (over || has_on_directive(element, "hover", true))
            && !has_on_directives(element, &["focus", "focusin"], true)
        {
            let message = if over {
                "a11y/mouse-events-have-key-events.message_enter"
            } else {
                "a11y/mouse-events-have-key-events.message_hover"
            };
            Self::report(ctx, element, message);
        }
        if has_on_directives(element, &["mouseout", "mouseleave"], true)
            && !has_on_directives(element, &["blur", "focusout"], true)
        {
            Self::report(
                ctx,
                element,
                "a11y/mouse-events-have-key-events.message_leave",
            );
        }
    }

    fn check_jsx(ctx: &mut MarkupContext<'_, '_>, element: &MarkupElement<'_>) {
        if Self::is_markup_opt_out(element) {
            return;
        }

        let has_hover = Self::has_static_event_handler(element, "hover");
        if element.is_component() {
            if has_hover
                && !Self::has_static_event_handler(element, "focus")
                && !Self::has_static_event_handler(element, "focusin")
            {
                let message = ctx
                    .lint()
                    .t("a11y/mouse-events-have-key-events.message_hover");
                let help = ctx.lint().t("a11y/mouse-events-have-key-events.help");
                ctx.lint().warn_at_with_help(message, element.range(), help);
            }
            return;
        }

        let has_mouse_enter = Self::has_static_event_handler(element, "mouseenter")
            || Self::has_static_event_handler(element, "mouseover")
            || has_hover;
        if has_mouse_enter
            && !Self::has_static_event_handler(element, "focus")
            && !Self::has_static_event_handler(element, "focusin")
        {
            let message = ctx
                .lint()
                .t("a11y/mouse-events-have-key-events.message_enter");
            let help = ctx.lint().t("a11y/mouse-events-have-key-events.help");
            ctx.lint().warn_at_with_help(message, element.range(), help);
        }

        let has_mouse_leave = Self::has_static_event_handler(element, "mouseleave")
            || Self::has_static_event_handler(element, "mouseout");
        if has_mouse_leave
            && !Self::has_static_event_handler(element, "blur")
            && !Self::has_static_event_handler(element, "focusout")
        {
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
        if !is_jsx_filename(ctx.filename) {
            Self::check_template(ctx, &MarkupElement::new(element));
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
    fn presentation_and_aria_hidden_do_not_opt_out() {
        let linter = create_linter();
        for source in [
            r#"<div role="presentation" @mouseover="highlight" @mouseout="unhighlight" />"#,
            r#"<div role="none" @mouseenter="highlight" @mouseleave="unhighlight" />"#,
            r#"<div aria-hidden="true" @mouseover="highlight" @mouseout="unhighlight" />"#,
        ] {
            let template = linter.lint_template(source, "test.vue");
            assert_eq!(template.warning_count, 2, "{source}");
            let sfc = linter.lint_sfc(
                &format!("<template>\n  {source}\n</template>\n"),
                "MyOverlay.vue",
            );
            assert_eq!(sfc.warning_count, 2, "{source}");
        }
    }

    #[test]
    fn test_valid_focusin_and_focusout_counterparts() {
        let linter = create_linter();
        for source in [
            r#"<div @mouseenter="show" @focusin="show">Content</div>"#,
            r#"<div @mouseover="show" @focusin="show">Content</div>"#,
            r#"<div @mouseleave="hide" @focusout="hide">Content</div>"#,
            r#"<div @mouseout="hide" @focusout="hide">Content</div>"#,
            r#"<div @mouseenter="show" @mouseleave="hide" @focusin="show" @focusout="hide">Content</div>"#,
        ] {
            let template = linter.lint_template(source, "test.vue");
            assert_eq!(template.warning_count, 0, "{source}");
            let sfc = linter.lint_sfc(
                &format!("<template>\n  {source}\n</template>\n"),
                "Hover.vue",
            );
            assert_eq!(sfc.warning_count, 0, "{source}");
        }
    }

    #[test]
    fn test_invalid_mismatched_focus_counterparts() {
        let linter = create_linter();
        for source in [
            r#"<div @mouseenter="show" @focusout="hide">Content</div>"#,
            r#"<div @mouseleave="hide" @focusin="show">Content</div>"#,
        ] {
            let template = linter.lint_template(source, "test.vue");
            assert_eq!(template.warning_count, 1, "{source}");
            let sfc = linter.lint_sfc(
                &format!("<template>\n  {source}\n</template>\n"),
                "Hover.vue",
            );
            assert_eq!(sfc.warning_count, 1, "{source}");
        }
    }

    #[test]
    fn test_component_hover_requires_focus() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<MyTooltip @hover="show" />
<MyTooltip @hover="() => show()" />
<MyTooltip @hover="show" @focus="show" />"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 2);
    }

    #[test]
    fn listener_props_are_not_v_on_handlers() {
        let result = create_linter().lint_template(
            r#"<MyTooltip :onHover="show" />
<MyTooltip :onHover="show" :onFocus="show" />"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn component_mouse_listeners_and_handler_values_follow_the_plugin() {
        let result = create_linter().lint_template(
            r#"<div>
<MyButton @mouseenter="show" />
<MyButton @mouseleave="hide" />
<MyButton @mouseover="show" @mouseout="hide" />
<MyButton @mouseenter="show" @focus="show" />
<div @mouseover="void 0" @focus="null">a</div>
<div @mouseout="void 0" @blur="null">b</div>
<div @mouseenter="null">c</div>
<div @mouseover="keys.show" @mouseout="hide" @blur="keys.hide">d</div>
</div>"#,
            "test.vue",
        );
        // A member expression or `null` is no handler; both groups report on `@mouseover @mouseout`.
        assert_eq!(result.warning_count, 7, "{:?}", result.diagnostics);
    }
}
