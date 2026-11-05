//! Exclusions a `v-else-if` guard's own discriminant test already implies.
//!
//! A branch guard restates every earlier branch's negation before its own
//! condition: `!(c.type === 'a') && !(c.type === 'b') && (c.type === 'c')`.
//! When the negations directly before a positive `R === L` test the same
//! reference `R` against other literals, `R === L` already removes every union
//! member and every `R` value they remove, so TypeScript narrows `R` and its
//! object to the same types without them.

use oxc_allocator::Allocator;
use oxc_ast::ast::{BinaryExpression, BinaryOperator, Expression, UnaryOperator};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};
use vize_carton::String;

/// A literal compared by value: `'a'` and `"a"` are the same literal.
#[derive(PartialEq)]
enum Literal<'a> {
    String(&'a str),
    Number(u64),
}

/// `reference === literal`, with the reference spelled as a static member chain.
struct Discriminant<'s, 'a> {
    reference: &'s str,
    literal: Literal<'a>,
    negated: bool,
}

/// The guard without its implied exclusions, or `None` when none apply.
pub(crate) fn drop_implied_exclusions(guard: &str) -> Option<String> {
    let allocator = Allocator::default();
    let expression = Parser::new(&allocator, guard, SourceType::ts())
        .parse_expression()
        .ok()?;
    // The parser may accept a valid prefix; never rewrite around trailing text.
    let span = expression.span();
    if guard.get(span.start as usize..span.end as usize) != Some(guard.trim()) {
        return None;
    }
    let mut terms = Vec::new();
    collect_terms(&expression, &mut terms);
    let tests: Vec<_> = terms.iter().map(|term| discriminant(term, guard)).collect();
    let mut kept = vec![true; terms.len()];
    for (index, test) in tests.iter().enumerate() {
        let Some(positive) = test.as_ref().filter(|test| !test.negated) else {
            continue;
        };
        for earlier in (0..index).rev() {
            match tests.get(earlier).and_then(Option::as_ref) {
                Some(test)
                    if test.negated
                        && test.reference == positive.reference
                        && test.literal != positive.literal =>
                {
                    if let Some(slot) = kept.get_mut(earlier) {
                        *slot = false;
                    }
                }
                _ => break,
            }
        }
    }
    if kept.iter().all(|kept| *kept) {
        return None;
    }
    let mut rewritten = String::default();
    for (term, _) in terms.iter().zip(&kept).filter(|(_, kept)| **kept) {
        if !rewritten.is_empty() {
            rewritten.push_str(" && ");
        }
        let span = term.span();
        rewritten.push_str(
            guard
                .get(span.start as usize..span.end as usize)
                .unwrap_or(guard),
        );
    }
    Some(rewritten)
}

fn collect_terms<'a, 'b>(expression: &'b Expression<'a>, terms: &mut Vec<&'b Expression<'a>>) {
    if let Expression::LogicalExpression(logical) = expression
        && logical.operator.is_and()
    {
        collect_terms(&logical.left, terms);
        collect_terms(&logical.right, terms);
    } else {
        terms.push(expression);
    }
}

fn discriminant<'s, 'a>(term: &Expression<'a>, source: &'s str) -> Option<Discriminant<'s, 'a>> {
    match term.without_parentheses() {
        Expression::UnaryExpression(unary) if unary.operator == UnaryOperator::LogicalNot => {
            let Expression::BinaryExpression(binary) = unary.argument.without_parentheses() else {
                return None;
            };
            equality(binary, source, true)
        }
        Expression::BinaryExpression(binary) => equality(binary, source, false),
        _ => None,
    }
}

fn equality<'s, 'a>(
    binary: &BinaryExpression<'a>,
    source: &'s str,
    negated: bool,
) -> Option<Discriminant<'s, 'a>> {
    if binary.operator != BinaryOperator::StrictEquality {
        return None;
    }
    let (reference, literal) = literal(&binary.right)
        .map(|literal| (&binary.left, literal))
        .or_else(|| literal(&binary.left).map(|literal| (&binary.right, literal)))?;
    let reference = reference.without_parentheses();
    is_static_reference(reference).then_some(())?;
    let span = reference.span();
    Some(Discriminant {
        reference: source.get(span.start as usize..span.end as usize)?,
        literal,
        negated,
    })
}

fn literal<'a>(expression: &Expression<'a>) -> Option<Literal<'a>> {
    match expression.without_parentheses() {
        Expression::StringLiteral(string) => Some(Literal::String(string.value.as_str())),
        Expression::NumericLiteral(number) if number.value.is_finite() => {
            Some(Literal::Number(number.value.to_bits()))
        }
        _ => None,
    }
}

/// An identifier or `a.b.c`: a reference TypeScript narrows by equality.
fn is_static_reference(expression: &Expression<'_>) -> bool {
    match expression {
        Expression::Identifier(_) => true,
        Expression::StaticMemberExpression(member) => {
            !member.optional && is_static_reference(member.object.without_parentheses())
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests;
