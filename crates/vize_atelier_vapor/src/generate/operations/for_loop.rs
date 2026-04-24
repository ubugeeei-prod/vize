use crate::ir::ForIRNode;
use vize_carton::{cstr, FxHashMap, String, ToCompactString};

use super::{
    super::{
        context::{ForScope, GenerateContext},
        generate_block,
    },
    insertion::{block_requires_parent_insertion_state, emit_insertion_state},
};

/// Generate For
pub(super) fn generate_for(
    ctx: &mut GenerateContext,
    for_node: &ForIRNode<'_>,
    element_template_map: &FxHashMap<usize, usize>,
) {
    ctx.use_helper("createFor");
    emit_insertion_state(ctx, for_node.parent, for_node.anchor);

    let depth = ctx.for_scopes.len();
    let source = if for_node.source.is_static {
        ["(", for_node.source.content.as_str(), ")"].concat()
    } else {
        let resolved = ctx.resolve_expression(&for_node.source.content);
        ["(", &resolved, ")"].concat()
    };

    let value_alias = for_node.value.as_ref().map(|v| v.content.clone());
    let key_alias = for_node.key.as_ref().map(|k| k.content.clone());

    // Build parameter list using _for_item0, _for_key0 naming
    let for_item_var = cstr!("_for_item{}", depth);
    let for_key_var = cstr!("_for_key{}", depth);

    let params: String = if key_alias.is_some() {
        [for_item_var.as_str(), ", ", for_key_var.as_str()]
            .concat()
            .into()
    } else {
        for_item_var.clone()
    };

    // Push for scope before generating body
    let scope = ForScope {
        value_alias: value_alias.clone(),
        key_alias: key_alias.clone(),
        index_alias: for_node.index.as_ref().map(|i| i.content.clone()),
        depth,
    };
    ctx.for_scopes.push(scope);

    let was_fragment = ctx.is_fragment;
    ctx.is_fragment = true;

    let for_id_str = for_node.id.to_compact_string();
    ctx.push_line(
        &[
            "const n",
            &for_id_str,
            " = _createFor(() => ",
            &source,
            ", (",
            &params,
            ") => {",
        ]
        .concat(),
    );
    ctx.indent();
    if block_requires_parent_insertion_state(&for_node.render) {
        emit_insertion_state(ctx, for_node.parent, for_node.anchor);
    }
    ctx.push_component_scope();
    generate_block(ctx, &for_node.render, element_template_map);
    ctx.pop_component_scope();
    ctx.deindent();

    // Generate key function if key_prop is provided
    let key_func = generate_for_key_function(for_node);

    // Check if this is a range-based for (source is a number literal)
    let is_range = for_node.source.content.as_str().parse::<f64>().is_ok();

    // Determine memo flag: 4 = range, 1 = only child of parent (nested v-for)
    let memo_flag = if is_range {
        Some("4")
    } else if for_node.only_child && was_fragment {
        // only_child flag is for nested v-for inside another element
        Some("1")
    } else {
        None
    };

    if let Some(key_fn) = key_func {
        if let Some(flag) = memo_flag {
            ctx.push_line(&["}, ", &key_fn, ", ", flag, ")"].concat());
        } else {
            ctx.push_line(&["}, ", &key_fn, ")"].concat());
        }
    } else {
        ctx.push_line("})");
    }

    ctx.is_fragment = was_fragment;
    ctx.for_scopes.pop();
}

/// Generate key function for v-for
fn generate_for_key_function(for_node: &ForIRNode<'_>) -> Option<String> {
    if let Some(ref key_prop) = for_node.key_prop {
        let key_expr = &key_prop.content;
        // Build params: (value_alias) or (value_alias, key_alias)
        let value_name = for_node
            .value
            .as_ref()
            .map(|v| v.content.as_str())
            .unwrap_or("_item");
        let key_name = for_node.key.as_ref().map(|k| k.content.as_str());

        let params = if let Some(k) = key_name {
            [value_name, ", ", k].concat()
        } else {
            value_name.to_compact_string().into()
        };

        Some(cstr!("({params}) => ({key_expr})"))
    } else {
        None
    }
}
