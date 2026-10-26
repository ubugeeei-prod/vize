//! Exceptional native aliases keep their original binding grammar as authority.

use alloc::vec::Vec;
use oxc_ast::ast::{BindingIdentifier, BindingPattern, FormalParameters};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};
use vize_l0::expression_guard::{expression_is_safe_to_parse, is_expression_trailing_trivia};
use vize_l0::{SmallVec, Span, String, profiler::global_profiler};
use vize_l1::embed::{EmbedSource, prepare_attribute_value};
use vize_l2::expr::ExprRef;
use vize_l2::scope::{ScopeBinding, ScopeOrigin};

use super::{Cx, ForName};

pub(super) enum AliasGoal<'a> {
    Ordinary,
    Rejected,
    Parsed {
        source: EmbedSource<'a>,
        parameters: FormalParameters<'a>,
    },
}

impl<'a> AliasGoal<'a> {
    pub(super) fn parse(cx: &Cx<'a>, expr: Option<&ExprRef<'a>>) -> Self {
        let Some(expr) = expr else {
            return Self::Ordinary;
        };
        let alias = expr.source();
        // Keep every ordinary and foreign position on its existing path.
        if cx.foreign_dialect.is_some() || (alias.is_ascii() && !alias.contains('\\')) {
            return Self::Ordinary;
        }
        if !expression_is_safe_to_parse(alias) {
            return Self::Rejected;
        }
        let Ok(source) = prepare_attribute_value(cx.allocator, cx.source, expr.span()) else {
            return Self::Rejected;
        };
        if source.text() != alias {
            return Self::Rejected;
        }
        // This is the actual native frontend's original parameter goal, not
        // the Core frontend's retained root or a synthetic declaration parse.
        global_profiler().record_counter("davinci.for_binding.parses", 1);
        let Ok(parameters) = Parser::new(cx.allocator.as_oxc(), source.text(), SourceType::ts())
            .parse_v_for_parameters()
        else {
            return Self::Rejected;
        };
        if !alias
            .get(parameters.span().end as usize..)
            .is_some_and(is_expression_trailing_trivia)
            || !visit_bindings(&parameters, &mut |identifier| {
                source
                    .authored_span(Span::new(identifier.span.start, identifier.span.end))
                    .is_ok()
            })
        {
            return Self::Rejected;
        }
        Self::Parsed { source, parameters }
    }

    /// `true` means this goal owns the position, even when it was refused.
    /// All exceptional declaration origins cover their full authored tokens,
    /// including escapes/entities, and are contained in the alias's span.
    pub(super) fn append_bindings(&self, bindings: &mut Vec<ScopeBinding>) -> bool {
        let Self::Parsed { source, parameters } = self else {
            return !matches!(self, Self::Ordinary);
        };
        let start = bindings.len();
        if !visit_bindings(parameters, &mut |identifier| {
            let Ok(span) =
                source.authored_span(Span::new(identifier.span.start, identifier.span.end))
            else {
                return false;
            };
            bindings.push(ScopeBinding {
                name: String::from(identifier.name.as_str()),
                origin: ScopeOrigin::Authored { span },
            });
            true
        }) {
            // A broken projection cannot publish a partial declaration list.
            bindings.truncate(start);
        }
        true
    }

    pub(super) fn position(&self) -> Option<ForName> {
        match self {
            Self::Ordinary => None,
            Self::Rejected => Some(ForName::Pending),
            Self::Parsed { parameters, .. } => Some(match parameters.items.as_slice() {
                [parameter] if parameters.rest.is_none() && parameter.initializer.is_none() => {
                    match &parameter.pattern {
                        BindingPattern::BindingIdentifier(identifier) => {
                            ForName::Named(String::from(identifier.name.as_str()))
                        }
                        _ => ForName::Pending,
                    }
                }
                _ => ForName::Pending,
            }),
        }
    }
}

fn visit_bindings<'a>(
    parameters: &FormalParameters<'a>,
    visit: &mut impl FnMut(&BindingIdentifier<'a>) -> bool,
) -> bool {
    let mut pending = SmallVec::<[&BindingPattern<'_>; 8]>::new();
    if let Some(rest) = &parameters.rest {
        pending.push(&rest.rest.argument);
    }
    for parameter in parameters.items.iter().rev() {
        pending.push(&parameter.pattern);
    }
    while let Some(pattern) = pending.pop() {
        match pattern {
            BindingPattern::BindingIdentifier(identifier) => {
                if !visit(identifier) {
                    return false;
                }
            }
            BindingPattern::ObjectPattern(object) => {
                if let Some(rest) = &object.rest {
                    pending.push(&rest.argument);
                }
                for property in object.properties.iter().rev() {
                    pending.push(&property.value);
                }
            }
            BindingPattern::ArrayPattern(array) => {
                if let Some(rest) = &array.rest {
                    pending.push(&rest.argument);
                }
                for element in array.elements.iter().rev().flatten() {
                    pending.push(element);
                }
            }
            BindingPattern::AssignmentPattern(assignment) => pending.push(&assignment.left),
        }
    }
    true
}
