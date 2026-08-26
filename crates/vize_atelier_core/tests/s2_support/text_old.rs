//! The legacy half of the text projection ([`super::text`]): the
//! transformed legacy tree re-grouped into units exactly as DOM codegen
//! groups them — maximal runs of consecutive text/interpolation
//! children (`crates/vize_atelier_core/src/codegen/children.rs`; a
//! comment or any other node kind breaks the run), rawtext-content
//! subtrees excluded per the lane-neutral rule.

use vize_atelier_core::{ExpressionNode, TemplateChildNode};
use vize_carton::String;

use super::text::{TPart, TUnit};

/// The rawtext-content exemption list — `vize_s1_to_s2::lower`'s
/// (`RAWTEXT_TAGS`), applied to the authored tag on both sides.
pub fn is_rawtext_tag(tag: &str) -> bool {
    matches!(
        tag,
        "script"
            | "style"
            | "textarea"
            | "title"
            | "iframe"
            | "noscript"
            | "xmp"
            | "listing"
            | "plaintext"
    )
}

/// Collect every unit under `children`, document order, outer before
/// nested — the same order the S2 projection walks.
pub fn collect_units(children: &[TemplateChildNode<'_>], units: &mut Vec<TUnit>) {
    walk(children, false, units);
}

fn part_of(child: &TemplateChildNode<'_>) -> Option<TPart> {
    match child {
        TemplateChildNode::Text(text) => Some(TPart {
            dynamic: false,
            text: Some(String::from(text.content)),
        }),
        TemplateChildNode::Interpolation(node) => Some(TPart {
            dynamic: true,
            text: match &node.content {
                ExpressionNode::Simple(simple) => Some(String::from(simple.content.trim())),
                ExpressionNode::Compound(_) => None,
            },
        }),
        _ => None,
    }
}

fn walk(children: &[TemplateChildNode<'_>], in_rawtext: bool, units: &mut Vec<TUnit>) {
    let mut run: Vec<TPart> = Vec::new();
    for child in children {
        if !in_rawtext && let Some(part) = part_of(child) {
            run.push(part);
            continue;
        }
        flush(&mut run, units);
        match child {
            TemplateChildNode::Element(element) => walk(
                &element.children,
                in_rawtext || is_rawtext_tag(element.tag),
                units,
            ),
            TemplateChildNode::If(node) => {
                for branch in node.branches.iter() {
                    walk(&branch.children, in_rawtext, units);
                }
            }
            TemplateChildNode::IfBranch(branch) => walk(&branch.children, in_rawtext, units),
            TemplateChildNode::For(node) => walk(&node.children, in_rawtext, units),
            _ => {}
        }
    }
    flush(&mut run, units);
}

fn flush(run: &mut Vec<TPart>, units: &mut Vec<TUnit>) {
    if run.is_empty() {
        return;
    }
    units.push(TUnit {
        parts: core::mem::take(run),
        compound: false,
    });
}
