//! Typed references for proven named values of the built-in component.

use oxc_ast::ast::Expression;
use vize_carton::{String, append};
use vize_croquis::analysis::ComponentUsage;

use crate::virtual_ts::component_reference::resolved_exact_component_binding_reference;

use super::super::{
    context::ComponentPropsContext, dynamic_component::owned_named_dynamic_component_ast,
};

pub(in crate::virtual_ts::scope) fn named_dynamic_reference(
    ctx: &ComponentPropsContext<'_, '_>,
    usage: &ComponentUsage,
) -> Option<String> {
    if !usage.name.contains('.') {
        return None;
    }
    named_dynamic_navigation_reference(ctx, usage)
}

/// Navigation follows the retained authored member even when props use its
/// narrowing-safe alias. Its ignored read never owns expression diagnostics.
pub(in crate::virtual_ts::scope) fn named_dynamic_navigation_reference(
    ctx: &ComponentPropsContext<'_, '_>,
    usage: &ComponentUsage,
) -> Option<String> {
    if ctx.legacy_vue2 {
        return None;
    }
    let ast = owned_named_dynamic_component_ast(ctx.template_ast, usage)?;
    if !matches!(ast, Expression::StaticMemberExpression(_)) {
        return None;
    }
    reference(ctx, ast)
}

fn reference(ctx: &ComponentPropsContext<'_, '_>, ast: &Expression<'_>) -> Option<String> {
    match ast {
        Expression::Identifier(identifier) => resolved_exact_component_binding_reference(
            ctx.summary,
            ctx.options,
            ctx.syntactic_type_only_imported_names,
            identifier.name.as_str(),
        ),
        Expression::StaticMemberExpression(member) => {
            let mut value = reference(ctx, &member.object)?;
            append!(value, ".{}", member.property.name);
            Some(value)
        }
        Expression::ParenthesizedExpression(parenthesized) => {
            reference(ctx, &parenthesized.expression)
        }
        _ => None,
    }
}
