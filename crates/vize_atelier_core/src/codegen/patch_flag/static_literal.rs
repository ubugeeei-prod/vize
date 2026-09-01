//! Static object/array literal classification for patch flags.
//!
//! Split from `patch_flag.rs`; the P1-7 retained-AST gate lives here. The
//! legacy path parses the *trimmed* content wrapped in parens; the retained
//! AST covers the node's untrimmed content, and whitespace never changes an
//! expression parse, so the shape decision is the same decision. The legacy
//! wrapper adds one `ParenthesizedExpression`, which
//! [`is_static_oxc_expression`] unwraps — decision-equal to walking the
//! retained AST directly.

use oxc_ast::ast as oxc_ast_types;
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_relief::SimpleExpressionNode;
use vize_s0::String;

pub(super) fn is_string_literal(content: &str) -> bool {
    (content.starts_with('\'') && content.ends_with('\''))
        || (content.starts_with('"') && content.ends_with('"'))
}

/// Node-aware entry (P1-7): reads the retained AST when it still describes
/// the node's bytes and the dialect gate holds; the legacy trimmed-wrapped
/// parse otherwise. `content` is the caller's trimmed view of
/// `simple.content`.
pub(super) fn is_static_object_or_array_literal_node(
    simple: &SimpleExpressionNode<'_>,
    content: &str,
) -> bool {
    match crate::retained::retained_whole_expression(simple) {
        Some(js) if crate::retained::js_module_compatible(js) => {
            let result = is_static_oxc_expression(js.ast);
            #[cfg(any(test, feature = "davinci-differential"))]
            {
                // Dual-run in an uncounted arena: lane-only work stays off
                // the production re-parse floor. Divergence panics.
                let legacy = is_static_object_or_array_literal_in(
                    content,
                    &oxc_allocator::Allocator::default(),
                );
                assert_eq!(
                    result, legacy,
                    "davinci-differential (P1-7): retained static-literal check diverged from the legacy re-parse for expression {content:?}"
                );
                crate::retained::differential::record_shape_comparison();
            }
            result
        }
        _ => {
            let allocator = crate::expr_parse_probe::parse_arena();
            is_static_object_or_array_literal_in(content, &allocator)
        }
    }
}

/// The legacy parse path, arena provided by the caller: the production
/// fallback counts it via the P0-3 probe; the differential dual-run uses an
/// uncounted arena.
fn is_static_object_or_array_literal_in(
    content: &str,
    allocator: &oxc_allocator::Allocator,
) -> bool {
    if !crate::steps::expression::expression_is_safe_to_parse(content) {
        return false;
    }
    let mut wrapped = String::with_capacity(content.len() + 2);
    wrapped.push('(');
    wrapped.push_str(content);
    wrapped.push(')');

    let parser = Parser::new(allocator, &wrapped, SourceType::default().with_module(true));
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
                    !prop.computed && is_static_oxc_expression(&prop.value)
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

#[cfg(test)]
mod tests {
    use super::is_static_object_or_array_literal_in;

    fn is_static(content: &str) -> bool {
        is_static_object_or_array_literal_in(content, &oxc_allocator::Allocator::default())
    }

    #[test]
    fn computed_object_keys_are_dynamic() {
        assert!(!is_static("{ [prop]: 'red' }"));
        assert!(!is_static("{ [prop]: true }"));
        assert!(!is_static("{ [k]: 1 }"));
    }

    #[test]
    fn static_object_and_array_literals_stay_static() {
        assert!(is_static("{ color: 'red' }"));
        assert!(is_static("['card']"));
        assert!(is_static("{ top: '1px', left: '2px' }"));
    }
}
