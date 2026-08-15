//! vue/valid-v-for
//!
//! Enforce valid `v-for` directives.
//!
//! This rule checks the following:
//! - `v-for` directive has an expression
//! - `v-for` directive's expression is valid (contains "in" or "of")
//! - `v-for` directive doesn't have invalid modifiers
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <div v-for></div>
//! <div v-for=""></div>
//! <div v-for.stop="item in items"></div>
//! ```
//!
//! ### Valid
//! ```vue
//! <div v-for="item in items" :key="item.id"></div>
//! <div v-for="(item, index) in items" :key="index"></div>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use crate::visitor::parse_v_for_variables;
use vize_relief::{DirectiveNode, ElementNode, ExpressionNode, PropNode};

static META: RuleMeta = RuleMeta {
    name: "vue/valid-v-for",
    description: "Enforce valid `v-for` directives",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Error,
};

/// Enforce valid v-for directives
pub struct ValidVFor;

impl Rule for ValidVFor {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn check_directive<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        element: &ElementNode<'a>,
        directive: &DirectiveNode<'a>,
    ) {
        // Only check v-for directives
        if directive.name != "for" {
            return;
        }

        // Check for modifiers (v-for should not have modifiers)
        if !directive.modifiers.is_empty() {
            ctx.error_with_help(
                ctx.t("vue/valid-v-for.missing_expression"),
                &directive.loc,
                ctx.t("vue/valid-v-for.help"),
            );
            return;
        }

        // Check for argument (v-for should not have an argument like v-for:something)
        if directive.arg.is_some() {
            ctx.error_with_help(
                ctx.t("vue/valid-v-for.missing_expression"),
                &directive.loc,
                ctx.t("vue/valid-v-for.help"),
            );
            return;
        }

        // Check for expression
        match &directive.exp {
            None => {
                ctx.error_with_help(
                    ctx.t("vue/valid-v-for.missing_expression"),
                    &directive.loc,
                    ctx.t("vue/valid-v-for.help"),
                );
            }
            Some(exp) => {
                // Validate the expression format
                let content = match exp {
                    ExpressionNode::Simple(s) => s.content,
                    ExpressionNode::Compound(_) => return, // Complex expressions are harder to validate
                };

                let trimmed = content.trim();

                // Check if empty
                if trimmed.is_empty() {
                    ctx.error_with_help(
                        ctx.t("vue/valid-v-for.missing_expression"),
                        &directive.loc,
                        ctx.t("vue/valid-v-for.help"),
                    );
                    return;
                }

                // Check for "in" or "of" keyword
                let has_in = trimmed.contains(" in ");
                let has_of = trimmed.contains(" of ");

                if !has_in && !has_of {
                    ctx.error_with_help(
                        ctx.t("vue/valid-v-for.invalid_syntax"),
                        &directive.loc,
                        ctx.t("vue/valid-v-for.help"),
                    );
                    return;
                }

                // Validate alias part (left side of in/of)
                let (alias_part, source_part) = if has_in {
                    if let Some(idx) = trimmed.find(" in ") {
                        (&trimmed[..idx], &trimmed[idx + 4..])
                    } else {
                        ctx.error_with_help(
                            ctx.t("vue/valid-v-for.invalid_syntax"),
                            &directive.loc,
                            ctx.t("vue/valid-v-for.help"),
                        );
                        return;
                    }
                } else if let Some(idx) = trimmed.find(" of ") {
                    (&trimmed[..idx], &trimmed[idx + 4..])
                } else {
                    ctx.error_with_help(
                        ctx.t("vue/valid-v-for.invalid_syntax"),
                        &directive.loc,
                        ctx.t("vue/valid-v-for.help"),
                    );
                    return;
                };

                let alias = alias_part.trim();
                let source = source_part.trim();

                // Check alias is not empty
                if alias.is_empty() {
                    ctx.error_with_help(
                        ctx.t("vue/valid-v-for.invalid_syntax"),
                        &directive.loc,
                        ctx.t("vue/valid-v-for.help"),
                    );
                    return;
                }

                // Check source is not empty
                if source.is_empty() {
                    ctx.error_with_help(
                        ctx.t("vue/valid-v-for.invalid_syntax"),
                        &directive.loc,
                        ctx.t("vue/valid-v-for.help"),
                    );
                    return;
                }

                check_key_uses_v_for_variables(ctx, element, exp);
            }
        }
    }
}

fn check_key_uses_v_for_variables(
    ctx: &mut LintContext<'_>,
    element: &ElementNode<'_>,
    v_for_exp: &ExpressionNode<'_>,
) {
    let Some(key_directive) = bound_key_directive(element) else {
        return;
    };
    let Some(ExpressionNode::Simple(key_expression)) = &key_directive.exp else {
        return;
    };

    let vars = parse_v_for_variables(v_for_exp);
    if vars.is_empty() {
        return;
    }
    if vars
        .iter()
        .any(|var| expression_references_identifier(key_expression.content, var.as_str()))
    {
        return;
    }

    ctx.error_with_help(
        ctx.t("vue/valid-v-for.key_uses_variables"),
        &key_directive.loc,
        ctx.t("vue/valid-v-for.help"),
    );
}

fn bound_key_directive<'a>(element: &'a ElementNode<'a>) -> Option<&'a DirectiveNode<'a>> {
    element.props.iter().find_map(|prop| match prop {
        PropNode::Directive(dir)
            if dir.name == "bind"
                && matches!(
                    dir.arg.as_ref(),
                    Some(ExpressionNode::Simple(arg)) if arg.content == "key"
                ) =>
        {
            Some(dir.as_ref())
        }
        _ => None,
    })
}

fn expression_references_identifier(expression: &str, name: &str) -> bool {
    if name.is_empty() || expression.is_empty() {
        return false;
    }
    let bytes = expression.as_bytes();
    let needle = name.as_bytes();
    let mut i = 0;
    while i + needle.len() <= bytes.len() {
        if bytes[i..i + needle.len()] == *needle {
            let prev_is_ident = i > 0 && is_ident_byte(bytes[i - 1]);
            let next_is_ident = bytes
                .get(i + needle.len())
                .is_some_and(|byte| is_ident_byte(*byte));
            if !prev_is_ident && !next_is_ident {
                return true;
            }
        }
        i += 1;
    }
    false
}

#[inline]
fn is_ident_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'$'
}

#[cfg(test)]
mod key_tests;

#[cfg(test)]
mod tests {
    use super::ValidVFor;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(ValidVFor));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_v_for() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div v-for="item in items" :key="item.id"></div>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_v_for_with_index() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div v-for="(item, index) in items" :key="index"></div>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_v_for_of() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div v-for="item of items" :key="item.id"></div>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_invalid_v_for_no_expression() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div v-for></div>"#, "test.vue");
        assert_eq!(result.error_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn test_invalid_v_for_empty_expression() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div v-for=""></div>"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_v_for_no_in_or_of() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div v-for="items"></div>"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }
}
