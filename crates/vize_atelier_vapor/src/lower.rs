//! Vapor IR lowering.
//!
//! Lowers the template AST into Vapor IR for code generation.

mod context;
mod control;
mod directive;
mod element;
mod text;

use vize_carton::{Allocator, String, Vec};

use crate::ir::{BlockIRNode, RootIRNode};
use vize_atelier_core::{RootNode, TemplateChildNode};

use context::TransformContext;
use control::{transform_for_node, transform_if_node};
use element::transform_element;
use text::{transform_interpolation, transform_text};

/// Transform AST to Vapor IR
pub fn transform_to_ir<'a>(
    allocator: &'a Allocator,
    root: &RootNode<'a>,
    source: &'a str,
) -> RootIRNode<'a> {
    transform_to_ir_with_diagnostics(allocator, root, source).0
}

pub(crate) fn transform_to_ir_with_diagnostics<'a>(
    allocator: &'a Allocator,
    root: &RootNode<'a>,
    source: &'a str,
) -> (RootIRNode<'a>, std::vec::Vec<String>) {
    let mut ctx = TransformContext::new(allocator, source);

    vize_atelier_core::walk_probe::record_walk(
        vize_atelier_core::walk_probe::WalkStage::VaporLower,
    );

    // Create block for root
    let block = transform_children(&mut ctx, &root.children);

    (
        RootIRNode {
            node: RootNode::new(allocator, ""),
            source,
            template: Default::default(),
            template_index_map: Default::default(),
            root_template_indexes: Vec::new_in(&allocator),
            component: Vec::new_in(&allocator),
            directive: Vec::new_in(&allocator),
            block,
            has_template_ref: false,
            has_deferred_v_show: false,
            templates: ctx.templates,
            element_template_map: ctx.element_template_map,
            standalone_text_elements: ctx.standalone_text_elements,
        },
        ctx.diagnostics,
    )
}

/// Transform children nodes
pub(crate) fn transform_children<'a>(
    ctx: &mut TransformContext<'a>,
    children: &[TemplateChildNode<'a>],
) -> BlockIRNode<'a> {
    vize_atelier_core::walk_probe::record_visits(
        vize_atelier_core::walk_probe::WalkStage::VaporLower,
        children.len(),
    );
    let mut block = BlockIRNode::new(ctx.allocator);
    // Note: Don't consume an ID for the block itself - element IDs should start from 0

    // Check if ALL children are text/interpolation (combined text case)
    let all_text_or_interp = children.len() > 1
        && children.iter().all(|c| {
            matches!(
                c,
                TemplateChildNode::Text(_) | TemplateChildNode::Interpolation(_)
            )
        })
        && children
            .iter()
            .any(|c| matches!(c, TemplateChildNode::Interpolation(_)));

    if all_text_or_interp {
        // Combined text/interpolation: create a single text element with space template
        transform_combined_block_text(ctx, children, &mut block);
        return block;
    }

    for child in children {
        match child {
            TemplateChildNode::Element(el) => {
                transform_element(ctx, el, &mut block);
            }
            TemplateChildNode::Text(text) => {
                transform_text(ctx, text, &mut block);
            }
            TemplateChildNode::Interpolation(interp) => {
                transform_interpolation(ctx, interp, &mut block);
            }
            TemplateChildNode::If(if_node) => {
                transform_if_node(ctx, if_node, &mut block);
            }
            TemplateChildNode::For(for_node) => {
                transform_for_node(ctx, for_node, &mut block);
            }
            TemplateChildNode::Comment(_) => {
                // Comments are ignored in Vapor mode
            }
            _ => {}
        }
    }

    block
}

/// Transform combined text/interpolation children at block level.
/// Creates a single text node with a space template and a SetText effect
/// that combines all text parts and interpolations.
fn transform_combined_block_text<'a>(
    ctx: &mut TransformContext<'a>,
    children: &[TemplateChildNode<'a>],
    block: &mut BlockIRNode<'a>,
) {
    use crate::ir::{OperationNode, SetTextIRNode};
    use vize_atelier_core::{ExpressionNode, SimpleExpressionNode, SourceLocation};
    use vize_carton::{Box, Vec};

    let element_id = ctx.next_id();

    // Consume IDs for remaining children (they won't be used, but keeps ID
    // allocation consistent with the expected output format)
    for _ in 1..children.len() {
        ctx.next_id();
    }

    // Register a space placeholder template
    ctx.add_template(element_id, vize_carton::String::from(" "));
    ctx.standalone_text_elements.insert(element_id);

    // Collect all text values
    let mut values = Vec::new_in(&ctx.allocator);
    for child in children.iter() {
        match child {
            TemplateChildNode::Text(text) => {
                let exp = SimpleExpressionNode::new(text.content, true, SourceLocation::STUB);
                values.push(Box::new_in(exp, &ctx.allocator));
            }
            TemplateChildNode::Interpolation(interp) => {
                if let ExpressionNode::Simple(simple) = &interp.content {
                    let exp = SimpleExpressionNode::new(
                        simple.content,
                        simple.is_static,
                        simple.loc.clone(),
                    );
                    values.push(Box::new_in(exp, &ctx.allocator));
                }
            }
            _ => {}
        }
    }

    let set_text = SetTextIRNode {
        element: element_id,
        values,
    };
    ctx.push_dynamic_operation(block, OperationNode::SetText(set_text));
    block.returns.push(element_id);
}

#[cfg(test)]
mod tests {
    use super::transform_to_ir;
    use vize_atelier_core::parser::parse;
    use vize_carton::Allocator;

    #[test]
    fn test_transform_simple_element() {
        let allocator = Allocator::new();
        let source = "<div>hello</div>";
        let (root, _) = parse(&allocator, source);
        let ir = transform_to_ir(&allocator, &root, source);

        assert!(!ir.block.returns.is_empty());
    }

    #[test]
    fn test_transform_nested_elements() {
        let allocator = Allocator::new();
        let source = "<div><span>nested</span></div>";
        let (root, _) = parse(&allocator, source);
        let ir = transform_to_ir(&allocator, &root, source);

        assert!(!ir.block.returns.is_empty());
    }
}
