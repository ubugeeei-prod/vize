//! If node code generation for Vapor mode.

use super::block::GenerateContext;
use crate::ir::{BlockIRNode, IfIRNode, NegativeBranch};
use vize_carton::{cstr, String};

/// Generate if node code
pub fn generate_if<F>(ctx: &mut GenerateContext, if_node: &IfIRNode<'_>, generate_block: F)
where
    F: Fn(&mut GenerateContext, &BlockIRNode<'_>) + Copy,
{
    let condition = if if_node.condition.is_static {
        cstr!("\"{}\"", if_node.condition.content)
    } else {
        vize_carton::CompactString::from(if_node.condition.content.as_str())
    };

    ctx.push_line_fmt(format_args!("_createIf(() => {condition}, () => {{"));
    ctx.indent();
    generate_block(ctx, &if_node.positive);
    ctx.deindent();

    if let Some(ref negative) = if_node.negative {
        ctx.push_line("}, () => {");
        ctx.indent();
        match negative {
            NegativeBranch::Block(block) => generate_block(ctx, block),
            NegativeBranch::If(nested_if) => generate_if(ctx, nested_if, generate_block),
        }
        ctx.deindent();
    }

    ctx.push_line("})");
}

/// Generate simple if expression (for inline conditionals)
pub fn generate_if_expression(condition: &str, then_expr: &str, else_expr: Option<&str>) -> String {
    if let Some(else_val) = else_expr {
        cstr!("{condition} ? {then_expr} : {else_val}")
    } else {
        cstr!("{condition} ? {then_expr} : null")
    }
}

/// Generate if as ternary for simple cases
pub fn can_use_ternary(if_node: &IfIRNode<'_>) -> bool {
    // Can use ternary if both branches have single return
    if_node.positive.returns.len() == 1
        && if_node.negative.as_ref().is_none_or(|n| match n {
            NegativeBranch::Block(b) => b.returns.len() == 1,
            NegativeBranch::If(_) => false,
        })
}

#[cfg(test)]
mod tests {
    use super::generate_if_expression;

    #[test]
    fn test_generate_if_expression() {
        let result = generate_if_expression("show", "_n1", Some("null"));
        assert_eq!(result, "show ? _n1 : null");
    }

    #[test]
    fn test_generate_if_expression_no_else() {
        let result = generate_if_expression("show", "_n1", None);
        assert_eq!(result, "show ? _n1 : null");
    }
}
