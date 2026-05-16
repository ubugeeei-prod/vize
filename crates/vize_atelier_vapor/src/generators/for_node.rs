//! For node code generation for Vapor mode.

use super::block::GenerateContext;
use crate::ir::{BlockIRNode, ForIRNode};
use vize_carton::{cstr, String, ToCompactString};

/// Generate for node code
pub fn generate_for<F>(ctx: &mut GenerateContext, for_node: &ForIRNode<'_>, generate_block: F)
where
    F: Fn(&mut GenerateContext, &BlockIRNode<'_>),
{
    let source = if for_node.source.is_static {
        cstr!("\"{}\"", for_node.source.content)
    } else {
        vize_carton::CompactString::from(for_node.source.content.as_str())
    };

    let value_name = for_node
        .value
        .as_ref()
        .map(|v| v.content.as_str())
        .unwrap_or("_item");

    let key_name = for_node.key.as_ref().map(|k| k.content.as_str());
    let index_name = for_node.index.as_ref().map(|i| i.content.as_str());

    let params = build_params(value_name, key_name, index_name);

    ctx.push_line_fmt(format_args!("_createFor(() => {source}, ({params}) => {{"));
    ctx.indent();
    generate_block(ctx, &for_node.render);
    ctx.deindent();

    // Add key function if key prop is specified
    if let Some(ref key_prop) = for_node.key_prop {
        let key_expr = if key_prop.is_static {
            cstr!("\"{}\"", key_prop.content)
        } else {
            vize_carton::CompactString::from(key_prop.content.as_str())
        };
        ctx.push_line_fmt(format_args!("}}, ({params}) => {key_expr})"));
    } else {
        ctx.push_line("})");
    }
}

/// Build parameter string for for callback
fn build_params(value: &str, key: Option<&str>, index: Option<&str>) -> String {
    match (key, index) {
        (Some(k), Some(i)) => cstr!("{value}, {k}, {i}"),
        (Some(k), None) => cstr!("{value}, {k}"),
        _ => value.to_compact_string(),
    }
}

/// Generate for with memo (optimized)
pub fn generate_for_memo<F>(ctx: &mut GenerateContext, for_node: &ForIRNode<'_>, generate_block: F)
where
    F: Fn(&mut GenerateContext, &BlockIRNode<'_>),
{
    let source = if for_node.source.is_static {
        cstr!("\"{}\"", for_node.source.content)
    } else {
        vize_carton::CompactString::from(for_node.source.content.as_str())
    };

    let value_name = for_node
        .value
        .as_ref()
        .map(|v| v.content.as_str())
        .unwrap_or("_item");

    let params = build_params(
        value_name,
        for_node.key.as_ref().map(|k| k.content.as_str()),
        for_node.index.as_ref().map(|i| i.content.as_str()),
    );

    if for_node.once {
        // Non-reactive for loop
        ctx.push_line_fmt(format_args!(
            "_createForStatic(() => {source}, ({params}) => {{"
        ));
    } else {
        ctx.push_line_fmt(format_args!("_createFor(() => {source}, ({params}) => {{"));
    }

    ctx.indent();
    generate_block(ctx, &for_node.render);
    ctx.deindent();
    ctx.push_line("})");
}

/// Check if for loop can be optimized
pub fn can_optimize_for(for_node: &ForIRNode<'_>) -> bool {
    for_node.once || for_node.only_child
}

#[cfg(test)]
mod tests {
    use super::build_params;

    #[test]
    fn test_build_params_simple() {
        let result = build_params("item", None, None);
        assert_eq!(result, "item");
    }

    #[test]
    fn test_build_params_with_key() {
        let result = build_params("item", Some("key"), None);
        assert_eq!(result, "item, key");
    }

    #[test]
    fn test_build_params_with_all() {
        let result = build_params("value", Some("key"), Some("index"));
        assert_eq!(result, "value, key, index");
    }
}
