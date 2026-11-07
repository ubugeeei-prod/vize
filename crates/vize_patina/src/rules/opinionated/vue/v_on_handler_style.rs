//! vue/v-on-handler-style
//!
//! Prefer method handlers for `v-on` directives.
//!
//! A handler such as `@click="handler"` is directly reusable. Inline statements
//! and inline functions are reported when they can be moved to a method.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <button @click="count++"></button>
//! <button @click="doThis(); doThat()"></button>
//! <button @click="foo = bar"></button>
//! <button @click="() => count++"></button>
//! ```
//!
//! ### Valid
//! ```vue
//! <button @click="handler"></button>
//! <button @click="foo.bar"></button>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use oxc_ast::ast::{Argument, BindingPattern, Expression as JsExpression, IdentifierReference};
use oxc_ast_visit::Visit;
use vize_l0::CompactString;
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

        if simple.is_some_and(retained_method_reference) || is_method_reference(expr) {
            return;
        }

        // Moving an inline function that closes over the current/parent v-for
        // scope to a component method would lose those template-local
        // bindings. An inline handler can instead be wrapped in a function, so
        // it remains reportable.
        if simple.is_some_and(retained_inline_function)
            && !simple.is_some_and(inline_function_forwards_parameters)
            && simple.is_some_and(|exp| references_v_for_variable(ctx, exp))
            || simple.is_none()
                && is_inline_function_text(expr)
                && !text_inline_function_forwards_parameters(expr)
                && text_references_v_for_variable(ctx, expr)
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

fn references_v_for_variable(ctx: &LintContext<'_>, expression: &SimpleExpressionNode<'_>) -> bool {
    let Some(parsed) = expression.js_ast.and_then(|ast| ast.as_expression()) else {
        return false;
    };
    let ignored = inline_function_parameter_names(parsed.ast);
    let mut visitor = VForReferenceVisitor {
        ctx,
        ignored,
        found: false,
    };
    visitor.visit_expression(parsed.ast);
    visitor.found
}

struct VForReferenceVisitor<'ctx, 'source> {
    ctx: &'ctx LintContext<'source>,
    ignored: Vec<CompactString>,
    found: bool,
}

impl<'a> Visit<'a> for VForReferenceVisitor<'_, '_> {
    fn visit_identifier_reference(&mut self, identifier: &IdentifierReference<'a>) {
        self.found |= !self
            .ignored
            .iter()
            .any(|name| name.as_str() == identifier.name.as_str())
            && self.ctx.is_v_for_var(identifier.name.as_str());
    }
}

fn inline_function_parameter_names(expression: &JsExpression<'_>) -> Vec<CompactString> {
    let params = match expression.get_inner_expression() {
        JsExpression::ArrowFunctionExpression(function) => &function.params,
        JsExpression::FunctionExpression(function) => &function.params,
        _ => return Vec::new(),
    };
    params
        .items
        .iter()
        .filter_map(|parameter| match &parameter.pattern {
            BindingPattern::BindingIdentifier(identifier) => {
                Some(CompactString::new(identifier.name.as_str()))
            }
            _ => None,
        })
        .collect()
}

fn inline_function_forwards_parameters(expression: &SimpleExpressionNode<'_>) -> bool {
    let Some(parsed) = expression.js_ast.and_then(|ast| ast.as_expression()) else {
        return false;
    };
    let JsExpression::ArrowFunctionExpression(function) = parsed.ast.get_inner_expression() else {
        return false;
    };
    let Some(call) = function
        .expression
        .then(|| function.body.statements.first())
        .flatten()
        .and_then(|statement| {
            let oxc_ast::ast::Statement::ExpressionStatement(statement) = statement else {
                return None;
            };
            let JsExpression::CallExpression(call) = statement.expression.get_inner_expression()
            else {
                return None;
            };
            Some(call)
        })
    else {
        return false;
    };
    matches!(
        call.callee.get_inner_expression(),
        JsExpression::Identifier(_)
    ) && function.params.items.len() == call.arguments.len()
        && function
            .params
            .items
            .iter()
            .zip(&call.arguments)
            .all(|(param, argument)| {
                let BindingPattern::BindingIdentifier(param) = &param.pattern else {
                    return false;
                };
                let Argument::Identifier(argument) = argument else {
                    return false;
                };
                param.name == argument.name
            })
}

fn retained_method_reference(expression: &SimpleExpressionNode<'_>) -> bool {
    let Some(parsed) = expression.js_ast.and_then(|ast| ast.as_expression()) else {
        return false;
    };
    match parsed.ast {
        JsExpression::Identifier(_) => true,
        // Optional member access (`actions?.save`) is not a stable method
        // reference: it conditionally evaluates to `undefined`, and upstream
        // classifies it as an inline handler.
        JsExpression::ChainExpression(_) => false,
        expression => expression.as_member_expression().is_some(),
    }
}

fn is_inline_function_text(expr: &str) -> bool {
    expr.contains("=>") || expr.trim_start().starts_with("function")
}

fn text_references_v_for_variable(ctx: &LintContext<'_>, expr: &str) -> bool {
    let ignored = text_inline_function_parameters(expr);
    let mut start = None;
    for (index, ch) in expr.char_indices() {
        if start.is_none() {
            if is_ident_start(ch) {
                start = Some(index);
            }
        } else if !is_ident_continue(ch) {
            let from = start.take().unwrap_or(index);
            let Some(name) = expr.get(from..index) else {
                continue;
            };
            if !ignored.contains(&name) && ctx.is_v_for_var(name) {
                return true;
            }
        }
    }
    start.is_some_and(|from| {
        expr.get(from..)
            .is_some_and(|name| !ignored.contains(&name) && ctx.is_v_for_var(name))
    })
}

fn text_inline_function_parameters(expr: &str) -> Vec<&str> {
    let Some((params, _)) = expr.split_once("=>") else {
        return Vec::new();
    };
    let params = params.trim().trim_start_matches('(').trim_end_matches(')');
    params
        .split(',')
        .filter_map(|param| {
            let name = param.split_once(':').map_or(param, |(name, _)| name).trim();
            let name = name.trim_end_matches('?');
            (!name.is_empty() && name.chars().all(is_ident_continue)).then_some(name)
        })
        .collect()
}

fn text_inline_function_forwards_parameters(expr: &str) -> bool {
    let Some((_, body)) = expr.split_once("=>") else {
        return false;
    };
    let params = text_inline_function_parameters(expr);
    let body = body.trim();
    let Some(open) = body.find('(') else {
        return false;
    };
    let Some(method) = body.get(..open) else {
        return false;
    };
    if !is_method_reference(method.trim()) || !body.ends_with(')') {
        return false;
    }
    let Some(arguments) = body.get(open + 1..body.len() - 1) else {
        return false;
    };
    let arguments: Vec<_> = arguments.split(',').map(str::trim).collect();
    params == arguments
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
    fn warns_on_optional_member_reference() {
        let result =
            create_linter().lint_template(r#"<button @click="actions?.save"></button>"#, "App.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn warns_on_typescript_annotated_inline_function() {
        let result = create_linter().lint_template(
            r#"<button @click="(settings: Settings) => save(settings)"></button>"#,
            "App.vue",
        );
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn warns_when_inline_function_forwards_a_scoped_slot_value() {
        let result = create_linter().lint_template(
            r#"<Widget v-slot="{ onSave }"><button @click="(value: Model) => onSave(value)" /></Widget>"#,
            "App.vue",
        );
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn arrow_parameter_shadows_an_outer_template_variable() {
        let result = create_linter().lint_template(
            r#"<button v-for="column in columns" @click="(column) => save(column?.id)" />"#,
            "App.vue",
        );
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn warns_on_arrow_function_that_can_be_a_method() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<button @click="() => count++"></button>"#, "App.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn warns_on_arrow_function_with_arg() {
        let linter = create_linter();
        let result =
            linter.lint_template(r#"<button @click="(e) => onClick(e)"></button>"#, "App.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn warns_on_function_expression() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<button @click="function () { count++ }"></button>"#,
            "App.vue",
        );
        assert_eq!(result.warning_count, 1);
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
