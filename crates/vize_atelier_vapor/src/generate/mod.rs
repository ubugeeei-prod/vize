//! Vapor code generation.
//!
//! Generates JavaScript code from Vapor IR.

mod context;
mod helpers;
mod operations;
mod setup;

use std::fmt::Write;

use crate::ir::{BlockIRNode, OperationNode, RootIRNode};
use vize_carton::{FxHashMap, FxHashSet, String, ToCompactString};

use context::GenerateContext;
use helpers::generate_effect;
use operations::generate_operation;
use setup::{collect_delegate_events, escape_template, generate_imports};

/// Vapor code generation result
pub struct VaporGenerateResult {
    /// Generated code
    pub code: String,
    /// Static templates
    pub templates: Vec<String>,
}

/// Generate Vapor code from IR
pub fn generate_vapor(ir: &RootIRNode<'_>) -> VaporGenerateResult {
    let mut ctx = GenerateContext::new(&ir.element_template_map, &ir.standalone_text_elements);

    // Template helper is always used if we have templates
    if !ir.templates.is_empty() {
        ctx.use_helper("template");
    }

    // Collect root template indices (templates used in top-level block returns
    // and in root-level v-if branches that return a single element)
    let mut root_template_indices: FxHashSet<usize> = FxHashSet::default();
    // Only mark as root if there's a single root return (not a fragment)
    if ir.block.returns.len() == 1 {
        let element_id = ir.block.returns[0];
        if let Some(&template_index) = ir.element_template_map.get(&element_id) {
            root_template_indices.insert(template_index);
        }
    }
    // Also mark templates from root-level v-if branches as root
    for op in ir.block.operation.iter() {
        if let OperationNode::If(if_node) = op {
            collect_root_if_templates(
                if_node,
                &ir.element_template_map,
                &mut root_template_indices,
            );
        }
    }

    // Generate template declarations (to separate string, we'll prepend imports later)
    let mut template_code = String::default();
    for (i, template) in ir.templates.iter().enumerate() {
        let is_root = root_template_indices.contains(&i);
        let is_svg = template.starts_with("<svg");
        match (is_root, is_svg) {
            (true, true) => writeln!(
                template_code,
                "const t{} = _template(\"{}\", true, 1)",
                i,
                escape_template(template)
            ),
            (true, false) => writeln!(
                template_code,
                "const t{} = _template(\"{}\", true)",
                i,
                escape_template(template)
            ),
            (false, true) => writeln!(
                template_code,
                "const t{} = _template(\"{}\", false, 1)",
                i,
                escape_template(template)
            ),
            (false, false) => writeln!(
                template_code,
                "const t{} = _template(\"{}\")",
                i,
                escape_template(template)
            ),
        }
        .ok();
    }

    // First pass: collect delegate events
    collect_delegate_events(&mut ctx, &ir.block);

    // Generate component function body first to collect used helpers
    ctx.push_line("export function render(_ctx) {");
    ctx.indent();

    // Generate block content (includes template instantiation, text nodes, operations, effects, return)
    generate_block(&mut ctx, &ir.block, &ir.element_template_map);

    ctx.deindent();
    ctx.push_line("}");

    // Generate delegate events code (after templates, before function)
    let mut delegate_code = String::default();
    if !ctx.delegate_events.is_empty() {
        ctx.use_helper("delegateEvents");
        let mut events: Vec<_> = ctx.delegate_events.iter().collect();
        events.sort();
        for event in events {
            writeln!(delegate_code, "_delegateEvents(\"{}\")", event).ok();
        }
    }

    // Now generate imports at the front with only used helpers
    let imports = generate_imports(&ctx);

    // Combine: imports + templates + delegate events + function body
    let mut final_code = imports;
    if !template_code.is_empty() {
        final_code.push_str(&template_code);
    }
    if !delegate_code.is_empty() {
        final_code.push_str(&delegate_code);
    }
    // Add blank line before function
    if !final_code.is_empty() {
        final_code.push('\n');
    }
    final_code.push_str(&ctx.code);

    VaporGenerateResult {
        code: final_code,
        templates: ir.templates.iter().cloned().collect(),
    }
}

/// Generate block
fn generate_block(
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

    // Generate text node references for effects in this block.
    // Skip _txt() for standalone text elements (interpolations with their own template)
    // since the element itself IS the text node.
    for effect in block.effect.iter() {
        for op in effect.operations.iter() {
            if let OperationNode::SetText(set_text) = op {
                if !ctx.standalone_text_elements.contains(&set_text.element) {
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
        }
    }

    // Check if we need setInsertionState for nested v-if/v-for
    let has_control_flow = block
        .operation
        .iter()
        .any(|op| matches!(op, OperationNode::If(_) | OperationNode::For(_)));
    let parent_element = if has_control_flow && ctx.is_fragment {
        // Find the parent template element (first element in returns that has a template)
        block
            .returns
            .iter()
            .find(|id| element_template_map.contains_key(id))
            .copied()
    } else {
        None
    };

    if let Some(parent_id) = parent_element {
        ctx.use_helper("setInsertionState");
        ctx.push_line_fmt(format_args!(
            "_setInsertionState(n{}, null, true)",
            parent_id
        ));
    }

    // Generate remaining operations (skip ChildRef/NextRef already generated above)
    for op in block.operation.iter() {
        if !matches!(op, OperationNode::ChildRef(_) | OperationNode::NextRef(_)) {
            generate_operation(ctx, op, element_template_map);
        }
    }

    // Generate effects
    for effect in block.effect.iter() {
        generate_effect(ctx, effect, element_template_map);
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

/// Collect root template indices from v-if branches (recursive for v-else-if chains)
fn collect_root_if_templates(
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
mod tests {
    use super::{generate_vapor, setup::escape_template};
    use crate::transform::transform_to_ir;
    use vize_atelier_core::parser::parse;
    use vize_carton::Bump;

    #[test]
    fn test_generate_simple() {
        let allocator = Bump::new();
        let (root, _) = parse(&allocator, "<div>hello</div>");
        let ir = transform_to_ir(&allocator, &root);
        let result = generate_vapor(&ir);

        assert!(!result.code.is_empty());
        assert!(result.code.contains("export function render"));
    }

    #[test]
    fn test_generate_with_event() {
        let allocator = Bump::new();
        let (root, _) = parse(&allocator, r#"<button @click="handleClick">Click</button>"#);
        let ir = transform_to_ir(&allocator, &root);
        let result = generate_vapor(&ir);

        assert!(result.code.contains("createInvoker"));
        assert!(result.code.contains("click"));
    }

    #[test]
    fn test_escape_template() {
        assert_eq!(escape_template("hello"), "hello");
        assert_eq!(escape_template("hello\nworld"), "hello\\nworld");
        assert_eq!(escape_template("hello\"world"), "hello\\\"world");
    }
}
