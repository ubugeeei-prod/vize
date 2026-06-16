//! Static-expression analysis for patch-flag computation.
//!
//! Pure predicates that decide whether an interpolation, event handler, or
//! bound directive expression is constant/static — used by `patch_flag` to
//! decide which patch flags and dynamic-prop entries an element needs. Split
//! out of `patch_flag` to keep that file focused on the flag computation.

use crate::options::{BindingMetadata, BindingType};
use crate::{DirectiveNode, ExpressionNode};
use oxc_ast::ast as oxc_ast_types;
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_carton::String;

/// Check if an interpolation references only constant bindings (LiteralConst or SetupConst)
/// These bindings never change at runtime, so no TEXT patch flag is needed.
pub(crate) fn is_constant_interpolation(
    expr: &ExpressionNode<'_>,
    bindings: Option<&BindingMetadata>,
) -> bool {
    let bindings = match bindings {
        Some(b) => b,
        None => return false, // No binding info, assume dynamic
    };

    match expr {
        ExpressionNode::Simple(simple) => {
            // Check if the expression is a simple identifier that's a constant
            // Both LiteralConst (e.g., const x = 'hello') and SetupConst (e.g., class Foo {})
            // are constant at runtime and don't need TEXT patch flag
            let name = simple.content.as_str();
            matches!(
                bindings.bindings.get(name),
                Some(BindingType::LiteralConst | BindingType::SetupConst)
            )
        }
        ExpressionNode::Compound(_) => false, // Compound expressions are dynamic
    }
}

/// Check if an event handler references a constant binding (SetupConst or LiteralConst)
pub(crate) fn is_const_handler(
    expr: &ExpressionNode<'_>,
    bindings: Option<&BindingMetadata>,
) -> bool {
    let bindings = match bindings {
        Some(b) => b,
        None => return false, // No binding info, assume dynamic
    };

    match expr {
        ExpressionNode::Simple(simple) => {
            // Check if the expression is a simple identifier that's a constant
            let name = simple.content.as_str();
            matches!(
                bindings.bindings.get(name),
                Some(BindingType::SetupConst | BindingType::LiteralConst)
            )
        }
        ExpressionNode::Compound(_) => false, // Compound expressions are dynamic
    }
}

/// Check if a directive's bound expression is a static literal (no runtime identifiers).
pub(crate) fn is_static_bound_expression(dir: &DirectiveNode<'_>) -> bool {
    let Some(ExpressionNode::Simple(simple)) = &dir.exp else {
        return false;
    };
    if simple.is_static {
        return true;
    }

    let content = simple.content.trim();
    matches!(content, "true" | "false" | "null")
        || is_string_literal(content)
        || content.parse::<f64>().is_ok()
        || is_static_object_or_array_literal(content)
        || (content.starts_with('`') && content.ends_with('`') && !content.contains("${"))
}

fn is_string_literal(content: &str) -> bool {
    (content.starts_with('\'') && content.ends_with('\''))
        || (content.starts_with('"') && content.ends_with('"'))
}

fn is_static_object_or_array_literal(content: &str) -> bool {
    let mut wrapped = String::with_capacity(content.len() + 2);
    wrapped.push('(');
    wrapped.push_str(content);
    wrapped.push(')');

    let allocator = oxc_allocator::Allocator::default();
    let parser = Parser::new(
        &allocator,
        &wrapped,
        SourceType::default().with_module(true),
    );
    let Ok(expr) = parser.parse_expression() else {
        return false;
    };

    is_static_oxc_expression(&expr)
}

fn is_static_oxc_expression(expr: &oxc_ast_types::Expression<'_>) -> bool {
    match expr {
        oxc_ast_types::Expression::StringLiteral(_)
        | oxc_ast_types::Expression::NumericLiteral(_)
        | oxc_ast_types::Expression::BooleanLiteral(_)
        | oxc_ast_types::Expression::NullLiteral(_)
        | oxc_ast_types::Expression::BigIntLiteral(_)
        | oxc_ast_types::Expression::RegExpLiteral(_) => true,
        oxc_ast_types::Expression::TemplateLiteral(template) => template.expressions.is_empty(),
        oxc_ast_types::Expression::UnaryExpression(unary) => {
            is_static_oxc_expression(&unary.argument)
        }
        oxc_ast_types::Expression::ParenthesizedExpression(paren) => {
            is_static_oxc_expression(&paren.expression)
        }
        oxc_ast_types::Expression::CallExpression(call)
            if matches!(
                &call.callee,
                oxc_ast_types::Expression::Identifier(ident)
                    if matches!(ident.name.as_str(), "_normalizeClass" | "_normalizeStyle")
            ) =>
        {
            call.arguments.iter().all(|arg| match arg {
                oxc_ast_types::Argument::SpreadElement(_) => false,
                _ => arg.as_expression().is_some_and(is_static_oxc_expression),
            })
        }
        oxc_ast_types::Expression::ObjectExpression(obj) => {
            obj.properties.iter().all(|prop| match prop {
                oxc_ast_types::ObjectPropertyKind::ObjectProperty(prop) => {
                    is_static_oxc_expression(&prop.value)
                }
                oxc_ast_types::ObjectPropertyKind::SpreadProperty(_) => false,
            })
        }
        oxc_ast_types::Expression::ArrayExpression(arr) => {
            arr.elements.iter().all(|elem| match elem {
                oxc_ast_types::ArrayExpressionElement::SpreadElement(_) => false,
                oxc_ast_types::ArrayExpressionElement::Elision(_) => true,
                _ => elem.as_expression().is_some_and(is_static_oxc_expression),
            })
        }
        _ => false,
    }
}
