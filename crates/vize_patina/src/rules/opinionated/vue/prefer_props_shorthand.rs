//! vue/prefer-props-shorthand
//!
//! Recommend using shorthand syntax for props when the prop name matches the variable name.
//!
//! Vue 3.4+ supports shorthand syntax where `:foo="foo"` can be written as just `:foo`.
//! This makes the template more concise and easier to read.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <MyComponent :foo="foo" />
//! <MyComponent :user-name="userName" />
//! <MyComponent :count="count" :name="name" />
//! ```
//!
//! ### Valid
//! ```vue
//! <!-- Shorthand syntax (Vue 3.4+) -->
//! <MyComponent :foo />
//! <MyComponent :user-name />
//! <MyComponent :count :name />
//!
//! <!-- Different names are fine -->
//! <MyComponent :foo="bar" />
//! <MyComponent :count="totalCount" />
//! ```

use crate::context::LintContext;
use crate::diagnostic::{Fix, LintDiagnostic, Severity, TextEdit};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_croquis::naming::names_match;
use vize_relief::{ElementNode, ExpressionNode};

static META: RuleMeta = RuleMeta {
    name: "vue/prefer-props-shorthand",
    description: "Recommend shorthand syntax for props (Vue 3.4+)",
    category: RuleCategory::Recommended,
    fixable: true,
    default_severity: Severity::Warning,
};

/// Prefer props shorthand rule
#[derive(Default)]
pub struct PreferPropsShorthand;

impl Rule for PreferPropsShorthand {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        // Only check on component elements (PascalCase or kebab-case with -)
        let tag = element.tag;
        let is_component =
            tag.contains('-') || tag.chars().next().is_some_and(|c| c.is_uppercase());

        if !is_component {
            return;
        }

        for attr in &element.props {
            if let vize_relief::PropNode::Directive(dir) = attr
                && dir.name == "bind"
                && !dir.shorthand
                && let Some(arg) = &dir.arg
            {
                // Get the prop name
                let prop_name = match arg {
                    ExpressionNode::Simple(s) => s.content,
                    _ => continue,
                };

                // Get the expression value
                if let Some(exp) = &dir.exp {
                    let value = match exp {
                        ExpressionNode::Simple(s) => s.content.trim(),
                        _ => continue,
                    };

                    // Check if it's a simple identifier matching the prop name
                    let is_simple_identifier =
                        value.chars().all(|c: char| c.is_alphanumeric() || c == '_');

                    if is_simple_identifier && names_match(prop_name, value) {
                        let mut diagnostic = LintDiagnostic::warn(
                            ctx.current_rule,
                            ctx.t("vue/prefer-props-shorthand.message"),
                            dir.loc.span.start,
                            dir.loc.span.end,
                        );
                        let help = ctx.t("vue/prefer-props-shorthand.help");
                        if let Some(help) = ctx.help_level().process(help.as_ref()) {
                            diagnostic = diagnostic.with_help(help);
                        }
                        if let Some(fix) = same_name_shorthand_fix(
                            ctx.source,
                            dir.loc.span.start,
                            dir.loc.span.end,
                        ) {
                            diagnostic = diagnostic.with_fix(fix);
                        }
                        ctx.report(diagnostic);
                    }
                }
            }
        }
    }
}

/// Delete `="value"` from `:user-name="userName"`, leaving `:user-name`.
fn same_name_shorthand_fix(source: &str, start: u32, end: u32) -> Option<Fix> {
    let start = start as usize;
    let end = end as usize;
    let slice = source.get(start..end)?;
    let eq = slice.find('=')?;
    let mut trim = eq;
    while trim > 0
        && slice
            .as_bytes()
            .get(trim - 1)
            .is_some_and(u8::is_ascii_whitespace)
    {
        trim -= 1;
    }
    if trim == 0 {
        return None;
    }
    Some(Fix::new(
        "Use shorthand prop syntax",
        TextEdit::delete((start + trim) as u32, end as u32),
    ))
}

#[cfg(test)]
mod tests {
    use super::PreferPropsShorthand;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;
    use vize_l0::config::VueVersion;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(PreferPropsShorthand));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_shorthand() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<MyComponent :foo />"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_different_names() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<MyComponent :foo="bar" />"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_same_name() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<MyComponent :foo="foo" />"#, "test.vue");
        assert_eq!(result.warning_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn test_vue2_compatibility_disables_vue34_shorthand() {
        let linter = create_linter().with_vue_version(Some(VueVersion::V2_7));
        let result = linter.lint_template(r#"<MyComponent :foo="foo" />"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn same_name_binding_emits_a_shorthand_fix() {
        let linter = create_linter();
        let source = r#"<MyComponent :foo="foo" />"#;
        let result = linter.lint_template(source, "test.vue");
        let fix = result.diagnostics[0].fix.as_ref().expect("shorthand fix");
        assert_eq!(fix.apply(source), r#"<MyComponent :foo />"#);

        let source = r#"<UserAvatar :user-name="userName" />"#;
        let result = linter.lint_template(source, "UserCard.vue");
        let fix = result.diagnostics[0].fix.as_ref().expect("shorthand fix");
        assert_eq!(fix.apply(source), r#"<UserAvatar :user-name />"#);
    }

    #[test]
    fn test_invalid_kebab_camel_match() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<MyComponent :user-name="userName" />"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_valid_expression() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<MyComponent :foo="foo + bar" />"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_camelize() {
        // Test via vize_l0::camelize (used internally by names_match)
        use vize_l0::camelize;
        assert_eq!(camelize("user-name").as_str(), "userName");
        assert_eq!(camelize("foo-bar-baz").as_str(), "fooBarBaz");
        assert_eq!(camelize("simple").as_str(), "simple");
    }
}
