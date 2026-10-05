//! Typed references for proven named values of the built-in component.

use oxc_ast::ast::Expression;
use vize_carton::{String, append};
use vize_croquis::analysis::ComponentUsage;

use crate::virtual_ts::component_reference::resolved_exact_component_binding_reference;

use super::super::{
    context::ComponentPropsContext, dynamic_component::owned_named_dynamic_component_ast,
};

pub(super) fn named_dynamic_reference(
    ctx: &ComponentPropsContext<'_, '_>,
    usage: &ComponentUsage,
) -> Option<String> {
    if ctx.legacy_vue2 || !usage.name.contains('.') {
        return None;
    }
    let ast = owned_named_dynamic_component_ast(ctx.template_ast, usage)?;
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
