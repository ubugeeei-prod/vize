//! v-for generation functions.
//!
//! Generates code for v-for directives including the renderList callback,
//! fragment wrapping, and individual item rendering.

mod generate;
pub(crate) mod helpers;

use crate::ast::{ForNode, RuntimeHelper, TemplateChildNode};
use crate::transforms::v_memo::{get_memo_exp, has_v_memo};

use super::{
    children::generate_children, context::CodegenContext, expression::generate_expression,
};

use generate::generate_for_item;
use helpers::extract_for_params;
use vize_carton::String;
use vize_carton::ToCompactString;

#[allow(unused_imports)]
pub(crate) use helpers::{
    extract_destructure_params, get_element_key, is_numeric_source, is_valid_ident, split_top_level,
};

/// Generate for node
pub fn generate_for(ctx: &mut CodegenContext, for_node: &ForNode<'_>) {
    generate_for_inner(ctx, for_node)
}

fn generate_for_inner(ctx: &mut CodegenContext, for_node: &ForNode<'_>) {
    ctx.use_helper(RuntimeHelper::OpenBlock);
    ctx.use_helper(RuntimeHelper::CreateElementBlock);
    ctx.use_helper(RuntimeHelper::Fragment);
    ctx.use_helper(RuntimeHelper::RenderList);

    // Determine if this is a numeric range (stable) or dynamic list
    let is_stable = is_numeric_source(&for_node.source);

    // Check if children have keys
    let has_key = for_node.children.iter().any(|child| {
        if let TemplateChildNode::Element(el) = child {
            get_element_key(el).is_some()
        } else {
            false
        }
    });

    // Fragment type: 64 = STABLE, 128 = KEYED, 256 = UNKEYED
    let fragment_flag = if is_stable {
        64 // STABLE_FRAGMENT
    } else if has_key {
        128 // KEYED_FRAGMENT
    } else {
        256 // UNKEYED_FRAGMENT
    };

    ctx.push("(");
    ctx.push(ctx.helper(RuntimeHelper::OpenBlock));
    if is_stable {
        ctx.push("(), ");
    } else {
        ctx.push("(true), ");
    }
    ctx.push(ctx.helper(RuntimeHelper::CreateElementBlock));
    ctx.push("(");
    ctx.push(ctx.helper(RuntimeHelper::Fragment));
    ctx.push(", null, ");
    ctx.push(ctx.helper(RuntimeHelper::RenderList));
    ctx.push("(");
    generate_expression(ctx, &for_node.source);
    ctx.push(", (");

    // Collect callback parameter names for scope registration
    let mut callback_params: Vec<String> = Vec::new();

    // Value alias
    if let Some(value) = &for_node.value_alias {
        generate_expression(ctx, value);
        extract_for_params(value, &mut callback_params);
    } else {
        ctx.push("_item");
    }

    // Key alias
    if let Some(key) = &for_node.key_alias {
        ctx.push(", ");
        generate_expression(ctx, key);
        extract_for_params(key, &mut callback_params);
    }

    // Index alias
    if let Some(index) = &for_node.object_index_alias {
        ctx.push(", ");
        generate_expression(ctx, index);
        extract_for_params(index, &mut callback_params);
    }

    // Register callback params so they don't get _ctx. prefix
    ctx.add_slot_params(&callback_params);

    // Check if the single child has v-memo (v-for + v-memo optimization)
    let child_memo_exp = if for_node.children.len() == 1 {
        if let TemplateChildNode::Element(el) = &for_node.children[0] {
            if has_v_memo(el) {
                get_memo_exp(el)
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    if let Some(memo_exp) = child_memo_exp {
        // v-for + v-memo: special optimized pattern
        // Register withMemo helper (needed by Vue runtime even though we don't call it directly)
        ctx.use_helper(RuntimeHelper::WithMemo);

        // Extra parameters: __, ___, _cached
        // Add placeholder parameters to reach _cached position
        if for_node.key_alias.is_none() {
            ctx.push(", __");
        }
        if for_node.object_index_alias.is_none() {
            ctx.push(", ___");
        }
        ctx.push(", _cached) => {");
        ctx.indent();
        ctx.newline();

        // const _memo = ([deps])
        ctx.push("const _memo = (");
        generate_expression(ctx, memo_exp);
        ctx.push(")");
        ctx.newline();

        // if (_cached && _cached.key === key && _isMemoSame(_cached, _memo)) return _cached
        ctx.use_helper(RuntimeHelper::IsMemoSame);
        let key_exp = if let TemplateChildNode::Element(el) = &for_node.children[0] {
            get_element_key(el)
        } else {
            None
        };
        ctx.push("if (_cached && _cached.key === ");
        if let Some(key) = key_exp {
            generate_expression(ctx, key);
        } else {
            ctx.push("undefined");
        }
        ctx.push(" && ");
        ctx.push(ctx.helper(RuntimeHelper::IsMemoSame));
        ctx.push("(_cached, _memo)) return _cached");
        ctx.newline();

        // const _item = (element generation)
        ctx.push("const _item = ");

        // Set in_v_for flag
        let prev_in_v_for = ctx.in_v_for;
        ctx.in_v_for = true;

        // Skip v-memo in generate_for_item since we handle it here
        ctx.skip_v_memo = true;
        generate_for_item(ctx, &for_node.children[0], is_stable);
        ctx.skip_v_memo = false;

        ctx.in_v_for = prev_in_v_for;

        ctx.newline();
        ctx.push("_item.memo = _memo");
        ctx.newline();
        ctx.push("return _item");

        // Unregister callback params
        ctx.remove_slot_params(&callback_params);

        ctx.deindent();
        ctx.newline();

        // Close callback, add _cache and index args
        let cache_index = ctx.next_cache_index();
        let flag_name = match fragment_flag {
            64 => "STABLE_FRAGMENT",
            128 => "KEYED_FRAGMENT",
            256 => "UNKEYED_FRAGMENT",
            _ => "FRAGMENT",
        };
        ctx.push("}, _cache, ");
        ctx.push(&cache_index.to_compact_string());
        ctx.push("), ");
        ctx.push(&fragment_flag.to_compact_string());
        ctx.push(" /* ");
        ctx.push(flag_name);
        ctx.push(" */))");
    } else {
        // Standard v-for (no v-memo)
        ctx.push(") => {");
        ctx.indent();
        ctx.newline();
        ctx.push("return ");

        // Set in_v_for flag so slot stability is DYNAMIC inside v-for
        let prev_in_v_for = ctx.in_v_for;
        ctx.in_v_for = true;

        // Generate child as block (not regular node)
        if for_node.children.len() == 1 {
            generate_for_item(ctx, &for_node.children[0], is_stable);
        } else {
            generate_children(ctx, &for_node.children);
        }

        // Restore in_v_for flag
        ctx.in_v_for = prev_in_v_for;

        // Unregister callback params
        ctx.remove_slot_params(&callback_params);

        ctx.deindent();
        ctx.newline();
        // Close with fragment flag
        let flag_name = match fragment_flag {
            64 => "STABLE_FRAGMENT",
            128 => "KEYED_FRAGMENT",
            256 => "UNKEYED_FRAGMENT",
            _ => "FRAGMENT",
        };
        ctx.push("}), ");
        ctx.push(&fragment_flag.to_compact_string());
        ctx.push(" /* ");
        ctx.push(flag_name);
        ctx.push(" */))");
    }
}

// Note: Directive skipping behavior (v-for with custom directives, :key handling)
// is tested via SFC snapshot tests in tests/fixtures/sfc/patches.toml.
