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
//! Only component listeners are checked. Native HTML elements use lowercase DOM
//! event names, and dynamic arguments (`@[event]`) cannot be statically checked.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <MyComponent @myEvent="handler" />
//! <MyComponent v-on:myEvent="handler" />
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
use vize_carton::hyphenate;
use vize_relief::{DirectiveNode, ElementNode, ElementType, ExpressionNode};

static META: RuleMeta = RuleMeta {
    name: "vue/v-on-event-hyphenation",
    description: "Enforce hyphenation of custom event names in v-on on components",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Enforce kebab-case custom event names in `v-on` on components.
pub struct VOnEventHyphenation;

impl VOnEventHyphenation {
    /// Whether `element` is a Vue component (and therefore receives custom,
    /// kebab-case events rather than native lowercase DOM events).
    ///
    /// The parser sets `tag_type == Component` for built-in components and any
    /// PascalCase tag; the explicit PascalCase check is a belt-and-braces guard
    /// so the rule still targets components even if the tag type is unset.
    fn is_component(element: &ElementNode<'_>) -> bool {
        element.tag_type == ElementType::Component
            || element
                .tag
                .as_str()
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_uppercase())
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
        if directive.name.as_str() != "on" {
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
            Some(ExpressionNode::Simple(s)) if s.is_static => s.content.as_str(),
            _ => return,
        };

        // camelCase if it contains an uppercase ASCII letter. (Native modifiers
        // are stripped into `directive.modifiers`, so the arg is the bare name.)
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
}

#[cfg(test)]
mod tests {
    use super::VOnEventHyphenation;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(VOnEventHyphenation));
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
}
