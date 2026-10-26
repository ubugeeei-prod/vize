//! vue/v-on-event-hyphenation
//!
//! Enforce hyphenation (kebab-case) of custom event names in `v-on`
//! listeners on Vue components.
//!
//! Vue templates are case-insensitive for HTML attributes, so a camelCase
//! listener such as `@myEvent` is parsed as `@myevent` and will never match an
//! event emitted as `my-event`. Hyphenating the listener (`@my-event`) keeps the
//! template consistent with the kebab-case form that components emit.
//!
//! Only custom component listeners are checked: a tag that is not a well-known
//! element of its namespace (`<MyComponent>`, `<my-component>`), or a native tag
//! cast with `is`. Native HTML elements use lowercase DOM event names, and
//! dynamic arguments (`@[event]`) cannot be statically checked.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <MyComponent @myEvent="handler" />
//! <MyComponent v-on:myEvent="handler" />
//! <my-component @update:modelValue="handler" />
//! ```
//!
//! ### Valid
//! ```vue
//! <MyComponent @my-event="handler" />
//! <div @myEvent="handler" />
//! <MyComponent @[dynamicEvent]="handler" />
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_l0::{hyphenate, is_html_tag, is_math_ml_tag, is_svg_tag};
use vize_relief::{DirectiveNode, ElementNode, ExpressionNode, Namespace, PropNode};

static META: RuleMeta = RuleMeta {
    name: "vue/v-on-event-hyphenation",
    description: "Enforce hyphenation of custom event names in v-on on components",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Event hyphenation style for `vue/v-on-event-hyphenation`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VOnEventHyphenationStyle {
    /// Require hyphenated event names.
    #[default]
    Always,
    /// Forbid hyphenated event names.
    Never,
}

/// Enforce custom event naming in `v-on` on components.
#[derive(Default)]
pub struct VOnEventHyphenation {
    style: VOnEventHyphenationStyle,
}

impl VOnEventHyphenation {
    pub fn new(style: VOnEventHyphenationStyle) -> Self {
        Self { style }
    }

    /// Whether `element` receives custom events rather than native DOM events.
    /// Matches eslint-plugin-vue's `isCustomComponent`: an `is` cast, or a tag
    /// that is not a well-known element of its namespace. A kebab-case tag such
    /// as `<va-select>` is a component even though the parser leaves it an
    /// element.
    fn is_component(element: &ElementNode<'_>) -> bool {
        if element.props.iter().any(is_cast) {
            return true;
        }
        // eslint-plugin-vue's element lists also know `<slot>` and a few newer
        // HTML elements, and `a`/`script`/`style` inside SVG.
        let tag = element.tag;
        match element.ns {
            Namespace::Html => {
                !is_html_tag(tag)
                    && !matches!(
                        tag,
                        "slot" | "fencedframe" | "geolocation" | "selectedcontent"
                    )
            }
            Namespace::Svg => !is_svg_tag(tag) && !matches!(tag, "a" | "script" | "style"),
            Namespace::MathMl => !is_math_ml_tag(tag),
        }
    }
}

/// `is="..."`, `:is="..."` or `v-is="..."`.
fn is_cast(prop: &PropNode<'_>) -> bool {
    match prop {
        PropNode::Attribute(attr) => attr.name == "is",
        PropNode::Directive(dir) => {
            dir.name == "is"
                || dir.name == "bind"
                    && matches!(
                        &dir.arg,
                        Some(ExpressionNode::Simple(arg)) if arg.is_static && arg.content == "is"
                    )
        }
    }
}

impl Rule for VOnEventHyphenation {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn check_directive<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        element: &ElementNode<'a>,
        directive: &DirectiveNode<'a>,
    ) {
        // Only v-on listeners.
        if directive.name != "on" {
            return;
        }

        // Only components receive custom (kebab-case) events; native HTML
        // elements use lowercase DOM event names.
        if !Self::is_component(element) {
            return;
        }

        // Read the static event-name argument. Dynamic arguments (`@[evt]`)
        // and object syntax (`v-on="..."`) cannot be checked statically.
        let event_name = match &directive.arg {
            Some(ExpressionNode::Simple(s)) if s.is_static => s.content,
            _ => return,
        };
        // `@vue:beforeMount` is a vnode lifecycle hook, not a component event.
        if event_name.starts_with("vue:") {
            return;
        }

        // camelCase if it contains an uppercase ASCII letter. (Native modifiers
        // are stripped into `directive.modifiers`, so the arg is the bare name.)
        match self.style {
            VOnEventHyphenationStyle::Always => {
                if !event_name.bytes().any(|b| b.is_ascii_uppercase()) {
                    return;
                }

                let hyphenated = hyphenate(event_name);
                ctx.warn_with_help(
                    ctx.t_fmt(
                        "vue/v-on-event-hyphenation.message",
                        &[("name", hyphenated.as_str())],
                    ),
                    &directive.loc,
                    ctx.t("vue/v-on-event-hyphenation.help"),
                );
            }
            VOnEventHyphenationStyle::Never => {
                if !event_name.contains('-') {
                    return;
                }

                ctx.warn_with_help(
                    ctx.t_fmt(
                        "vue/v-on-event-hyphenation.message_never",
                        &[("name", event_name)],
                    ),
                    &directive.loc,
                    ctx.t("vue/v-on-event-hyphenation.help_never"),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{VOnEventHyphenation, VOnEventHyphenationStyle};
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(VOnEventHyphenation::default()));
        Linter::with_registry(registry)
    }

    fn create_never_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(VOnEventHyphenation::new(
            VOnEventHyphenationStyle::Never,
        )));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_invalid_camel_case_on_component() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<MyComponent @myEvent="handler" />"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_camel_case_longform() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<MyComponent v-on:myEvent="handler" />"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_valid_kebab_case_on_component() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<MyComponent @my-event="handler" />"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_single_word_on_component() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<MyComponent @click="handler" />"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_native_element_is_skipped() {
        let linter = create_linter();
        // Native DOM events are lowercase; camelCase here is the user's concern,
        // not ours, so the rule must not flag native elements.
        let result = linter.lint_template(r#"<div @myEvent="handler" />"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_kebab_case_and_cast_components_are_checked() {
        let result = create_linter().lint_template(
            r#"<div><va-color-indicator @update:modelValue="a" /><table is="vue:my-row" @rowClick="b" /><button :is="c" @myEvent="d" /><svg><my-shape @shapeMove="e" /><circle @fooBar="f" /></svg><span @fooBar="g" /><slot @fooBar="h" /><svg><a @fooBar="i" /></svg></div>"#,
            "test.vue",
        );
        let reported: Vec<_> = result
            .diagnostics
            .iter()
            .map(|diagnostic| (diagnostic.start, diagnostic.end))
            .collect();
        // `@update:modelValue="a"`, `@rowClick="b"`, `@myEvent="d"`, `@shapeMove="e"`.
        assert_eq!(reported, [(25, 47), (73, 86), (105, 117), (135, 149)]);
    }

    #[test]
    fn test_sfc_kebab_case_component_listeners() {
        let result = create_linter().lint_sfc(
            include_str!(
                "../../../../../../tests/_fixtures/differential/linter/v-on-hyphenation-custom-tags-10822/ColorPalette.vue.txt"
            ),
            "ColorPalette.vue",
        );
        let reported: Vec<_> = result
            .diagnostics
            .iter()
            .map(|diagnostic| (diagnostic.rule_name, diagnostic.start, diagnostic.end))
            .collect();
        assert_eq!(
            reported,
            [
                ("vue/v-on-event-hyphenation", 135, 172),
                ("vue/v-on-event-hyphenation", 279, 311),
            ]
        );
    }

    #[test]
    fn test_dynamic_argument_is_skipped() {
        let linter = create_linter();
        let result =
            linter.lint_template(r#"<MyComponent @[dynamicEvent]="handler" />"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_object_syntax_is_skipped() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<MyComponent v-on="handlers" />"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_reports_kebab_case_form() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<MyComponent @fooBar="handler" />"#, "test.vue");
        assert_eq!(result.warning_count, 1);
        assert!(
            result.diagnostics[0].message.contains("foo-bar"),
            "message should suggest the kebab-case form, got: {}",
            result.diagnostics[0].message
        );
    }

    #[test]
    fn test_never_reports_hyphenated_event_on_component() {
        let result = create_never_linter().lint_template(
            r#"<MyComponent @my-event="handler" @myEvent="handler" />"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_vnode_lifecycle_hooks_are_skipped() {
        let always = create_linter().lint_template(
            r#"<MyComponent @vue:beforeMount="a" @vue:before-unmount="b" />"#,
            "test.vue",
        );
        assert_eq!(always.warning_count, 0);
        let never = create_never_linter().lint_template(
            r#"<MyComponent @vue:beforeMount="a" @vue:before-unmount="b" />"#,
            "test.vue",
        );
        assert_eq!(never.warning_count, 0);
    }

    #[test]
    fn test_never_skips_native_and_dynamic_arguments() {
        let result = create_never_linter().lint_template(
            r#"<div @my-event="handler" /><MyComponent @[myEvent]="handler" />"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }
}
