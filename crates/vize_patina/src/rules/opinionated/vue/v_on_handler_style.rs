//! vue/v-on-handler-style
//!
//! Prefer method handlers for `v-on` directives.
//!
//! A handler such as `@click="handler"` is directly reusable. Inline functions
//! are also accepted, while inline statements are reported.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <button @click="count++"></button>
//! <button @click="doThis(); doThat()"></button>
//! <button @click="foo = bar"></button>
//! ```
//!
//! ### Valid
//! ```vue
//! <button @click="handler"></button>
//! <button @click="foo.bar"></button>
//! <button @click="() => count++"></button>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use oxc_ast::ast::Expression as JsExpression;
use vize_relief::{
    DirectiveNode, ElementNode, ExpressionNode, SimpleExpressionNode, SourceLocation,
};

static META: RuleMeta = RuleMeta {
    name: "vue/v-on-handler-style",
    description: "Enforce writing v-on handlers as a method reference or an inline function",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Enforce v-on handler style (method reference or inline function).
pub struct VOnHandlerStyle;

impl Rule for VOnHandlerStyle {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn check_directive<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        _element: &ElementNode<'a>,
        directive: &DirectiveNode<'a>,
    ) {
        if directive.name != "on" {
            return;
        }

        // Both static and dynamic event arguments are checked. `v-on="obj"`
        // has no argument and remains outside this rule.
        let Some(_) = &directive.arg else {
            return;
        };

        let Some(exp) = &directive.exp else {
            return;
        };
        let (expr, loc, simple) = match exp {
            ExpressionNode::Simple(exp) => (exp.content.trim(), &exp.loc, Some(exp.as_ref())),
            // TypeScript-only template expressions such as
            // `(value: Model) => save(value)` are retained as compound nodes.
            // Their authored text is still sufficient for this syntactic rule.
            ExpressionNode::Compound(exp) => {
                let Some(authored) = ctx
                    .source
                    .get(exp.loc.span.start as usize..exp.loc.span.end as usize)
                else {
                    return;
                };
                (authored.trim(), &exp.loc, None)
            }
        };
        if expr.is_empty() {
            return;
        }

        if simple.is_some_and(retained_method_reference)
            || is_method_reference(expr)
            || is_typescript_asserted_method_reference(expr)
        {
            return;
        }

        if simple.is_some_and(retained_inline_function)
            || simple.is_none() && is_inline_function_text(expr)
        {
            return;
        }

        // The parser location includes whitespace inside the quotes, while
        // eslint-plugin-vue reports the JavaScript node itself.
        let loc = trimmed_expression_location(ctx.source, loc);

        ctx.warn_with_help(
            ctx.t("vue/v-on-handler-style.message"),
            &loc,
            ctx.t("vue/v-on-handler-style.help"),
        );
    }
}

fn retained_inline_function(expression: &SimpleExpressionNode<'_>) -> bool {
    expression
        .js_ast
        .and_then(|ast| ast.as_expression())
        .is_some_and(|parsed| {
            matches!(
                parsed.ast.get_inner_expression(),
                JsExpression::ArrowFunctionExpression(_) | JsExpression::FunctionExpression(_)
            )
        })
}

fn retained_method_reference(expression: &SimpleExpressionNode<'_>) -> bool {
    let Some(parsed) = expression.js_ast.and_then(|ast| ast.as_expression()) else {
        return false;
    };
    match parsed.ast.get_inner_expression() {
        JsExpression::Identifier(_) => true,
        JsExpression::ChainExpression(chain) => chain.expression.as_member_expression().is_some(),
        expression => expression.as_member_expression().is_some(),
    }
}

fn is_typescript_asserted_method_reference(expr: &str) -> bool {
    let Some((reference, assertion)) = expr.rsplit_once(" as ") else {
        return false;
    };
    !assertion.trim().is_empty() && is_method_reference(reference.trim())
}

fn is_inline_function_text(expr: &str) -> bool {
    expr.contains("=>") || expr.trim_start().starts_with("function")
}

/// Whether `expr` is a bare method reference: a single identifier or a
/// member-access path such as `foo` or `foo.bar`, with no call parentheses
/// or operators. Whitespace around `.` is ignored, so a member expression
/// wrapped across lines (`gestures.swipe\n.handleMove`) stays a reference.
/// The path must not start or end with `.` or contain an empty segment
/// (`foo..bar`).
fn is_method_reference(expr: &str) -> bool {
    let mut chars = expr.chars().peekable();
    let mut expect_segment = true;
    let mut saw_segment = false;
    while let Some(ch) = chars.peek().copied() {
        if ch.is_whitespace() {
            chars.next();
            continue;
        }
        if ch == '.' {
            if expect_segment {
                return false;
            }
            chars.next();
            expect_segment = true;
            continue;
        }
        if !expect_segment || !is_ident_start(ch) {
            return false;
        }
        chars.next();
        while chars.peek().copied().is_some_and(is_ident_continue) {
            chars.next();
        }
        saw_segment = true;
        expect_segment = false;
    }
    saw_segment && !expect_segment
}

fn is_ident_start(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch == '_' || ch == '$'
}

fn is_ident_continue(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_' || ch == '$'
}

fn trimmed_expression_location(source: &str, loc: &SourceLocation) -> SourceLocation {
    let Some(authored) = source.get(loc.span.start as usize..loc.span.end as usize) else {
        return loc.clone();
    };
    let start_trimmed = authored.trim_start();
    let leading = authored.len().saturating_sub(start_trimmed.len()) as u32;
    let trimmed = start_trimmed.trim_end();
    SourceLocation::new(
        loc.span.start + leading,
        loc.span.start + leading + trimmed.len() as u32,
    )
}

#[cfg(test)]
mod tests {
    use super::VOnHandlerStyle;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;
    use crate::{HelpLevel, Locale};

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(VOnHandlerStyle));
        Linter::with_registry(registry)
    }

    #[test]
    fn allows_method_reference() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<button @click="handler"></button>"#, "App.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn allows_member_access_reference() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<button @click="foo.bar"></button>"#, "App.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn allows_optional_member_reference() {
        let result =
            create_linter().lint_template(r#"<button @click="actions?.save"></button>"#, "App.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn allows_typescript_asserted_method_reference() {
        let result = create_linter().lint_template(
            r#"<button @click="actions.save as any"></button>"#,
            "App.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn allows_typescript_annotated_inline_function() {
        let result = create_linter().lint_template(
            r#"<button @click="(settings: Settings) => save(settings)"></button>"#,
            "App.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn allows_inline_function_that_forwards_a_scoped_slot_value() {
        let result = create_linter().lint_template(
            r#"<Widget v-slot="{ onSave }"><button @click="(value: Model) => onSave(value)" /></Widget>"#,
            "App.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn arrow_parameter_shadows_an_outer_template_variable() {
        let result = create_linter().lint_template(
            r#"<button v-for="column in columns" @click="(column) => save(column?.id)" />"#,
            "App.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn allows_arrow_function_that_can_be_a_method() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<button @click="() => count++"></button>"#, "App.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn allows_arrow_function_with_arg() {
        let linter = create_linter();
        let result =
            linter.lint_template(r#"<button @click="(e) => onClick(e)"></button>"#, "App.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn allows_function_expression() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<button @click="function () { count++ }"></button>"#,
            "App.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn warns_on_inline_increment() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<button @click="count++"></button>"#, "App.vue");
        assert_eq!(result.warning_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn short_help_is_distinct_and_actionable_in_every_locale() {
        for (locale, expected) in [
            (
                Locale::En,
                "Write the handler as a method reference or an inline function instead of an inline statement.",
            ),
            (
                Locale::Ja,
                "ハンドラをインライン文ではなく、メソッド参照またはインライン関数として記述してください。",
            ),
            (
                Locale::Zh,
                "请将处理函数写成方法引用或内联函数，而不是内联语句。",
            ),
        ] {
            let result = create_linter()
                .with_locale(locale)
                .with_help_level(HelpLevel::Short)
                .lint_template(r#"<button @click="count++"></button>"#, "App.vue");

            assert_eq!(result.diagnostics[0].help.as_deref(), Some(expected));
        }
    }

    #[test]
    fn warns_on_inline_call() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<button @click="foo()"></button>"#, "App.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn warns_on_assignment() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<button @click="a = b"></button>"#, "App.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn warns_on_multiple_statements() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<button @click="doThis(); doThat()"></button>"#,
            "App.vue",
        );
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn ignores_object_syntax() {
        let linter = create_linter();
        let result =
            linter.lint_template(r#"<button v-on="{ click: handler }"></button>"#, "App.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn ignores_handler_without_expression() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<form @submit.prevent></form>"#, "App.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn allows_member_access_split_across_lines() {
        let linter = create_linter();
        let result = linter.lint_template(
            "<div @touchmove=\"\n      gestures.swipe\n        .handleMove\n    \" />",
            "App.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn warns_when_whitespace_separates_two_identifiers() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<button @click="foo bar"></button>"#, "App.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn warns_on_member_call() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<button @click="foo.bar()"></button>"#, "App.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn reports_only_the_trimmed_handler_expression() {
        let source = "<button @click=\"\n  emit('save')\n\"></button>";
        let result = create_linter().lint_template(source, "App.vue");
        assert_eq!(result.warning_count, 1);
        let start = source.find("emit").unwrap() as u32;
        let end = start + "emit('save')".len() as u32;
        assert_eq!(
            (result.diagnostics[0].start, result.diagnostics[0].end),
            (start, end)
        );
    }

    #[test]
    fn checks_dynamic_event_arguments() {
        let result =
            create_linter().lint_template(r#"<button @[event]="save()"></button>"#, "App.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn allows_handler_that_closes_over_v_for_variables() {
        let result = create_linter().lint_template(
            r#"<div v-for="(item, index) in items"><button @click="(event) => save(event, item, index)" /></div>"#,
            "App.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn warns_on_inline_handler_that_uses_v_for_variables() {
        let result = create_linter().lint_template(
            r#"<div v-for="item in items"><button @click="save(item)" /></div>"#,
            "App.vue",
        );
        assert_eq!(result.warning_count, 1);
    }
}
