//! Project source-owned parameter facts, without reparsing or guessing names.

use oxc_ast::ast::{BindingPattern, Expression};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};
use vize_l0::expression_guard::expression_is_safe_to_parse;
use vize_l0::{Allocator, SmallVec, Span, String};
use vize_l2::scope::{ScopeFacts, ScopeOrigin};

/// Exceptional alias names already observed by the original binding-goal
/// producer. `Some(empty)` is a refusal, never permission to scan raw text.
/// Ordinary aliases keep the existing consumer path and allocation window.
#[doc(hidden)]
pub fn exceptional_for_binding_names<'a>(
    alias: &str,
    alias_span: Span,
    facts: Option<&'a ScopeFacts>,
) -> Option<impl Iterator<Item = &'a str> + use<'a>> {
    if alias.is_ascii() && !alias.contains('\\') {
        return None;
    }
    Some(
        facts
            .into_iter()
            .flat_map(|facts| &facts.bindings)
            .filter_map(move |binding| match binding.origin {
                ScopeOrigin::Authored { span }
                    if span.start >= alias_span.start && span.end <= alias_span.end =>
                {
                    Some(binding.name.as_str())
                }
                _ => None,
            }),
    )
}

/// Rewrite each default initializer and computed key inside a loop alias
/// pattern with `rewrite`, keeping every declaration spelled as authored.
/// Callers enter the alias scope first: the pattern's own names stay local.
/// `Ok(None)` keeps the alias as authored, including one that does not parse
/// as parameters: the existing diagnostics own it.
#[doc(hidden)]
pub fn rewrite_for_alias_expressions<E>(
    alias: &str,
    mut rewrite: impl FnMut(&str) -> Result<String, E>,
) -> Result<Option<String>, E> {
    if !alias.contains(['=', '[']) || !expression_is_safe_to_parse(alias) {
        return Ok(None);
    }
    let mut wrapped = String::with_capacity(alias.len() + 10);
    wrapped.push('(');
    wrapped.push_str(alias);
    wrapped.push_str(") => null");
    let allocator = Allocator::new();
    let parser = Parser::new(
        allocator.as_oxc(),
        wrapped.as_str(),
        SourceType::ts().with_module(true),
    );
    let Ok(Expression::ArrowFunctionExpression(arrow)) = parser.parse_expression() else {
        return Ok(None);
    };
    let mut spans = SmallVec::<[oxc_span::Span; 4]>::new();
    for param in &arrow.params.items {
        pattern_expressions(&param.pattern, &mut spans);
    }
    if spans.is_empty() {
        return Ok(None);
    }
    let mut result = String::with_capacity(alias.len() + 8 * spans.len());
    let mut copied = 0;
    for span in spans {
        // The wrapper adds one leading byte before the authored alias.
        let (start, end) = (span.start as usize - 1, span.end as usize - 1);
        result.push_str(alias.get(copied..start).unwrap_or_default());
        result.push_str(rewrite(alias.get(start..end).unwrap_or_default())?.as_str());
        copied = end;
    }
    result.push_str(alias.get(copied..).unwrap_or_default());
    Ok(Some(result))
}

fn pattern_expressions(pattern: &BindingPattern<'_>, spans: &mut SmallVec<[oxc_span::Span; 4]>) {
    match pattern {
        BindingPattern::BindingIdentifier(_) => {}
        BindingPattern::ObjectPattern(object) => {
            for property in &object.properties {
                if property.computed {
                    spans.push(property.key.span());
                }
                pattern_expressions(&property.value, spans);
            }
            if let Some(rest) = &object.rest {
                pattern_expressions(&rest.argument, spans);
            }
        }
        BindingPattern::ArrayPattern(array) => {
            for element in array.elements.iter().flatten() {
                pattern_expressions(element, spans);
            }
            if let Some(rest) = &array.rest {
                pattern_expressions(&rest.argument, spans);
            }
        }
        BindingPattern::AssignmentPattern(assignment) => {
            pattern_expressions(&assignment.left, spans);
            spans.push(assignment.right.span());
        }
    }
}
