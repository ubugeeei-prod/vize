//! vue/scoped-event-names
//!
//! Recommend using scoped event names for better organization.
//!
//! When a component emits multiple related events, grouping them
//! with a scope prefix (e.g., `audio:play`, `audio:pause`) improves
//! readability and organization.
//!
//! This rule suggests using the `context:event` format when multiple
//! events share a common context.
//!
//! ## Examples
//!
//! ### Not Recommended
//! ```vue
//! <AudioPlayer @playAudio="play" @pauseAudio="pause" @reloadAudio="reload" />
//! ```
//!
//! ### Recommended
//! ```vue
//! <AudioPlayer @audio:play="play" @audio:pause="pause" @audio:reload="reload" />
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_croquis::builtins::is_builtin_component;
use vize_l0::cstr;
use vize_relief::{DirectiveNode, ElementNode, ElementType, ExpressionNode};

static META: RuleMeta = RuleMeta {
    name: "vue/scoped-event-names",
    description: "Recommend scoped event names using context:event format",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Known context prefixes that suggest grouping
const COMMON_SUFFIXES: &[&str] = &[
    "Audio", "Video", "Form", "Dialog", "Modal", "Menu", "List", "Item", "User", "Data", "File",
    "Image", "Input", "Select", "Button",
];

/// Recommend scoped event names
pub struct ScopedEventNames;

impl ScopedEventNames {
    /// Whether `element` has an author-owned component event surface.
    ///
    /// Native elements expose platform event names (`click`, `contextmenu`,
    /// `keyup`, ...), so they cannot adopt `context:event` names.
    /// Framework components own fixed contracts; a dynamic target can also
    /// resolve to a native element with platform-owned event names.
    fn has_author_owned_events(element: &ElementNode<'_>) -> bool {
        element.tag_type == ElementType::Component && !is_builtin_component(element.tag)
    }

    /// Check if an event name could benefit from scoping
    fn could_benefit_from_scope(event_name: &str) -> Option<&'static str> {
        for suffix in COMMON_SUFFIXES {
            let lower_suffix = suffix.to_lowercase();
            let lower_name = event_name.to_lowercase();

            // Check camelCase pattern (e.g., "playAudio")
            if lower_name.ends_with(&lower_suffix) && lower_name.len() > lower_suffix.len() {
                return Some(suffix);
            }
            // Check kebab-case pattern (e.g., "play-audio")
            if lower_name.ends_with(cstr!("-{}", lower_suffix).as_str()) {
                return Some(suffix);
            }
        }
        None
    }

    /// Check if event is already scoped
    fn is_scoped(event_name: &str) -> bool {
        event_name.contains(':')
    }
}

impl Rule for ScopedEventNames {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn check_directive<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        element: &ElementNode<'a>,
        directive: &DirectiveNode<'a>,
    ) {
        // Only check v-on directives
        if directive.name != "on" {
            return;
        }

        if !Self::has_author_owned_events(element) {
            return;
        }

        // Get the event name from the argument
        let event_name = match &directive.arg {
            Some(ExpressionNode::Simple(s)) if s.is_static => s.content,
            _ => return,
        };

        // Skip if already scoped
        if Self::is_scoped(event_name) {
            return;
        }

        // Check if this event could benefit from scoping
        if Self::could_benefit_from_scope(event_name).is_some() {
            ctx.warn_with_help(
                ctx.t("vue/scoped-event-names.message"),
                &directive.loc,
                ctx.t("vue/scoped-event-names.help"),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ScopedEventNames;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(ScopedEventNames));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_scoped_event() {
        let linter = create_linter();
        let result =
            linter.lint_template(r#"<button @audio:play="handlePlay"></button>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_warn_unscoped_event_with_suffix() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<AudioPlayer @playAudio="handlePlay" />"#, "test.vue");
        assert_eq!(result.warning_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn test_valid_simple_event() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<button @click="handleClick"></button>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_native_event_with_common_suffix_on_native_element() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<li @contextmenu.prevent="onMenu" @dblclick="onOpen">x</li>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn framework_and_dynamic_targets_do_not_own_event_naming() {
        let linter = create_linter();
        let source = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/_fixtures/differential/linter/dynamic-event-owner-11728/PlatformTargets.vue.txt"
        ));
        let result = linter.lint_sfc(source, "test.vue");
        assert_eq!(result.error_count, 0, "{source}: {:?}", result.diagnostics);
        assert_eq!(
            result.warning_count, 0,
            "{source}: {:?}",
            result.diagnostics
        );
        assert!(
            result.diagnostics.is_empty(),
            "{source}: {:?}",
            result.diagnostics
        );
        let authored = linter.lint_sfc(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/_fixtures/differential/linter/dynamic-event-owner-11728/AuthorOwned.vue.txt"
        )), "test.vue");
        assert_eq!(authored.warning_count, 1, "{:?}", authored.diagnostics);
        assert_eq!(
            authored
                .diagnostics
                .iter()
                .map(|diagnostic| diagnostic.rule_name)
                .collect::<Vec<_>>(),
            ["vue/scoped-event-names"]
        );
    }

    #[test]
    fn test_valid_update_event() {
        let linter = create_linter();
        let result =
            linter.lint_template(r#"<input @update:modelValue="handleUpdate" />"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }
}
