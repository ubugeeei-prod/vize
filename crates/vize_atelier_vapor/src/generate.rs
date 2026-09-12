//! Vapor code generation.
//!
//! Generates JavaScript code from Vapor IR.
mod context;
mod destructure;
mod entry;
mod expression;
mod expression_retained;
mod helpers;
mod operations;
mod setup;

use crate::ir::{BlockIRNode, CreateComponentIRNode, NegativeBranch, OperationNode};
use vize_carton::{FxHashMap, FxHashSet, String, ToCompactString, ensure_sufficient_stack};

use context::GenerateContext;
pub use entry::{
    VaporGenerateExperimentalOptions, VaporGenerateOptions, VaporGenerateResult, generate_vapor,
    generate_vapor_with_options, generate_vapor_with_options_and_experimentals,
};
use helpers::generate_effect;
use operations::generate_operation;

fn block_has_template_refs(block: &BlockIRNode<'_>) -> bool {
    ensure_sufficient_stack(|| {
        block.operation.iter().any(operation_has_template_refs)
            || block
                .effect
                .iter()
                .any(|effect| effect.operations.iter().any(operation_has_template_refs))
    })
}

fn operation_has_template_refs(op: &OperationNode<'_>) -> bool {
    match op {
        OperationNode::SetTemplateRef(_) => true,
        OperationNode::If(if_node) => {
            block_has_template_refs(&if_node.positive)
                || if_node
                    .negative
                    .as_ref()
                    .is_some_and(negative_branch_has_template_refs)
        }
        OperationNode::For(for_node) => block_has_template_refs(&for_node.render),
        OperationNode::CreateComponent(component) => component
            .slots
            .iter()
            .any(|slot| block_has_template_refs(&slot.block)),
        _ => false,
    }
}

fn negative_branch_has_template_refs(branch: &crate::ir::NegativeBranch<'_>) -> bool {
    match branch {
        crate::ir::NegativeBranch::Block(block) => block_has_template_refs(block),
        crate::ir::NegativeBranch::If(if_node) => {
            block_has_template_refs(&if_node.positive)
                || if_node
                    .negative
                    .as_ref()
                    .is_some_and(negative_branch_has_template_refs)
        }
    }
}

fn collect_custom_directives(block: &BlockIRNode<'_>) -> std::vec::Vec<String> {
    let mut directives = FxHashSet::default();
    collect_custom_directives_from_block(block, &mut directives);

    let mut directives: std::vec::Vec<_> = directives.into_iter().collect();
    directives.sort();
    directives
}

fn collect_custom_directives_from_block(
    block: &BlockIRNode<'_>,
    directives: &mut FxHashSet<String>,
) {
    for operation in block.operation.iter() {
        collect_custom_directives_from_operation(operation, directives);
    }

    for effect in block.effect.iter() {
        for operation in effect.operations.iter() {
            collect_custom_directives_from_operation(operation, directives);
        }
    }
}

fn collect_custom_directives_from_component(
    component: &CreateComponentIRNode<'_>,
    directives: &mut FxHashSet<String>,
) {
    for slot in component.slots.iter() {
        collect_custom_directives_from_block(&slot.block, directives);
    }
}

fn collect_custom_directives_from_negative_branch(
    branch: &NegativeBranch<'_>,
    directives: &mut FxHashSet<String>,
) {
    match branch {
        NegativeBranch::Block(block) => collect_custom_directives_from_block(block, directives),
        NegativeBranch::If(if_node) => {
            collect_custom_directives_from_block(&if_node.positive, directives);
            if let Some(negative) = &if_node.negative {
                collect_custom_directives_from_negative_branch(negative, directives);
            }
        }
    }
}

fn collect_custom_directives_from_operation(
    operation: &OperationNode<'_>,
    directives: &mut FxHashSet<String>,
) {
    match operation {
        OperationNode::Directive(directive) if !directive.builtin => {
            directives.insert(String::new(directive.name));
        }
        OperationNode::If(if_node) => {
            collect_custom_directives_from_block(&if_node.positive, directives);
            if let Some(negative) = &if_node.negative {
                collect_custom_directives_from_negative_branch(negative, directives);
            }
        }
        OperationNode::For(for_node) => {
            collect_custom_directives_from_block(&for_node.render, directives);
        }
        OperationNode::CreateComponent(component) => {
            collect_custom_directives_from_component(component, directives);
        }
        _ => {}
    }
}

fn directive_resolution_ident(name: &str) -> String {
    let mut ident = String::with_capacity(name.len());
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            ident.push(ch);
        } else {
            ident.push('_');
        }
    }
    ident
}

/// Generate block
fn generate_block(
    ctx: &mut GenerateContext,
    block: &BlockIRNode<'_>,
    element_template_map: &FxHashMap<usize, usize>,
) {
    ensure_sufficient_stack(|| generate_block_guarded(ctx, block, element_template_map));
}

fn generate_block_guarded(
    ctx: &mut GenerateContext,
    block: &BlockIRNode<'_>,
    element_template_map: &FxHashMap<usize, usize>,
) {
    // Instantiate templates for elements in this block's returns
    for element_id in block.returns.iter() {
        if let Some(&template_index) = element_template_map.get(element_id) {
            let mut line = String::with_capacity(32);
            line.push_str("const n");
            line.push_str(&element_id.to_compact_string());
            line.push_str(" = t");
            line.push_str(&template_index.to_compact_string());
            line.push_str("()");
            ctx.push_line(&line);
        }
    }

    // Generate ChildRef/NextRef operations first (before text refs, since text refs
    // may reference child nodes created by these operations)
    for op in block.operation.iter() {
        if matches!(op, OperationNode::ChildRef(_) | OperationNode::NextRef(_)) {
            generate_operation(ctx, op, element_template_map);
        }
    }

    // Generate text node references for any SetText operations in this block.
    // Skip _txt() for standalone text elements (interpolations with their own template)
    // since the element itself IS the text node.
    for op in block.operation.iter() {
        maybe_generate_text_ref(ctx, op);
    }
    for effect in block.effect.iter() {
        for op in effect.operations.iter() {
            maybe_generate_text_ref(ctx, op);
        }
    }

    // Generate remaining operations (skip ChildRef/NextRef already generated above)
    for op in block.operation.iter() {
        if !matches!(op, OperationNode::ChildRef(_) | OperationNode::NextRef(_)) {
            generate_operation(ctx, op, element_template_map);
        }
    }

    // Generate effects. Multiple reactive updates in the same block share a
    // single effect, matching Vue Vapor output for sibling dynamic children.
    if block.effect.len() == 1 {
        generate_effect(ctx, &block.effect[0], element_template_map);
    } else if !block.effect.is_empty() {
        ctx.use_helper("renderEffect");
        ctx.push_line("_renderEffect(() => {");
        ctx.indent();
        for effect in block.effect.iter() {
            for op in effect.operations.iter() {
                generate_operation(ctx, op, element_template_map);
            }
        }
        ctx.deindent();
        ctx.push_line("})");
    }

    // Generate return
    if !block.returns.is_empty() {
        let returns = block
            .returns
            .iter()
            .map(|r| ["n", &r.to_compact_string()].concat())
            .collect::<Vec<_>>()
            .join(", ");

        if block.returns.len() == 1 {
            ctx.push_line(&["return ", &returns].concat());
        } else {
            ctx.push_line(&["return [", &returns, "]"].concat());
        }
    }
}

fn maybe_generate_text_ref(ctx: &mut GenerateContext, op: &OperationNode<'_>) {
    if let OperationNode::SetText(set_text) = op
        && !ctx.standalone_text_elements.contains(&set_text.element)
        && !ctx.text_nodes.contains_key(&set_text.element)
    {
        ctx.use_helper("txt");
        let var_name = ctx.next_text_node(set_text.element);
        let mut line = String::with_capacity(32);
        line.push_str("const ");
        line.push_str(&var_name);
        line.push_str(" = _txt(n");
        line.push_str(&set_text.element.to_compact_string());
        line.push(')');
        ctx.push_line(&line);
    }
}

/// Collect root template indices from v-if branches (recursive for v-else-if chains)
fn collect_root_if_templates(
    if_node: &crate::ir::IfIRNode<'_>,
    element_template_map: &FxHashMap<usize, usize>,
    root_indices: &mut FxHashSet<usize>,
) {
    ensure_sufficient_stack(|| {
        collect_root_if_templates_guarded(if_node, element_template_map, root_indices);
    });
}

fn collect_root_if_templates_guarded(
    if_node: &crate::ir::IfIRNode<'_>,
    element_template_map: &FxHashMap<usize, usize>,
    root_indices: &mut FxHashSet<usize>,
) {
    // Only mark as root if the branch returns a single element
    if if_node.positive.returns.len() == 1 {
        let element_id = if_node.positive.returns[0];
        if let Some(&template_index) = element_template_map.get(&element_id) {
            root_indices.insert(template_index);
        }
    }
    // Handle negative branch
    if let Some(ref negative) = if_node.negative {
        match negative {
            crate::ir::NegativeBranch::Block(block) => {
                if block.returns.len() == 1 {
                    let element_id = block.returns[0];
                    if let Some(&template_index) = element_template_map.get(&element_id) {
                        root_indices.insert(template_index);
                    }
                }
            }
            crate::ir::NegativeBranch::If(nested_if) => {
                collect_root_if_templates(nested_if, element_template_map, root_indices);
            }
        }
    }
}

#[cfg(test)]
mod tests;
