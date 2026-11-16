//! eslint-plugin-vue's handler styles, selected through `linter.ruleOptions`.
//!
//! A handler is a method reference (`handler`), an inline function
//! (`() => handler()`), or an inline handler (`handler()`, `count++`). The
//! preferred style is never reported. With `"method"` first, an inline handler
//! or function is reported as a method candidate unless it calls something
//! other than one identifier with exactly its own parameters and reads a
//! `v-for` or slot variable; such a handler falls to the second style.
//!
//! A member reference such as `foo.bar` or `foo?.bar` is never reported, as
//! upstream leaves `foo.bar` alone.

use oxc_allocator::Allocator;
use oxc_ast::ast::{
    Argument, BindingPattern, Expression, FormalParameters, FunctionBody, IdentifierReference,
    Statement, TSType,
};
use oxc_ast_visit::Visit;
use oxc_parser::{Parser, ParserReturn};
use oxc_semantic::{Semantic, SemanticBuilder};
use oxc_span::SourceType;
use vize_l0::config::{VOnHandlerAllowed, VOnHandlerKind};
use vize_l0::{String, cstr};

use crate::context::LintContext;

/// The parsed facts the style decision reads.
pub(super) struct Handler {
    /// `Some(parameter count)` for an inline function.
    function_params: Option<usize>,
    /// The single `identifier(...)` call the handler consists of.
    call: Option<IdentifierCall>,
    pub(super) has_comment: bool,
    /// Names the handler reads or writes without declaring them.
    free_names: Vec<String>,
}

struct IdentifierCall {
    arguments: usize,
    /// Every argument is the same-named parameter, in order.
    forwards_params: bool,
}

/// A report: the message key and its help key.
pub(super) type Report = (&'static str, &'static str);

const HELP_METHOD: &str = "vue/v-on-handler-style.help";
const HELP_INLINE: &str = "vue/v-on-handler-style.help_inline";
const HELP_INLINE_FUNCTION: &str = "vue/v-on-handler-style.help_inline_function";

/// Decide a method reference (`handler`).
pub(super) fn method_reference(allowed: VOnHandlerAllowed) -> Option<Report> {
    match allowed.preferred() {
        VOnHandlerKind::Method => None,
        VOnHandlerKind::Inline => Some((
            "vue/v-on-handler-style.prefer_inline_over_method",
            HELP_INLINE,
        )),
        VOnHandlerKind::InlineFunction => Some((
            "vue/v-on-handler-style.prefer_inline_function_over_method",
            HELP_INLINE_FUNCTION,
        )),
    }
}

/// Decide an inline handler or inline function.
pub(super) fn handler(
    ctx: &LintContext<'_>,
    allowed: VOnHandlerAllowed,
    ignore_includes_comment: bool,
    handler: &Handler,
) -> Option<Report> {
    let own = if handler.function_params.is_some() {
        VOnHandlerKind::InlineFunction
    } else {
        VOnHandlerKind::Inline
    };
    if allowed.preferred() == own {
        return None;
    }
    // The handler's own style, allowed second, ends the search unreported.
    for style in [Some(allowed.preferred()), allowed.fallback()]
        .into_iter()
        .flatten()
        .filter(|style| *style != own)
    {
        let report = match style {
            VOnHandlerKind::Method => method_candidate(ctx, ignore_includes_comment, handler),
            VOnHandlerKind::Inline => Some(match handler.function_params {
                Some(params) if params > 1 => (
                    "vue/v-on-handler-style.prefer_inline_over_inline_function_with_multiple_params",
                    HELP_INLINE,
                ),
                _ => (
                    "vue/v-on-handler-style.prefer_inline_over_inline_function",
                    HELP_INLINE,
                ),
            }),
            VOnHandlerKind::InlineFunction => Some((
                "vue/v-on-handler-style.prefer_inline_function_over_inline",
                HELP_INLINE_FUNCTION,
            )),
        };
        if report.is_some() {
            return report;
        }
    }
    None
}

/// Upstream's `verifyCanUseMethodHandlerFor*`: `None` lets the next style decide.
fn method_candidate(
    ctx: &LintContext<'_>,
    ignore_includes_comment: bool,
    handler: &Handler,
) -> Option<Report> {
    if ignore_includes_comment && handler.has_comment {
        return None;
    }
    let plain_call = handler.call.as_ref().is_some_and(|call| {
        if handler.function_params.is_some() {
            call.forwards_params
        } else {
            call.arguments == 0
        }
    });
    if !plain_call && handler.free_names.iter().any(|name| ctx.is_v_for_var(name)) {
        return None;
    }
    let key = match (handler.function_params.is_some(), handler.call.is_some()) {
        (false, true) => "vue/v-on-handler-style.prefer_method_over_inline",
        (false, false) => "vue/v-on-handler-style.prefer_method_over_inline_without_id_call",
        (true, true) => "vue/v-on-handler-style.prefer_method_over_inline_function",
        (true, false) => {
            "vue/v-on-handler-style.prefer_method_over_inline_function_without_id_call"
        }
    };
    Some((key, HELP_METHOD))
}

/// Parse a handler that is not a plain reference. Unparsable text yields
/// `None`; the template compiler reports it.
///
/// The retained `js_ast` carries neither scopes nor comments, and a statement
/// list or a typed parameter is not retained at all, so the handler text is
/// parsed here; only an opted-in upstream style reaches this.
pub(super) fn analyze(text: &str) -> Option<Handler> {
    let allocator = Allocator::default();
    let wrapped = cstr!("({text}\n);");
    if let Some(parsed) = parse(&allocator, &wrapped)
        && let [Statement::ExpressionStatement(statement)] = parsed.program.body.as_slice()
    {
        let semantic = SemanticBuilder::new().build(&parsed.program).semantic;
        let free_names = free_names(&semantic, &parsed.program);
        let has_comment = !parsed.program.comments.is_empty();
        let expression = statement.expression.without_parentheses();
        let (function_params, call) = match expression {
            Expression::ArrowFunctionExpression(arrow) => (
                Some(param_count(&arrow.params)),
                body_call(&arrow.body, Some(&arrow.params)),
            ),
            Expression::FunctionExpression(function) => (
                Some(param_count(&function.params)),
                function
                    .body
                    .as_ref()
                    .and_then(|body| body_call(body, Some(&function.params))),
            ),
            expression => (None, identifier_call(expression, None)),
        };
        return Some(Handler {
            function_params,
            call,
            has_comment,
            free_names,
        });
    }
    // An inline handler may be a statement list (`a(); b()`).
    let parsed = parse(&allocator, text)?;
    let semantic = SemanticBuilder::new().build(&parsed.program).semantic;
    Some(Handler {
        function_params: None,
        call: statements_call(&parsed.program.body, None),
        has_comment: !parsed.program.comments.is_empty(),
        free_names: free_names(&semantic, &parsed.program),
    })
}

fn parse<'a>(allocator: &'a Allocator, text: &'a str) -> Option<ParserReturn<'a>> {
    let parsed = Parser::new(allocator, text, SourceType::ts()).parse();
    (!parsed.panicked && parsed.diagnostics.is_empty()).then_some(parsed)
}

fn param_count(params: &FormalParameters<'_>) -> usize {
    params.items.len() + usize::from(params.rest.is_some())
}

fn body_call(
    body: &FunctionBody<'_>,
    params: Option<&FormalParameters<'_>>,
) -> Option<IdentifierCall> {
    if !body.directives.is_empty() {
        return None;
    }
    statements_call(&body.statements, params)
}

/// Upstream's `getIdentifierCallExpression`: one expression statement, through
/// nested blocks and empty statements.
fn statements_call(
    statements: &[Statement<'_>],
    params: Option<&FormalParameters<'_>>,
) -> Option<IdentifierCall> {
    let mut statements = statements;
    loop {
        let mut significant = statements
            .iter()
            .filter(|statement| !matches!(statement, Statement::EmptyStatement(_)));
        let (Some(statement), None) = (significant.next(), significant.next()) else {
            return None;
        };
        match statement {
            Statement::ExpressionStatement(statement) => {
                return identifier_call(statement.expression.without_parentheses(), params);
            }
            Statement::BlockStatement(block) => statements = &block.body,
            _ => return None,
        }
    }
}

fn identifier_call(
    expression: &Expression<'_>,
    params: Option<&FormalParameters<'_>>,
) -> Option<IdentifierCall> {
    let Expression::CallExpression(call) = expression else {
        return None;
    };
    if call.optional || !matches!(call.callee.without_parentheses(), Expression::Identifier(_)) {
        return None;
    }
    Some(IdentifierCall {
        arguments: call.arguments.len(),
        forwards_params: params.is_some_and(|params| {
            params.rest.is_none()
                && params.items.len() == call.arguments.len()
                && params
                    .items
                    .iter()
                    .zip(&call.arguments)
                    .all(|(param, argument)| {
                        param.initializer.is_none()
                            && matches!(
                                (&param.pattern, argument),
                                (BindingPattern::BindingIdentifier(param), Argument::Identifier(argument))
                                    if param.name == argument.name
                            )
                    })
        }),
    })
}

fn free_names<'a>(semantic: &Semantic<'a>, program: &oxc_ast::ast::Program<'a>) -> Vec<String> {
    let mut names = FreeNames {
        semantic,
        names: Vec::new(),
    };
    names.visit_program(program);
    names.names
}

struct FreeNames<'s, 'a> {
    semantic: &'s Semantic<'a>,
    names: Vec<String>,
}

impl<'a> Visit<'a> for FreeNames<'_, 'a> {
    fn visit_identifier_reference(&mut self, id: &IdentifierReference<'a>) {
        let Some(reference) = id.reference_id.get() else {
            return;
        };
        let reference = self.semantic.scoping().get_reference(reference);
        if reference.is_value() && reference.symbol_id().is_none() {
            self.names.push(String::from(id.name.as_str()));
        }
    }

    fn visit_ts_type(&mut self, _: &TSType<'a>) {}
}
