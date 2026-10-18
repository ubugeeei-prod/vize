use crate::ir::ForIRNode;
use vize_atelier_core::codegen::document::EmitDocument;
use vize_carton::{FxHashMap, String, ToCompactString, cstr};

use super::{
    super::{
        context::GenerateContext,
        destructure::{needs_parsed_reads, parse_destructure_bindings},
        generate_block,
    },
    block_shape::{fragment_rows, single_node_rows},
    insertion::emit_insertion_state,
};

/// Generate For
pub(super) fn generate_for(
    ctx: &mut GenerateContext,
    for_node: &ForIRNode<'_>,
    element_template_map: &FxHashMap<usize, usize>,
) {
    if for_node.match_scope {
        super::match_scope::generate_match_scope(ctx, for_node, element_template_map);
        return;
    }
    ctx.use_helper("createFor");
    emit_insertion_state(ctx, for_node.parent, for_node.anchor);

    let depth = ctx.for_scopes.len();
    let mut source = EmitDocument::plain("(");
    if for_node.source.is_static {
        source.push_str(for_node.source.content);
    } else {
        source.push_spanned(&ctx.spanned_expression_node(&for_node.source));
    }
    source.push_str(")");

    let value_alias = for_node.value.as_ref().map(|v| v.content);
    let key_alias = for_node.key.as_ref().map(|k| k.content);
    let index_alias = for_node.index.as_ref().map(|i| i.content);

    // Build parameter list using _for_item0, _for_key0 naming
    let for_item_var = cstr!("_for_item{}", depth);
    let for_key_var = cstr!("_for_key{}", depth);
    let for_index_var = cstr!("_for_index{}", depth);

    let mut params = for_item_var.clone();
    if key_alias.is_some() || index_alias.is_some() {
        params.push_str(", ");
        params.push_str(&for_key_var);
    }
    if index_alias.is_some() {
        params.push_str(", ");
        params.push_str(&for_index_var);
    }

    // Push for scope before generating body
    ctx.push_for_scope(value_alias, key_alias, index_alias);

    let slot_root = ctx.transition_slot
        && for_node.parent.is_none()
        && for_node.key_prop.is_some()
        && matches!(for_node.render.returns.as_slice(), [id] if element_template_map.contains_key(id));
    let previous_transition = ctx.transition_slot;
    ctx.transition_slot = false;
    let was_fragment = ctx.is_fragment;
    ctx.is_fragment = true;

    // `_createFor(` opens at the authored element carrying `v-for`.
    let for_id_str = for_node.id.to_compact_string();
    let mut head = EmitDocument::plain(&["const n", &for_id_str, " = "].concat());
    head.push_spanned(&ctx.spanned_at("_createFor(", ctx.unit_of(&for_node.source)));
    head.push_str("() => ");
    head.push_spanned(&source);
    head.push_str(&[", (", &params, ") => {"].concat());
    ctx.push_line_spanned(&head);
    ctx.indent();
    ctx.push_component_scope();
    // A `v-if` that is a loop row renders its branches as fragments.
    let row_ifs = for_node
        .render
        .operation
        .iter()
        .filter_map(|operation| match operation {
            crate::ir::OperationNode::If(node) if for_node.render.returns.contains(&node.id) => {
                Some(node.id)
            }
            _ => None,
        })
        .collect::<std::vec::Vec<_>>();
    let outer_row_ifs = core::mem::replace(&mut ctx.loop_row_ifs, row_ifs);
    generate_block(ctx, &for_node.render, element_template_map);
    ctx.loop_row_ifs = outer_row_ifs;
    ctx.pop_component_scope();
    ctx.deindent();

    // Generate key function if key_prop is provided
    let key_func = generate_for_key_function(ctx, for_node);

    // Check if this is a range-based for (source is a number literal)
    let is_range = for_node.source.content.parse::<f64>().is_ok();

    // The runtime's `createFor` flags, as Vue's compiler encodes them.
    let table = [
        (for_node.only_child, 1, "FAST_REMOVE"),
        (for_node.component, 2, "IS_COMPONENT"),
        (fragment_rows(&for_node.render), 16, "IS_FRAGMENT"),
        (
            slot_root
                || !for_node.component && single_node_rows(&for_node.render, element_template_map),
            8,
            "IS_SINGLE_NODE",
        ),
        (is_range, 4, "ONCE"),
        (slot_root, 32, "SLOT_ROOT"),
        (
            for_node.wrapped_rows && !previous_transition,
            64,
            "WRAPPED_ROWS",
        ),
    ];
    let flags = table
        .iter()
        .filter(|(set, _, _)| *set)
        .fold(0u32, |flags, (_, bit, _)| flags | bit);
    ctx.push_indent();
    ctx.push("}");
    if let Some(key_fn) = &key_func {
        ctx.push(", ");
        ctx.push(key_fn);
    }
    if flags != 0 {
        use std::fmt::Write as _;
        if key_func.is_none() {
            ctx.push(", undefined");
        }
        let _ = write!(ctx, ", {flags} /* ");
        for (index, (_, _, name)) in table.iter().filter(|(set, _, _)| *set).enumerate() {
            if index > 0 {
                ctx.push(", ");
            }
            ctx.push(name);
        }
        ctx.push(" */");
    }
    ctx.push(")\n");

    ctx.transition_slot = previous_transition;
    ctx.is_fragment = was_fragment;
    ctx.for_scopes.pop();
}

/// Generate key function for v-for
fn generate_for_key_function(
    ctx: &mut GenerateContext<'_>,
    for_node: &ForIRNode<'_>,
) -> Option<String> {
    if let Some(ref key_prop) = for_node.key_prop {
        let key_expr = resolve_key_expression(ctx, for_node, key_prop);
        // Build params: (value_alias) or (value_alias, key_alias)
        let value_name = match for_node.value.as_ref().map(|v| v.content) {
            Some(pattern)
                if pattern.trim_start().starts_with(['{', '[']) && needs_parsed_reads(pattern) =>
            {
                ctx.resolve_pattern_parameter(pattern)
            }
            Some(alias) => alias.to_compact_string(),
            None => "_item".to_compact_string(),
        };
        let key_name = for_node.key.as_ref().map(|k| k.content);
        let index_name = for_node.index.as_ref().map(|i| i.content);

        let mut params = value_name;
        if key_name.is_some() || index_name.is_some() {
            params.push_str(", ");
            params.push_str(key_name.unwrap_or("_key"));
        }
        if let Some(index) = index_name {
            params.push_str(", ");
            params.push_str(index);
        }

        Some(cstr!("({params}) => ({key_expr})"))
    } else {
        None
    }
}

fn resolve_key_expression(
    ctx: &GenerateContext<'_>,
    for_node: &ForIRNode<'_>,
    key_prop: &vize_atelier_core::SimpleExpressionNode<'_>,
) -> String {
    // Node-aware: a retained key AST is consumed instead of reparsed.
    let mut resolved = ctx.resolve_expression_node(key_prop);
    let Some(current_scope) = ctx.for_scopes.last() else {
        return resolved;
    };

    if let Some(value) = for_node.value.as_ref() {
        // The key callback destructures the raw item itself, so a parsed
        // read goes back to the local it stands for, longest read first.
        let mut reads = current_scope
            .read_overrides
            .iter()
            .collect::<std::vec::Vec<_>>();
        reads.sort_by_key(|binding| std::cmp::Reverse(binding.path.len()));
        for binding in reads {
            if resolved.contains(binding.path.as_str()) {
                resolved = resolved
                    .replace(binding.path.as_str(), binding.local.as_str())
                    .into();
            }
        }
        restore_current_alias_reference(
            &mut resolved,
            &cstr!("_for_item{}.value", current_scope.depth),
            value.content,
        );
    }
    if let Some(key) = for_node.key.as_ref() {
        restore_current_alias_reference(
            &mut resolved,
            &cstr!("_for_key{}.value", current_scope.depth),
            key.content,
        );
    }
    if let Some(index) = for_node.index.as_ref() {
        restore_current_alias_reference(
            &mut resolved,
            &cstr!("_for_index{}.value", current_scope.depth),
            index.content,
        );
    }

    resolved
}

fn restore_current_alias_reference(expr: &mut String, reference: &str, alias: &str) {
    if is_simple_local_alias(alias) && expr.contains(reference) {
        *expr = expr.replace(reference, alias).into();
    } else if alias.trim_start().starts_with(['{', '[', '(']) {
        let mut bindings = parse_destructure_bindings(alias);
        bindings.sort_by_key(|binding| std::cmp::Reverse(binding.path.len()));
        for binding in bindings {
            let property_reference = cstr!("{}{}", reference, binding.path);
            if expr.contains(property_reference.as_str()) {
                *expr = expr
                    .replace(property_reference.as_str(), binding.local.as_str())
                    .into();
            }
        }
    }
}

fn is_simple_local_alias(alias: &str) -> bool {
    oxc_syntax::identifier::is_identifier_name(alias)
}

#[cfg(test)]
mod tests {
    use super::restore_current_alias_reference;
    use vize_carton::ToCompactString;

    #[test]
    fn restores_destructured_key_alias_references() {
        let mut expr = "_for_item0.value.name + _for_item0.value.user.id".to_compact_string();

        restore_current_alias_reference(
            &mut expr,
            "_for_item0.value",
            "{ name: label, user: { id: userId } }",
        );

        assert_eq!(expr, "label + userId");
    }
}
