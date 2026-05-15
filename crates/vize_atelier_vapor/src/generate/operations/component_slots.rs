use crate::ir::IRSlot;
use vize_carton::{cstr, FxHashMap, String};

use super::super::{context::GenerateContext, generate_block};

/// Generate a single slot function body
pub(super) fn generate_slot_fn(
    ctx: &mut GenerateContext,
    slot: &IRSlot<'_>,
    element_template_map: &FxHashMap<usize, usize>,
    use_with_vapor_ctx: bool,
) {
    let slot_props_var = slot
        .fn_exp
        .as_ref()
        .map(|fn_exp| ctx.push_slot_scope(fn_exp.content.as_str()));
    if use_with_vapor_ctx {
        ctx.use_helper("withVaporCtx");
        let param: String = slot_props_var
            .as_ref()
            .map(|v| cstr!(" _withVaporCtx(({}) => {{\n", v))
            .unwrap_or_else(|| String::from(" _withVaporCtx(() => {\n"));
        ctx.push(&param);
    } else {
        let param: String = slot_props_var
            .as_ref()
            .map(|v| cstr!(" ({}) => {{\n", v))
            .unwrap_or_else(|| String::from(" () => {\n"));
        ctx.push(&param);
    }
    ctx.indent();
    ctx.push_component_scope();
    generate_block(ctx, &slot.block, element_template_map);
    ctx.pop_component_scope();
    ctx.deindent();
    ctx.push_indent();
    ctx.push("}");
    if use_with_vapor_ctx {
        ctx.push(")");
    }
    if slot_props_var.is_some() {
        ctx.pop_slot_scope();
    }
}
