//! a11y/no-static-element-interactions
//!
//! Disallow event handlers on non-interactive/static elements.
//!
//! Static elements like `<div>`, `<span>` should not have event handlers
//! without also having an interactive role and tabindex.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <div @click="handleClick">Click</div>
//! ```
//!
//! ### Valid
//! ```vue
//! <button @click="handleClick">Click</button>
//! <div role="button" tabindex="0" @click="handleClick">Click</div>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::{ElementNode, ExpressionNode, PropNode};

use super::helpers::{
    get_static_attribute_value, has_interactive_role, is_aria_hidden_true,
    is_component_like_element, is_interactive_element, is_presentation_role,
};

static META: RuleMeta = RuleMeta {
    name: "a11y/no-static-element-interactions",
    description: "Disallow event handlers on static elements",
    category: RuleCategory::Accessibility,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Disallow event handlers on static elements
#[derive(Default)]
pub struct NoStaticElementInteractions;

const INTERACTIVE_EVENTS: &[&str] = &[
    "click",
    "dblclick",
    "mousedown",
    "mouseup",
    "keydown",
    "keyup",
    "keypress",
];

fn has_interactive_event(element: &ElementNode) -> bool {
    for prop in &element.props {
        if let PropNode::Directive(dir) = prop
            && dir.name == "on"
            && let Some(ExpressionNode::Simple(arg)) = &dir.arg
            && arg.is_static
            && INTERACTIVE_EVENTS.contains(&arg.content)
        {
            return true;
        }
    }
    false
}

impl Rule for NoStaticElementInteractions {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        if is_component_like_element(element) {
            return;
        }

        // A bound role is chosen at runtime and is outside this rule's
        // static-attribute domain. Do not infer that it has no role (#10926).
        if element.props.iter().any(|prop| {
            matches!(prop, PropNode::Directive(dir)
                if dir.name == "bind"
                    && matches!(&dir.arg, Some(ExpressionNode::Simple(arg))
                        if arg.is_static && arg.content == "role"))
        }) {
            return;
        }

        if is_aria_hidden_true(element) || is_presentation_role(element) {
            return;
        }

        // Dialogs and toolbars are composite widgets, not static containers.
        // `role="button"` stays on the interactive-role path below.
        if element.tag == "dialog"
            || matches!(
                get_static_attribute_value(element, "role"),
                Some("dialog" | "alertdialog" | "toolbar")
            )
        {
            return;
        }

        // Skip natively interactive elements
        if is_interactive_element(element.tag) {
            return;
        }

        // Skip elements with interactive roles
        if has_interactive_role(element) {
            return;
        }

        // Check if element has interactive event handlers
        if has_interactive_event(element) {
            ctx.warn_with_help(
                ctx.t_fmt(
                    "a11y/no-static-element-interactions.message",
                    &[("tag", element.tag)],
                ),
                &element.loc,
                ctx.t("a11y/no-static-element-interactions.help"),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::NoStaticElementInteractions;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(NoStaticElementInteractions));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_button_with_click() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<button @click="handle">Click</button>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_div_with_role() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div role="button" @click="handle">Click</div>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn bound_roles_stay_outside_the_static_role_domain() {
        let linter = create_linter();
        for binding in [
            r#":role="clickable ? 'button' : undefined""#,
            r#"v-bind:role="role""#,
            r#":role="'button'""#,
            r#":role="'article'""#,
            r#":role="undefined""#,
            ":role",
        ] {
            let source = format!(r#"<span {binding} @click="press" @keydown.enter="press" />"#);
            let result = linter.lint_template(&source, "test.vue");
            assert_eq!(result.error_count, 0, "{source}");
            assert_eq!(result.warning_count, 0, "{source}");
            assert!(result.diagnostics.is_empty(), "{source}: {result:?}");
        }
    }

    #[test]
    fn other_bindings_do_not_hide_static_element_diagnostics() {
        let linter = create_linter();
        for binding in [
            r#":id="id""#,
            r#":tabindex="0""#,
            r#"v-role="role""#,
            r#":[role]="value""#,
            r#"role="article""#,
        ] {
            let source = format!(r#"<span {binding} @click="press" />"#);
            let result = linter.lint_template(&source, "test.vue");
            assert_eq!(result.error_count, 0, "{source}");
            assert_eq!(result.warning_count, 1, "{source}");
            assert_eq!(result.diagnostics.len(), 1, "{source}");
            let diagnostic = &result.diagnostics[0];
            assert_eq!(diagnostic.rule_name, "a11y/no-static-element-interactions");
            assert_eq!(diagnostic.severity, crate::diagnostic::Severity::Warning);
            assert_eq!(
                diagnostic.message,
                "<span> is a static element and should not have interactive event handlers"
            );
            assert_eq!((diagnostic.start, diagnostic.end), (0, source.len() as u32));
            assert!(diagnostic.help.is_some());
            assert!(diagnostic.labels.is_empty());
            assert!(diagnostic.fix.is_none());
        }
    }

    #[test]
    fn clickable_tag_sfc_with_runtime_role_does_not_report() {
        let source = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/_fixtures/differential/linter/a11y-dynamic-role-10926/ClickableTag.vue.txt"
        ));
        for newline in ["\n", "\r\n"] {
            let result =
                create_linter().lint_sfc(&source.replace('\n', newline), "ClickableTag.vue");
            assert_eq!(result.error_count, 0);
            assert_eq!(result.warning_count, 0);
            assert!(result.diagnostics.is_empty(), "{newline:?}: {result:?}");
        }
        let control = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/_fixtures/differential/linter/a11y-dynamic-role-10926/StaticControl.vue.txt"
        ));
        let result = create_linter().lint_sfc(control, "StaticControl.vue");
        assert_eq!(result.error_count, 0);
        assert_eq!(result.warning_count, 1);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(
            result.diagnostics[0].rule_name,
            "a11y/no-static-element-interactions"
        );
    }

    #[test]
    fn test_valid_div_no_events() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div>Content</div>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_div_with_click() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div @click="handle">Click</div>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_valid_dynamic_event_argument() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div @[click]="handle">Click</div>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_span_with_keydown() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<span @keydown="handle">Content</span>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_valid_dialog_alertdialog_toolbar_and_native_dialog() {
        let linter = create_linter();
        for source in [
            r#"<div role="dialog" @click="close">Close</div>"#,
            r#"<div role="alertdialog" @keydown="trap">Alert</div>"#,
            r#"<div role="toolbar" @click="focusTool">Tools</div>"#,
            r#"<dialog @click="close">Close</dialog>"#,
        ] {
            let result = linter.lint_template(source, "test.vue");
            assert_eq!(result.warning_count, 0, "{source}");
        }
    }

    #[test]
    fn test_role_button_without_key_handler_stays_unreported() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div role="button" @click="handle">Click</div>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_presentation_and_none_events() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div role="presentation" @click="close" /><div role="none" @keydown="close" />"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_aria_hidden_backdrop_click() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div aria-hidden="true" class="backdrop" @click="close" />"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_component_skipped() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<MyDiv @click="handle">Click</MyDiv>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_kebab_case_component_skipped() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<a-button type="primary" @click="handle">Click</a-button>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }
}
