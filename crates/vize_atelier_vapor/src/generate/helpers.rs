//! Effect generation and inline operation helpers.

use crate::ir::{IREffect, OperationNode};
use vize_atelier_core::codegen::document::EmitDocument;
use vize_carton::{FxHashMap, String, cstr};

use super::{
    context::GenerateContext,
    operations::{generate_operation, merged_props_call, set_prop_call, set_text_call},
    setup::escape_js_string_literal,
};

/// Generate effect
pub(crate) fn generate_effect(
    ctx: &mut GenerateContext,
    effect: &IREffect<'_>,
    element_template_map: &FxHashMap<usize, usize>,
) {
    ctx.use_helper("renderEffect");

    // If only one operation, use single-line format
    if let [op] = effect.operations.as_slice()
        && let Some(op_code) = generate_operation_inline(ctx, op)
    {
        ctx.push_indent();
        ctx.push("_renderEffect(() => ");
        ctx.push_spanned(&op_code);
        ctx.push(")\n");
        return;
    }

    ctx.push_line("_renderEffect(() => {");
    ctx.indent();

    for op in effect.operations.iter() {
        generate_operation(ctx, op, element_template_map);
    }

    ctx.deindent();
    ctx.push_line("})");
}

/// Generate operation inline (returns code with its anchors). SetProp and
/// SetText share the statement emitters' call builders.
pub(crate) fn generate_operation_inline(
    ctx: &mut GenerateContext,
    op: &OperationNode<'_>,
) -> Option<EmitDocument> {
    match op {
        OperationNode::SetProp(set_prop) => Some(set_prop_call(ctx, set_prop)),
        OperationNode::SetDynamicProps(set_props) => Some(EmitDocument::from(
            generate_set_dynamic_props_inline(ctx, set_props),
        )),
        OperationNode::SetMergedProps(merged) => {
            Some(EmitDocument::from(merged_props_call(ctx, merged)))
        }
        OperationNode::SetText(set_text) => Some(set_text_call(ctx, set_text)),
        _ => None,
    }
}

/// Generate SetDynamicProps inline (returns code string)
fn generate_set_dynamic_props_inline(
    ctx: &mut GenerateContext,
    set_props: &crate::ir::SetDynamicPropsIRNode<'_>,
) -> String {
    let element = cstr!("n{}", set_props.element);

    if set_props.is_event {
        ctx.use_helper("setDynamicEvents");
        if let Some(first) = set_props.props.first() {
            let resolved = ctx.resolve_expression_node(first);
            cstr!("_setDynamicEvents({element}, {resolved})")
        } else {
            cstr!("_setDynamicEvents({element})")
        }
    } else {
        ctx.use_helper("setDynamicProps");
        let props_parts: Vec<String> = set_props
            .props
            .iter()
            .map(|p| {
                if p.is_static {
                    cstr!("\"{}\"", escape_js_string_literal(p.content))
                } else {
                    ctx.resolve_expression_node(p)
                }
            })
            .collect();
        cstr!("_setDynamicProps({element}, [{}])", props_parts.join(", "))
    }
}
