//! Shared retained-AST walk for the template core-extension rules.

mod camelcase;
mod constant;
mod precision;
mod predicates;
mod reporting;
mod tokens;
mod visitor;

use super::origin::Origin;
use crate::context::{LintContext, RetainedVFor};
use constant::is_constant;
use oxc_ast::ast::{Expression, FormalParameters, Program};
use oxc_ast_visit::Visit;
use oxc_span::{GetSpan, Span};
use predicates::is_irregular_whitespace;
use visitor::CoreVisitor;
use vize_l1::markup::entity::EntityContext;
use vize_relief::{DirectiveNode, ExpressionNode, SimpleExpressionNode};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum CoreKind {
    Camelcase,
    DotNotation,
    Eqeqeq,
    NoConsole,
    NoConstantCondition,
    NoEmptyPattern,
    NoImplicitCoercion,
    NoIrregularWhitespace,
    NoLossOfPrecision,
    NoNegatedCondition,
    NoSparseArrays,
    NoUselessConcat,
    ObjectShorthand,
    PreferTemplate,
    QuoteProps,
}

impl CoreKind {
    fn message(self) -> &'static str {
        match self {
            Self::Camelcase => "Identifier is not in camel case",
            Self::DotNotation => "Use dot notation instead of a computed property",
            Self::Eqeqeq => "Use strict equality operators",
            Self::NoConsole => "Unexpected console access in a template expression",
            Self::NoConstantCondition => "Unexpected constant condition",
            Self::NoEmptyPattern => "Unexpected empty destructuring pattern",
            Self::NoImplicitCoercion => "Use an explicit type conversion",
            Self::NoIrregularWhitespace => "Unexpected irregular whitespace",
            Self::NoLossOfPrecision => "This numeric literal loses precision",
            Self::NoNegatedCondition => "Unexpected negated ternary condition",
            Self::NoSparseArrays => "Unexpected sparse array",
            Self::NoUselessConcat => "Unexpected concatenation of literal strings",
            Self::ObjectShorthand => "Use object property shorthand",
            Self::PreferTemplate => "Use a template literal instead of string concatenation",
            Self::QuoteProps => "Unnecessarily quoted object property",
        }
    }
}

pub(super) fn check_expression<'a>(
    ctx: &mut LintContext<'a>,
    expression: &ExpressionNode<'a>,
    kind: CoreKind,
    entity_context: EntityContext,
) {
    check_expression_with_directive(ctx, expression, kind, entity_context, None);
}

fn check_expression_with_directive<'a>(
    ctx: &mut LintContext<'a>,
    expression: &ExpressionNode<'a>,
    kind: CoreKind,
    entity_context: EntityContext,
    directive: Option<&str>,
) {
    let ExpressionNode::Simple(simple) = expression else {
        return;
    };
    let Some(origin) = Origin::new(ctx.source, simple.content, simple.loc.span, entity_context)
    else {
        return;
    };

    if kind == CoreKind::NoIrregularWhitespace {
        for (index, character) in simple.content.char_indices() {
            if is_irregular_whitespace(character)
                && let Some(range) = origin.range(Span::new(
                    index as u32,
                    (index + character.len_utf8()) as u32,
                ))
            {
                ctx.warn_at_with_help(kind.message(), range, compatibility_help());
            }
        }
        return;
    }

    let Some(goal) = template_goal(ctx, simple, directive) else {
        return;
    };
    let bindings = match goal {
        TemplateGoal::Expression(_) => !matches!(directive, Some("for" | "slot")),
        TemplateGoal::Handler(_) => true,
        TemplateGoal::SlotParameters(_) | TemplateGoal::VFor(_) => false,
    };
    let mut visitor = CoreVisitor::new(ctx, origin, simple.content, kind, bindings);
    if kind == CoreKind::Camelcase {
        // A use reports even before its declaration: collect declarations first.
        visitor.silent = true;
        walk_goal(&mut visitor, &goal);
        visitor.silent = false;
        visitor.camelcase_bindings = bindings;
        visitor.set_parse_offset(0);
    }
    if kind == CoreKind::NoConstantCondition
        && matches!(directive, Some("if" | "else-if" | "show"))
        && let TemplateGoal::Expression(expression) = goal
        && is_constant(expression, true)
    {
        visitor.report(expression.without_parentheses().span());
    }
    walk_goal(&mut visitor, &goal);
}

/// The JavaScript goal a template expression parses as.
#[derive(Clone, Copy)]
pub(super) enum TemplateGoal<'a> {
    Expression(&'a Expression<'a>),
    SlotParameters(&'a FormalParameters<'a>),
    VFor(RetainedVFor<'a>),
    Handler(&'a Program<'a>),
}

/// The retained AST of an expression, or the handler / `v-for` parse.
pub(super) fn template_goal<'a>(
    ctx: &mut LintContext<'a>,
    simple: &SimpleExpressionNode<'a>,
    directive: Option<&str>,
) -> Option<TemplateGoal<'a>> {
    if let Some(retained) = simple
        .js_ast
        .filter(|retained| retained.raw == simple.content)
    {
        if let Some(expression) = retained.as_expression() {
            return Some(TemplateGoal::Expression(expression.ast));
        }
        if let Some(Ok(parameters)) = retained.as_slot_parameters() {
            return Some(TemplateGoal::SlotParameters(parameters));
        }
    }
    match directive {
        Some("for") => ctx
            .compatibility_v_for(simple.loc.span, simple.content)
            .map(TemplateGoal::VFor),
        Some("on") => ctx
            .compatibility_handler(simple.loc.span, simple.content)
            .map(TemplateGoal::Handler),
        _ => None,
    }
}

fn walk_goal(visitor: &mut CoreVisitor<'_, '_>, goal: &TemplateGoal<'_>) {
    match goal {
        TemplateGoal::Expression(expression) => visitor.visit_expression(expression),
        TemplateGoal::SlotParameters(parameters) => visitor.visit_formal_parameters(parameters),
        TemplateGoal::VFor(retained) => {
            visitor.set_parse_offset(retained.parameters_offset);
            visitor.camelcase_bindings = false;
            visitor.visit_formal_parameters(retained.parameters);
            visitor.set_parse_offset(retained.source_offset);
            visitor.camelcase_bindings = true;
            visitor.visit_expression(retained.source);
        }
        TemplateGoal::Handler(program) => visitor.visit_program(program),
    }
}

pub(super) fn check_directive<'a>(
    ctx: &mut LintContext<'a>,
    directive: &DirectiveNode<'a>,
    kind: CoreKind,
) {
    if let Some(expression) = &directive.exp {
        check_expression_with_directive(
            ctx,
            expression,
            kind,
            EntityContext::Attribute,
            Some(directive.name),
        );
    }
    if !matches!(kind, CoreKind::ObjectShorthand | CoreKind::QuoteProps)
        && let Some(argument) = &directive.arg
    {
        check_expression_with_directive(ctx, argument, kind, EntityContext::Attribute, None);
    }
}

pub(super) fn check_sfc_styles(ctx: &mut LintContext<'_>, kind: CoreKind) {
    let spans = ctx
        .sfc_descriptor()
        .into_iter()
        .flat_map(|descriptor| &descriptor.styles)
        .flat_map(|style| {
            crate::rules::css::value_tokens::ValueTokens::new(style.content.as_ref())
                .bindings
                .into_iter()
                .filter_map(move |(start, end)| {
                    style_expression_span(style.content.as_ref(), start, end)
                        .map(|(start, end)| (style.loc.start + start, style.loc.start + end))
                })
        })
        .collect::<Vec<_>>();

    for (start, end) in spans {
        let full_source = ctx.source;
        let Some(source) = full_source.get(start..end) else {
            continue;
        };
        let span = vize_l0::Span::new(start as u32, end as u32);
        let Some(origin) = Origin::new(ctx.source, source, span, EntityContext::Text) else {
            continue;
        };
        let Some(expression) = ctx.compatibility_expression(span, source) else {
            continue;
        };
        CoreVisitor::new(ctx, origin, source, kind, false).visit_expression(expression);
    }
}

fn style_expression_span(source: &str, start: usize, end: usize) -> Option<(usize, usize)> {
    let call = source.get(start..end)?;
    let inner = call.strip_prefix("v-bind(")?.strip_suffix(')')?;
    let leading = inner.len().checked_sub(inner.trim_start().len())?;
    let trailing = inner.len().checked_sub(inner.trim_end().len())?;
    let mut expression_start = start.checked_add("v-bind(".len() + leading)?;
    let mut expression_end = end.checked_sub(1 + trailing)?;
    let expression = source.get(expression_start..expression_end)?;
    if matches!(
        (expression.as_bytes().first(), expression.as_bytes().last()),
        (Some(b'\''), Some(b'\'')) | (Some(b'"'), Some(b'"'))
    ) {
        expression_start = expression_start.checked_add(1)?;
        expression_end = expression_end.checked_sub(1)?;
    }
    (expression_start < expression_end).then_some((expression_start, expression_end))
}

fn compatibility_help() -> &'static str {
    "Move this JavaScript expression to a script binding or rewrite it using the recommended syntax"
}
