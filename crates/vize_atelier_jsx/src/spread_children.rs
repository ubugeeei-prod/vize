//! Which backends can spread `{...items}` into the children (#6888).
//!
//! The lowering emits a VDOM Fragment block (`lower/spread.rs`). Vapor and SSR
//! have no raw-children representation, so they replace the block with the
//! previous stringified value and report the gap instead of emitting code
//! that references the client block helpers.

use vize_relief::{
    CompoundExpressionChild, CompoundExpressionNode, ElementNode, ExpressionNode,
    InterpolationNode, RootNode, TemplateChildNode,
};

use crate::diagnostics::JsxDiagnostic;
use crate::lower::boxed_in;

pub(crate) const SPREAD_BLOCK_OPEN: &str = "(_openBlock(), _createBlock(_Fragment, null, [...";
pub(crate) const SPREAD_BLOCK_CLOSE: &str = "], -2 /* BAIL */))";

const SPREAD_CHILD_UNSUPPORTED: &str = "spread children (`{...items}`) are only supported in VDOM output; Vapor and SSR stringify the value instead of spreading it";

/// Replace every spread-child block in `root` and report each one.
pub(crate) fn reject_spread_children<'a>(
    allocator: &'a vize_l0::Allocator,
    root: &mut RootNode<'a>,
    diagnostics: &mut Vec<JsxDiagnostic>,
) {
    for child in root.children.iter_mut() {
        visit(allocator, child, diagnostics);
    }
}

fn visit<'a>(
    allocator: &'a vize_l0::Allocator,
    child: &mut TemplateChildNode<'a>,
    diagnostics: &mut Vec<JsxDiagnostic>,
) {
    match child {
        TemplateChildNode::Element(el) => visit_element(allocator, el, diagnostics),
        TemplateChildNode::If(if_node) => {
            for branch in if_node.branches.iter_mut() {
                for child in branch.children.iter_mut() {
                    visit(allocator, child, diagnostics);
                }
            }
        }
        TemplateChildNode::For(for_node) => {
            for child in for_node.children.iter_mut() {
                visit(allocator, child, diagnostics);
            }
        }
        TemplateChildNode::CompoundExpression(compound) if is_spread_block(compound) => {
            diagnostics.push(JsxDiagnostic::error_at(
                SPREAD_CHILD_UNSUPPORTED,
                &compound.loc,
            ));
            compound.children.pop();
            let Some(CompoundExpressionChild::Simple(argument)) = compound.children.pop() else {
                return;
            };
            *child = TemplateChildNode::Interpolation(boxed_in(
                allocator,
                InterpolationNode {
                    content: ExpressionNode::Simple(argument),
                    loc: compound.loc.clone(),
                    #[cfg(feature = "legacy")]
                    raw: false,
                },
            ));
        }
        _ => {}
    }
}

fn visit_element<'a>(
    allocator: &'a vize_l0::Allocator,
    el: &mut ElementNode<'a>,
    diagnostics: &mut Vec<JsxDiagnostic>,
) {
    for child in el.children.iter_mut() {
        visit(allocator, child, diagnostics);
    }
}

/// Whether `compound` is a block built by `Lowerer::spread_block`.
fn is_spread_block(compound: &CompoundExpressionNode<'_>) -> bool {
    matches!(
        compound.children.as_slice(),
        [
            CompoundExpressionChild::String(SPREAD_BLOCK_OPEN),
            CompoundExpressionChild::Simple(_),
            CompoundExpressionChild::String(SPREAD_BLOCK_CLOSE),
        ]
    )
}

/// The Vue helpers a spread-child block names, with their bound aliases.
const SPREAD_HELPERS: [(&str, &str); 3] = [
    ("openBlock", "_openBlock"),
    ("createBlock", "_createBlock"),
    ("Fragment", "_Fragment"),
];

/// Whether any spread-child block remains in `root` (VDOM output).
pub(crate) fn has_spread_children(root: &RootNode<'_>) -> bool {
    root.children.iter().any(contains_spread_block)
}

/// Import the helpers the spread-child blocks reference. They are emitted as
/// literal code, which the core codegen does not track for its preamble.
pub(crate) fn import_spread_helpers(preamble: &mut vize_l0::String) {
    let mut specifiers = vize_l0::String::default();
    for (name, alias) in SPREAD_HELPERS {
        let bound = preamble.contains(vize_l0::cstr!(" as {alias}").as_str())
            || preamble.contains(vize_l0::cstr!("const {alias} ").as_str());
        if !bound {
            if !specifiers.is_empty() {
                specifiers.push_str(", ");
            }
            specifiers.push_str(name);
            specifiers.push_str(" as ");
            specifiers.push_str(alias);
        }
    }
    if specifiers.is_empty() {
        return;
    }
    if !preamble.is_empty() && !preamble.ends_with('\n') {
        preamble.push('\n');
    }
    preamble.push_str("import { ");
    preamble.push_str(&specifiers);
    preamble.push_str(" } from \"vue\"\n");
}

fn contains_spread_block(child: &TemplateChildNode<'_>) -> bool {
    match child {
        TemplateChildNode::Element(el) => el.children.iter().any(contains_spread_block),
        TemplateChildNode::If(if_node) => if_node
            .branches
            .iter()
            .any(|branch| branch.children.iter().any(contains_spread_block)),
        TemplateChildNode::For(for_node) => for_node.children.iter().any(contains_spread_block),
        TemplateChildNode::CompoundExpression(compound) => is_spread_block(compound),
        _ => false,
    }
}
