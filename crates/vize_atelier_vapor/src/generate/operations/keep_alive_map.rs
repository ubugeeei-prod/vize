//! Authored source-map links for computed KeepAlive policy props.

use crate::generate::context::GenerateContext;
use crate::ir::{IRProp, PropValueKind};
use vize_atelier_core::SimpleExpressionNode;
use vize_atelier_core::codegen::document::EmitDocument;
use vize_carton::{String, cstr};

pub(super) fn plain_source(text: String) -> EmitDocument {
    EmitDocument::from(text)
}

pub(super) fn dynamic_source(
    ctx: &GenerateContext,
    prop: &IRProp<'_>,
    direct_key: String,
    key: String,
    value: String,
    map_computed: bool,
) -> EmitDocument {
    let text = cstr!("() => ({{ [({})]: {} }})", key, value);
    let mapped_value = if map_computed
        && prop.value_kind == PropValueKind::Expression
        && !prop.key.is_handler_key
        && key == direct_key
    {
        prop.values
            .first()
            .filter(|node| value == ctx.resolve_expression_node(node))
    } else {
        None
    };
    let Some(value_node) = mapped_value else {
        return plain_source(text);
    };
    let mut source = EmitDocument::plain("() => ({ [(");
    source.push_spanned(&computed_key(ctx, &prop.key));
    source.push_str(")]: ");
    source.push_spanned(&ctx.spanned_expression_node(value_node));
    source.push_str(" })");
    debug_assert_eq!(source.as_str(), text.as_str());
    source
}

pub(super) fn format_sources(sources: &[EmitDocument]) -> EmitDocument {
    let mut result = EmitDocument::plain("{ $: [");
    if sources.len() >= 2 {
        result.push_str("\n");
        for (i, source) in sources.iter().enumerate() {
            result.push_str("    ");
            result.push_spanned(source);
            if i < sources.len() - 1 {
                result.push_str(",");
            }
            result.push_str("\n");
        }
        result.push_str("  ] }");
    } else {
        if let Some(source) = sources.first() {
            result.push_spanned(source);
        }
        result.push_str("] }");
    }
    result
}

fn computed_key(ctx: &GenerateContext, key: &SimpleExpressionNode<'_>) -> EmitDocument {
    let document = ctx.spanned_expression_node(key);
    let (text, mut links) = document.into_parts();
    for link in links
        .iter_mut()
        .filter(|link| link.segment && link.name.is_some())
    {
        if let Some(delimiter) = link
            .authored
            .start
            .checked_sub(1)
            .filter(|&start| ctx.source.as_bytes().get(start as usize) == Some(&b'['))
        {
            // Official computed-key names anchor at their opening bracket.
            link.authored.start = delimiter;
        }
    }
    EmitDocument::from_parts(text, links)
}
