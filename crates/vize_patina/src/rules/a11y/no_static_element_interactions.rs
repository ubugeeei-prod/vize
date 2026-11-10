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
use crate::markup::{MarkupBindingKind, MarkupContext, MarkupElement, MarkupRule, MarkupTraversal};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::{ElementNode, Namespace};

use super::helpers;
use super::markup_helpers::{
    get_static_markup_attribute_value, has_static_markup_event, is_aria_hidden_true_markup,
    is_component_like_markup_element, is_presentation_markup_role,
};
use super::reference_values::is_hidden_from_screen_reader;
use super::vuejs_accessibility::{
    AttrValue, INTERACTIVE_HANDLERS, attribute, bound_branches, has_on_directives,
    is_custom_component, is_html_namespace, is_interactive_element, is_interactive_role,
    is_jsx_filename, is_presentation_role, is_vue_template,
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

/// A bound role with an interactive conditional branch is interactive when
/// chosen (#10926); the plugin reads any non-literal binding as no role.
fn has_interactive_role_branch(element: &MarkupElement<'_>) -> bool {
    let mut interactive = false;
    bound_branches(element, "role", &mut |value| {
        interactive |= is_interactive_role(Some(value));
    }) && interactive
}

impl MarkupRule for NoStaticElementInteractions {
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

impl NoStaticElementInteractions {
    fn reports(
        element: &MarkupElement<'_>,
        template: bool,
        html_namespace: impl FnOnce() -> bool,
    ) -> bool {
        if !template {
            return legacy_jsx_reports(element);
        }
        // The handlers first: most elements have none.
        if !has_on_directives(element, INTERACTIVE_HANDLERS, true)
            || is_custom_component(element, html_namespace)
            || is_hidden_from_screen_reader(element)
            || is_presentation_role(element)
            || is_interactive_element(element)
        {
            return false;
        }
        let role = attribute(element, "role");
        !(is_interactive_role(role)
            || (role == Some(AttrValue::Expr) && has_interactive_role_branch(element)))
    }

    fn report(ctx: &mut LintContext<'_>, element: &MarkupElement<'_>) {
        ctx.warn_at_with_help(
            ctx.t_fmt(
                "a11y/no-static-element-interactions.message",
                &[("tag", element.tag())],
            ),
            element.range(),
            ctx.t("a11y/no-static-element-interactions.help"),
        );
    }
}

/// JSX keeps its exact static attribute and handler lookups.
fn legacy_jsx_reports(element: &MarkupElement<'_>) -> bool {
    let mut bound_role = false;
    element.walk_bindings(&mut |binding| {
        bound_role |= binding.kind() == MarkupBindingKind::Bind
            && binding.is_static_unqualified_arg_exact("role");
    });
    if is_component_like_markup_element(element)
        || element.is_unqualified_tag_exact("dialog")
        || helpers::is_interactive_element(element.tag())
        || !has_static_markup_event(element, helpers::INTERACTIVE_HANDLER_EVENTS)
        || bound_role
        || is_aria_hidden_true_markup(element)
        || is_presentation_markup_role(element)
    {
        return false;
    }
    let role = get_static_markup_attribute_value(element, "role");
    !(matches!(role, Some("dialog" | "alertdialog" | "toolbar"))
        || role.is_some_and(helpers::is_interactive_role))
}

impl Rule for NoStaticElementInteractions {
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
    fn bound_roles_follow_the_plugin_unless_a_branch_is_interactive() {
        let linter = create_linter();
        for (binding, expected) in [
            // An interactive branch is a widget when chosen (#10926).
            (r#":role="clickable ? 'button' : undefined""#, 0),
            (r#":role="'button'""#, 0),
            (r#"v-bind:role="role""#, 1),
            (r#":role="'article'""#, 1),
            (r#":role="undefined""#, 1),
            (":role", 1),
            (r#":role="open ? 'group' : 'img'""#, 1),
        ] {
            let source = format!(r#"<span {binding} @click="press" @keydown.enter="press" />"#);
            let result = linter.lint_template(&source, "test.vue");
            assert_eq!(result.warning_count, expected, "{source}: {result:?}");
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
    fn modifier_only_listeners_are_not_interactive_handlers() {
        let result = create_linter().lint_template(
            r#"<span @click.stop><button>Action</button></span><span @mousedown.prevent><button>End</button></span>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0, "{:?}", result.diagnostics);
    }

    #[test]
    fn composite_roles_are_not_static_but_tooltip_is() {
        let source = r#"<div role="tree" @keydown="onKey" /><div role="tablist" @keydown="onKey" /><div role="menu" @keydown="onKey" /><div role="row" @keydown="onKey" /><div role="columnheader" @keydown="onKey" /><div role="tooltip" @keydown="onKey" />"#;
        let result = create_linter().lint_template(source, "test.vue");
        assert_eq!(result.warning_count, 1, "{:?}", result.diagnostics);
    }

    #[test]
    fn dialog_and_toolbar_roles_are_not_widgets() {
        let linter = create_linter();
        for source in [
            r#"<div role="dialog" @click="close">Close</div>"#,
            r#"<div role="alertdialog" @keydown="trap">Alert</div>"#,
            r#"<div role="toolbar" @click="focusTool">Tools</div>"#,
            r#"<dialog @click="close">Close</dialog>"#,
        ] {
            let result = linter.lint_template(source, "test.vue");
            assert_eq!(result.warning_count, 1, "{source}");
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
    fn matches_the_plugin_on_issue_12761() {
        // The plugin also reports line 50; its interactive branch keeps it clean here (#10926).
        let source = r#"<script setup lang="ts">
const onEvent = (): void => undefined;
const keys = {
  onKeydown: (): void => undefined,
  tabindex: (option: string): number => (option === "a" ? 0 : -1),
};
const role = "button";
const enabled = true;
const hidden = true;
const label = "Options";
const select = (option: string): void => void option;
</script>

<template>
  <div @contextmenu="onEvent">x</div>
  <div @doubleclick="onEvent">x</div>
  <div @drag="onEvent">x</div>
  <div @dragend="onEvent">x</div>
  <div @dragenter="onEvent">x</div>
  <div @dragexit="onEvent">x</div>
  <div @dragleave="onEvent">x</div>
  <div @dragover="onEvent">x</div>
  <div @dragstart="onEvent">x</div>
  <div @drop="onEvent">x</div>
  <div @mouseenter="onEvent">x</div>
  <div @mouseleave="onEvent">x</div>
  <div @mousemove="onEvent">x</div>
  <div @mouseout="onEvent">x</div>
  <div @mouseover="onEvent">x</div>
  <div role="grid" tabindex="0" @keydown="onEvent">x</div>
  <div role="menubar" tabindex="0" @keydown="onEvent">x</div>
  <div role="progressbar" tabindex="0" @keydown="onEvent">x</div>
  <div role="radiogroup" tabindex="0" @keydown="onEvent">x</div>
  <div role="rowheader" tabindex="0" @keydown="onEvent">x</div>
  <div role="treegrid" tabindex="0" @keydown="onEvent">x</div>
  <div role="doc-backlink" tabindex="0" @keydown="onEvent">x</div>
  <div role="doc-biblioref" tabindex="0" @keydown="onEvent">x</div>
  <div role="doc-glossref" tabindex="0" @keydown="onEvent">x</div>
  <div role="doc-noteref" tabindex="0" @keydown="onEvent">x</div>
  <div role="dialog" @keydown="onEvent">x</div>
  <div role="alertdialog" @keydown="onEvent">x</div>
  <div role="toolbar" @keydown="onEvent">x</div>
  <div role="BUTTON" @click="onEvent">x</div>
  <div role="button link" @click="onEvent">x</div>
  <dialog @click="onEvent">x</dialog>
  <div @click="keys.onKeydown">x</div>
  <div @keydown="keys.onKeydown">x</div>
  <a @click="onEvent">x</a>
  <div :role="role" @click="onEvent">x</div>
  <div :role="enabled ? 'button' : undefined" :tabindex="enabled ? 0 : undefined" @click="onEvent" @keydown="onEvent">x</div>
  <div :role="enabled ? 'group' : 'img'" @click="onEvent">x</div>
  <div aria-hidden @click="onEvent">x</div>
  <div :aria-hidden="hidden" @click="onEvent">x</div>
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
                15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 40, 41, 42, 45, 48, 49,
                51, 55, 56, 57, 58
            ],
            "{:?}",
            result.diagnostics
        );
    }
}
