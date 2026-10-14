//! Effect generation and inline operation helpers.

use crate::ir::OperationNode;
use vize_atelier_core::codegen::document::EmitDocument;
use vize_carton::{String, cstr};

use super::{
    context::GenerateContext,
    operations::{merged_props_call, set_html_call, set_prop_call, set_text_call},
    setup::escape_js_string_literal,
};

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
        OperationNode::SetHtml(set_html) => Some(EmitDocument::from(set_html_call(ctx, set_html))),
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
        cstr!(
            "_setDynamicProps({element}, [{}]{})",
            props_parts.join(", "),
            super::operations::svg_argument(set_props.namespace)
        )
    }
}
