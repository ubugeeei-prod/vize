//! Authored source-map links for computed KeepAlive policy props.

use crate::generate::context::GenerateContext;
use crate::ir::{ComponentKind, CreateComponentIRNode, PropValueKind};
use vize_atelier_core::SimpleExpressionNode;
use vize_atelier_core::codegen::document::EmitDocument;
use vize_carton::{String, cstr};

use super::component_props::expression_value;

pub(super) fn map_computed_props(
    ctx: &GenerateContext,
    component: &CreateComponentIRNode<'_>,
    text: String,
) -> EmitDocument {
    if component.kind != ComponentKind::KeepAlive || ctx.spans.is_none() {
        return EmitDocument::from(text);
    }

    let mut document = EmitDocument::default();
    let mut cursor = 0;
    for prop in &component.props {
        if prop.key.is_static
            || prop.key.is_handler_key
            || prop.value_kind != PropValueKind::Expression
        {
            continue;
        }
        let Some(value) = prop.values.first() else {
            continue;
        };
        let key_code = ctx.resolve_expression_node(&prop.key);
        let value_code = ctx.resolve_expression_node(value);
        if expression_value(ctx, prop) != value_code {
            continue;
        }
        let needle = cstr!("[({key_code})]: {value_code}");
        let Some(rest) = text.get(cursor..) else {
            break;
        };
        let Some(relative) = rest.find(needle.as_str()) else {
            continue;
        };
        let start = cursor + relative;
        let Some(prefix) = text.get(cursor..start) else {
            break;
        };
        document.push_str(prefix);
        document.push_str("[(");
        document.push_spanned(&computed_key(ctx, &prop.key));
        document.push_str(")]: ");
        document.push_spanned(&ctx.spanned_expression_node(value));
        cursor = start + needle.len();
    }

    if let Some(rest) = text.get(cursor..) {
        document.push_str(rest);
    }
    document
}

fn computed_key(ctx: &GenerateContext, key: &SimpleExpressionNode<'_>) -> EmitDocument {
    let document = ctx.spanned_expression_node(key);
    let Some(delimiter) = key
        .loc
        .span
        .start
        .checked_sub(1)
        .filter(|&start| ctx.source.as_bytes().get(start as usize) == Some(&b'['))
    else {
        return document;
    };
    let (text, mut links) = document.into_parts();
    if let Some(link) = links
        .iter_mut()
        .find(|link| link.segment && link.authored.start == key.loc.span.start)
    {
        // The official compiler anchors a computed prop key at its opening
        // bracket while retaining the expression name on the segment.
        link.authored.start = delimiter;
    }
    EmitDocument::from_parts(text, links)
}
