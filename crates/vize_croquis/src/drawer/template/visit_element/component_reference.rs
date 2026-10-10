//! Component target expression recognition, preserving its ordinary-expression contract.

use oxc_allocator::Allocator;
use oxc_ast::ast::Expression;
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_carton::CompactString;
use vize_relief::{ExpressionNode, JsExpression};

pub(super) fn expression_identifier(
    exp: &ExpressionNode<'_>,
    template_source: &str,
) -> Option<CompactString> {
    let (source, retained) = match exp {
        ExpressionNode::Simple(simple) => (
            simple.content,
            simple.js_ast.and_then(|js| js.as_expression()),
        ),
        ExpressionNode::Compound(compound) => (compound.loc.span.slice(template_source), None),
    };
    component_reference_expression(source, retained.as_ref())
}

/// Recognize `<component :is="...">` targets that name a component: a lone
/// identifier or a static member chain.
///
/// Nodes carrying the parse-once retained AST (P1-5) are shape-checked
/// directly and the legacy throwaway parse dies for them (Davinci P1-6);
/// nodes without one (invalid or incomplete text, compound expressions) keep
/// the legacy parse. Under `cfg(any(test, feature = "legacy-differential"))`
/// every retained check is dual-run against the legacy parse and divergence
/// panics — the P1-6 differential lane.
fn component_reference_expression(
    source: &str,
    retained: Option<&JsExpression<'_>>,
) -> Option<CompactString> {
    let result = match retained {
        Some(js) => component_reference_from_ast(js.ast, source),
        None => parse_component_reference_expression(source),
    };
    #[cfg(any(test, feature = "legacy-differential"))]
    if retained.is_some() {
        let legacy = parse_component_reference_expression(source);
        assert_eq!(
            result, legacy,
            "davinci-differential (P1-6): retained-AST component-reference check diverged from the legacy parse for expression {source:?}"
        );
        crate::drawer::differential::record_component_reference_comparison();
    }
    result
}

fn component_reference_from_ast(ast: &Expression<'_>, source: &str) -> Option<CompactString> {
    match ast {
        Expression::Identifier(_) | Expression::StaticMemberExpression(_) => {
            Some(CompactString::new(source.trim()))
        }
        _ => None,
    }
}

fn parse_component_reference_expression(source: &str) -> Option<CompactString> {
    let allocator = Allocator::default();
    let source_type = SourceType::from_path("expr.ts").unwrap_or_default();
    let expression = Parser::new(&allocator, source, source_type)
        .parse_expression()
        .ok()?;
    match expression {
        Expression::Identifier(_) | Expression::StaticMemberExpression(_) => {
            Some(CompactString::new(source.trim()))
        }
        _ => None,
    }
}
