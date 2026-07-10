//! vue/valid-v-else
//!
//! Enforce valid `v-else` directives.
//!
//! `v-else` must:
//! - Be on an element immediately following a `v-if` or `v-else-if` element
//! - Not have an expression
//! - Not be used with `v-if` or `v-else-if` on the same element
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <div v-else="foo"></div>
//! <div v-else v-if="bar"></div>
//! <div v-else></div> <!-- without preceding v-if -->
//! ```
//!
//! ### Valid
//! ```vue
//! <div v-if="foo"></div>
//! <div v-else></div>
//! ```

use crate::context::LintContext;
use crate::diagnostic::{Fix, Severity, TextEdit};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::{
    DirectiveNode, ElementNode, PropNode, RootNode, SourceLocation, TemplateChildNode,
};

static META: RuleMeta = RuleMeta {
    name: "vue/valid-v-else",
    description: "Enforce valid `v-else` directives",
    category: RuleCategory::Essential,
    fixable: true,
    default_severity: Severity::Error,
};

/// Enforce valid v-else directives
pub struct ValidVElse;

impl Rule for ValidVElse {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_template<'a>(&self, ctx: &mut LintContext<'a>, root: &RootNode<'a>) {
        check_adjacent_if_chain(ctx, &root.children);
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        check_adjacent_if_chain(ctx, &element.children);
    }

    fn check_directive<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        element: &ElementNode<'a>,
        directive: &DirectiveNode<'a>,
    ) {
        if directive.name.as_str() != "else" {
            return;
        }

        // Check 1: v-else should not have an expression
        if directive.exp.is_some() {
            let fix = Fix::new(
                "Remove the expression from v-else",
                TextEdit::delete(directive.loc.start.offset, directive.loc.end.offset),
            );
            ctx.report(
                crate::diagnostic::LintDiagnostic::error(
                    META.name,
                    ctx.t("vue/valid-v-else.unexpected_expression").as_ref(),
                    directive.loc.start.offset,
                    directive.loc.end.offset,
                )
                .with_help(ctx.t("vue/valid-v-else.help").as_ref())
                .with_fix(fix),
            );
        }

        // Check 2: v-else should not be used with v-if or v-else-if
        let has_v_if = element.props.iter().any(|p| {
            matches!(p, PropNode::Directive(d) if d.name.as_str() == "if" || d.name.as_str() == "else-if")
        });
        if has_v_if {
            ctx.error_with_help(
                ctx.t("vue/valid-v-else.missing_v_if"),
                &directive.loc,
                ctx.t("vue/valid-v-else.help"),
            );
        }
    }
}

struct IfChainDirectiveInfo {
    has_v_if: bool,
    else_if_loc: Option<SourceLocation>,
    else_loc: Option<SourceLocation>,
}

fn check_adjacent_if_chain(ctx: &mut LintContext, children: &[TemplateChildNode]) {
    let mut can_follow_if = false;

    for child in children {
        match child {
            TemplateChildNode::Element(element) => {
                let info = get_if_chain_directive_info(element);

                if info.has_v_if {
                    can_follow_if = true;
                    continue;
                }

                if let Some(loc) = info.else_if_loc {
                    if !can_follow_if {
                        report_missing_adjacent_if(ctx, &loc);
                    }
                    continue;
                }

                if let Some(loc) = info.else_loc {
                    if !can_follow_if {
                        report_missing_adjacent_if(ctx, &loc);
                    }
                    can_follow_if = false;
                    continue;
                }

                can_follow_if = false;
            }
            TemplateChildNode::Text(text) if text.content.trim().is_empty() => {}
            TemplateChildNode::Comment(_) => {}
            _ => {
                can_follow_if = false;
            }
        }
    }
}

fn get_if_chain_directive_info(element: &ElementNode) -> IfChainDirectiveInfo {
    let mut info = IfChainDirectiveInfo {
        has_v_if: false,
        else_if_loc: None,
        else_loc: None,
    };

    for prop in element.props.iter() {
        if let PropNode::Directive(dir) = prop {
            match dir.name.as_str() {
                "if" => info.has_v_if = true,
                "else-if" => info.else_if_loc = Some(dir.loc.clone()),
                "else" => info.else_loc = Some(dir.loc.clone()),
                _ => {}
            }
        }
    }

    info
}

fn report_missing_adjacent_if(ctx: &mut LintContext, loc: &SourceLocation) {
    ctx.error_with_help(
        ctx.t("vue/valid-v-else.missing_v_if"),
        loc,
        ctx.t("vue/valid-v-else.help"),
    );
}

#[cfg(test)]
mod tests {
    use super::ValidVElse;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(ValidVElse));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_v_else() {
        let linter = create_linter();
        let result =
            linter.lint_template(r#"<div v-if="foo"></div><div v-else></div>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_v_else_if_chain() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div v-if="foo"></div><div v-else-if="bar"></div><div v-else></div>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_v_else_after_comment() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div v-if="foo"></div><!-- fallback branch --><div v-else></div>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_invalid_v_else_with_expression() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div v-if="foo"></div><div v-else="bar"></div>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn test_invalid_v_else_with_v_if() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div v-if="foo" v-else></div>"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_v_else_without_adjacent_v_if() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div v-else></div>"#, "test.vue");
        assert_eq!(result.error_count, 1);
        assert_eq!(result.diagnostics[0].rule_name, "vue/valid-v-else");
    }

    #[test]
    fn test_invalid_v_else_if_without_adjacent_v_if() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div v-else-if="foo"></div>"#, "test.vue");
        assert_eq!(result.error_count, 1);
        assert_eq!(result.diagnostics[0].rule_name, "vue/valid-v-else");
    }

    #[test]
    fn test_invalid_v_else_after_text_gap() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div v-if="foo"></div> text <div v-else></div>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 1);
    }
}
