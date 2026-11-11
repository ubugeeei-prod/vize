//! a11y/click-events-have-key-events
//!
//! Require keyboard event handlers with click events on non-interactive elements.
//!
//! Non-interactive elements with click handlers should also have keyboard event
//! handlers to ensure keyboard accessibility.
//!
//! Based on eslint-plugin-vuejs-accessibility click-events-have-key-events rule.

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::markup::{MarkupBindingKind, MarkupContext, MarkupElement, MarkupRule, MarkupTraversal};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::{ElementNode, Namespace};

use super::helpers;
use super::markup_helpers::{
    has_static_markup_event, is_aria_hidden_true_markup, is_component_like_markup_element,
    is_presentation_markup_role,
};
use super::reference_values::is_hidden_from_screen_reader;
use super::vuejs_accessibility::{
    has_on_directive, has_on_directives, is_custom_component, is_html_namespace,
    is_interactive_element, is_jsx_filename, is_presentation_role, is_vue_template,
};

static META: RuleMeta = RuleMeta {
    name: "a11y/click-events-have-key-events",
    description: "Require keyboard event handlers with click events",
    category: RuleCategory::Accessibility,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Require keyboard event handlers with click events
#[derive(Default)]
pub struct ClickEventsHaveKeyEvents;

/// JSX keeps its exact static attribute and handler lookups.
fn legacy_jsx_reports(element: &MarkupElement<'_>) -> bool {
    let mut role = None;
    element.walk_bindings(&mut |binding| {
        if role.is_none()
            && binding.kind() == MarkupBindingKind::Attribute
            && binding.is_unqualified_arg_exact("role")
        {
            role = binding.static_value();
        }
    });
    !is_component_like_markup_element(element)
        && !helpers::is_interactive_element(element.tag())
        && has_static_markup_event(element, &["click"])
        && !is_aria_hidden_true_markup(element)
        && !is_presentation_markup_role(element)
        && !role.is_some_and(helpers::is_interactive_role)
        && !has_static_markup_event(element, &["keydown", "keyup", "keypress"])
}

impl MarkupRule for ClickEventsHaveKeyEvents {
    fn name(&self) -> &'static str {
        META.name
    }

    fn traversal(&self) -> MarkupTraversal {
        MarkupTraversal::Elements
    }

    fn enter_element<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, element: &MarkupElement<'a>) {
        let template = is_vue_template(ctx);
        if Self::reports(element, template, || is_html_namespace(ctx, element)) {
            Self::report(ctx.lint(), element);
        }
    }
}

impl ClickEventsHaveKeyEvents {
    fn reports(
        element: &MarkupElement<'_>,
        template: bool,
        html_namespace: impl FnOnce() -> bool,
    ) -> bool {
        if !template {
            return legacy_jsx_reports(element);
        }
        // The handler first: most elements have none.
        has_on_directive(element, "click", true)
            && !is_custom_component(element, html_namespace)
            && !is_hidden_from_screen_reader(element)
            && !is_presentation_role(element)
            && !is_interactive_element(element)
            && !has_on_directives(element, &["keydown", "keyup", "keypress"], true)
    }

    fn report(ctx: &mut LintContext<'_>, element: &MarkupElement<'_>) {
        ctx.warn_at_with_help(
            ctx.t("a11y/click-events-have-key-events.message"),
            element.range(),
            ctx.t("a11y/click-events-have-key-events.help"),
        );
    }
}

impl Rule for ClickEventsHaveKeyEvents {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn as_markup_rule(&self) -> Option<&dyn MarkupRule> {
        Some(self)
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        let template = !is_jsx_filename(ctx.filename);
        let markup = MarkupElement::new(element);
        if Self::reports(&markup, template, || element.ns == Namespace::Html) {
            Self::report(ctx, &markup);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ClickEventsHaveKeyEvents;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(ClickEventsHaveKeyEvents));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_button() {
        let linter = create_linter();
        let result =
            linter.lint_template(r#"<button @click="handleClick">Click</button>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_div_with_both() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div @click="handleClick" @keydown="handleKeydown">Click</div>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn an_interactive_role_still_needs_a_key_handler() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div role="button" @click="handleClick">Click</div>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_div_click_only() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div @click="handleClick">Click</div>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_valid_dynamic_click_argument() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div @[click]="handleClick">Click</div>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_span_click_only() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<span @click="toggle">Toggle</span>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn modifier_only_click_is_not_a_handler() {
        let result = create_linter().lint_template(
            r#"<span @click.stop><button>Action</button></span>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0, "{:?}", result.diagnostics);
    }

    #[test]
    fn test_valid_presentation_and_none_click() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div role="presentation" @click="close" /><div role="none" @click="close" />"#,
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
    fn test_valid_vue_component_with_click() {
        let linter = create_linter();
        // Vue components handle their own a11y internally
        let result = linter.lint_template(
            r#"<MkButton @click="handleClick">Click</MkButton>"#,
            "test.vue",
        );
        assert_eq!(
            result.warning_count, 0,
            "Should not flag Vue components with @click"
        );
    }

    #[test]
    fn test_valid_nuxt_link_with_click() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<NuxtLink @click.stop>Link</NuxtLink>"#, "test.vue");
        assert_eq!(
            result.warning_count, 0,
            "Should not flag NuxtLink component"
        );
    }

    #[test]
    fn test_valid_kebab_case_component_with_click() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<a-button type="primary" @click="handleClick">Click</a-button>"#,
            "test.vue",
        );
        assert_eq!(
            result.warning_count, 0,
            "Should not flag kebab-case Vue components with @click"
        );
    }

    fn reported_lines(source: &str, result: &crate::linter::LintResult) -> Vec<usize> {
        result
            .diagnostics
            .iter()
            .map(|diagnostic| {
                source
                    .get(..diagnostic.start as usize)
                    .map_or(0, |prefix| prefix.matches('\n').count())
                    + 1
            })
            .collect()
    }

    #[test]
    fn matches_the_plugin_on_issue_12760() {
        // eslint-plugin-vuejs-accessibility 2.6.3 reports exactly these lines.
        let source = r#"<script setup lang="ts">
const onEvent = (): void => undefined;
const keys = {
  onKeydown: (): void => undefined,
  tabindex: (option: string): number => (option === "a" ? 0 : -1),
};
const role = "button";
const hidden = true;
const label = "Options";
const select = (option: string): void => void option;
</script>

<template>
  <div role="button" tabindex="0" @click="onEvent">x</div>
  <div role="checkbox" tabindex="0" @click="onEvent">x</div>
  <div role="columnheader" tabindex="0" @click="onEvent">x</div>
  <div role="combobox" tabindex="0" @click="onEvent">x</div>
  <div role="gridcell" tabindex="0" @click="onEvent">x</div>
  <div role="link" tabindex="0" @click="onEvent">x</div>
  <div role="listbox" tabindex="0" @click="onEvent">x</div>
  <div role="menu" tabindex="0" @click="onEvent">x</div>
  <div role="menuitem" tabindex="0" @click="onEvent">x</div>
  <div role="menuitemcheckbox" tabindex="0" @click="onEvent">x</div>
  <div role="menuitemradio" tabindex="0" @click="onEvent">x</div>
  <div role="option" tabindex="0" @click="onEvent">x</div>
  <div role="radio" tabindex="0" @click="onEvent">x</div>
  <div role="row" tabindex="0" @click="onEvent">x</div>
  <div role="scrollbar" tabindex="0" @click="onEvent">x</div>
  <div role="searchbox" tabindex="0" @click="onEvent">x</div>
  <div role="slider" tabindex="0" @click="onEvent">x</div>
  <div role="spinbutton" tabindex="0" @click="onEvent">x</div>
  <div role="switch" tabindex="0" @click="onEvent">x</div>
  <div role="tab" tabindex="0" @click="onEvent">x</div>
  <div role="tablist" tabindex="0" @click="onEvent">x</div>
  <div role="textbox" tabindex="0" @click="onEvent">x</div>
  <div role="tree" tabindex="0" @click="onEvent">x</div>
  <div role="treeitem" tabindex="0" @click="onEvent">x</div>
  <div @click="onEvent" @keydown="keys.onKeydown">x</div>
  <div @click="keys.onKeydown">x</div>
  <div aria-hidden @click="onEvent">x</div>
  <div :aria-hidden="hidden" @click="onEvent">x</div>
  <a @click="onEvent">x</a>
  <a tabindex="0" @click="onEvent">x</a>
  <div :role="'presentation'" @click="onEvent">x</div>
  <select aria-label="s"><option @click="onEvent">a</option></select>
  <details @click="onEvent"><summary>s</summary>x</details>
  <details><summary @click="onEvent">s</summary>x</details>
  <video src="/v.mp4" @click="onEvent"><track kind="captions" /></video>
  <audio src="/a.mp3" @click="onEvent"><track kind="captions" /></audio>
</template>
"#;
        let result = create_linter().lint_sfc(source, "App.vue");
        assert_eq!(
            reported_lines(source, &result),
            vec![
                14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34,
                35, 36, 37, 38, 42, 43, 46, 47, 48, 49
            ],
            "{:?}",
            result.diagnostics
        );
    }
}
