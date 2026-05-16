//! vue/no-reserved-component-names
//!
//! Disallow the use of reserved names as component names.
//!
//! HTML element names, SVG element names, and Vue built-in component names
//! should not be used as component names.
//!
//! This rule checks the **component definition** (filename), NOT the names
//! of other components used in the template. This matches the behavior of
//! eslint-plugin-vue. Using `<Transition>` or `<KeepAlive>` in a template
//! is perfectly valid — they are Vue built-in components being used correctly.
//!
//! ## Examples
//!
//! ### Invalid (filename)
//! ```text
//! Div.vue
//! Slot.vue
//! Transition.vue (shadows Vue built-in)
//! ```
//!
//! ### Valid (filename)
//! ```text
//! MyComponent.vue
//! AppHeader.vue
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_carton::is_html_tag;
use vize_croquis::builtins::is_builtin_component;
use vize_relief::ast::RootNode;

static META: RuleMeta = RuleMeta {
    name: "vue/no-reserved-component-names",
    description: "Disallow the use of reserved names as component names",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Error,
};

/// Reserved names that cannot be used (specific edge cases)
const RESERVED_NAMES: &[&str] = &[
    "annotation-xml",
    "color-profile",
    "font-face",
    "font-face-src",
    "font-face-uri",
    "font-face-format",
    "font-face-name",
    "missing-glyph",
];

/// Disallow reserved component names
pub struct NoReservedComponentNames {
    /// Also disallow HTML element names
    pub disallow_html: bool,
    /// Also disallow Vue built-ins
    pub disallow_vue_builtins: bool,
}

impl Default for NoReservedComponentNames {
    fn default() -> Self {
        Self {
            disallow_html: true,
            disallow_vue_builtins: true,
        }
    }
}

impl NoReservedComponentNames {
    /// Extract the component name from a filename.
    fn extract_component_name(filename: &str) -> Option<&str> {
        let basename = filename.rsplit('/').next().unwrap_or(filename);
        let basename = basename.rsplit('\\').next().unwrap_or(basename);
        basename.strip_suffix(".vue")
    }
}

impl Rule for NoReservedComponentNames {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_template<'a>(&self, ctx: &mut LintContext<'a>, root: &RootNode<'a>) {
        let filename = ctx.filename;

        // Only check .vue files
        let Some(component_name) = Self::extract_component_name(filename) else {
            return;
        };

        let name_lower = component_name.to_lowercase();

        // Check against reserved names
        if RESERVED_NAMES.contains(&name_lower.as_str()) {
            ctx.error_with_help(
                ctx.t_fmt(
                    "vue/no-reserved-component-names.message",
                    &[("name", component_name)],
                ),
                &root.loc,
                ctx.t("vue/no-reserved-component-names.help"),
            );
            return;
        }

        // Check against HTML elements
        if self.disallow_html && is_html_tag(&name_lower) {
            ctx.error_with_help(
                ctx.t_fmt(
                    "vue/no-reserved-component-names.message",
                    &[("name", component_name)],
                ),
                &root.loc,
                ctx.t("vue/no-reserved-component-names.help"),
            );
            return;
        }

        // Check against Vue built-ins
        if self.disallow_vue_builtins
            && (is_builtin_component(&name_lower) || is_builtin_component(component_name))
        {
            ctx.error_with_help(
                ctx.t_fmt(
                    "vue/no-reserved-component-names.message",
                    &[("name", component_name)],
                ),
                &root.loc,
                ctx.t("vue/no-reserved-component-names.help"),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::NoReservedComponentNames;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(NoReservedComponentNames::default()));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_custom_component() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div>hello</div>"#, "MyComponent.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_invalid_html_name() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div>hello</div>"#, "Div.vue");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_vue_builtin() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div>hello</div>"#, "Transition.vue");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_using_transition_in_template_is_valid() {
        let linter = create_linter();
        // Using <Transition> in a template is CORRECT usage of Vue built-in
        let result = linter.lint_template(
            r#"<Transition name="fade"><div>hello</div></Transition>"#,
            "MyComponent.vue",
        );
        assert_eq!(
            result.error_count, 0,
            "Using Vue built-in <Transition> in template should not be flagged"
        );
    }

    #[test]
    fn test_using_keep_alive_in_template_is_valid() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<KeepAlive><div>hello</div></KeepAlive>"#,
            "MyComponent.vue",
        );
        assert_eq!(
            result.error_count, 0,
            "Using Vue built-in <KeepAlive> in template should not be flagged"
        );
    }

    #[test]
    fn test_using_teleport_in_template_is_valid() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<Teleport to="body"><div>hello</div></Teleport>"#,
            "MyComponent.vue",
        );
        assert_eq!(
            result.error_count, 0,
            "Using Vue built-in <Teleport> in template should not be flagged"
        );
    }

    #[test]
    fn test_using_suspense_in_template_is_valid() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<Suspense><div>hello</div></Suspense>"#,
            "MyComponent.vue",
        );
        assert_eq!(
            result.error_count, 0,
            "Using Vue built-in <Suspense> in template should not be flagged"
        );
    }

    #[test]
    fn test_non_vue_file() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div>hello</div>"#, "test.html");
        assert_eq!(result.error_count, 0);
    }
}
