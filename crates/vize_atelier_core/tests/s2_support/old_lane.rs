//! The legacy-lane projection: the DOM-output-determining facts of every
//! `IfNode` and every `ForNode` after the shipped transform ran, in
//! document order.

use vize_atelier_core::{ExpressionNode, ForNode, IfBranchNode, PropNode, TemplateChildNode};
use vize_carton::String;

/// A branch key as the legacy transform holds it (`user_key`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OldKey {
    /// No key prop was extracted.
    None,
    /// A static `key` attribute; the payload is its authored value
    /// (`None` for a bare attribute, which never collides).
    Static(Option<String>),
    /// A `:key` binding: the trimmed expression text (`None` = a
    /// compound rebuild), plus whether the legacy arg-content match
    /// admitted a dynamic-argument spelling (`:[key]` — the recorded
    /// quirk S2 mirrors on ordinary carriers and still counts for
    /// residual wrapper cases).
    Dynamic {
        /// The trimmed expression text.
        text: Option<String>,
        /// The spelling's argument is dynamic.
        dynamic_arg: bool,
    },
}

/// One branch's projected facts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OldBranch {
    /// `None` = unconditional; `Some(None)` = a compound rebuild with no
    /// single source text; `Some(Some(text))` = the trimmed source text.
    pub condition: Option<Option<String>>,
    /// The extracted key prop.
    pub key: OldKey,
    /// Whether the branch carried `<template v-if>` (its wrapper has no
    /// S2 attribute surface, so key comparison is counted instead).
    pub template_if: bool,
}

/// One `v-if` chain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OldChain {
    /// The branches, in authored order.
    pub branches: Vec<OldBranch>,
}

/// One expression position as the legacy lane holds it: `None` = a
/// compound rebuild with no single source text; `Some(text)` = the
/// trimmed source text.
pub type OldText = Option<String>;

/// One `v-for`'s projected facts — the binding surface DOM codegen
/// consumes (`renderList(source, (value, key, index) => ...)`). The
/// iterated element's `key` prop is deliberately absent here: the
/// legacy lane never lifts it (codegen reads it per vnode), so it is
/// element surface, not for structure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OldFor {
    /// The iterated source.
    pub source: OldText,
    /// The value/key/index alias positions; the outer `None` = the
    /// position was not authored.
    pub value: Option<OldText>,
    /// The second position (object key).
    pub key: Option<OldText>,
    /// The third position (index).
    pub index: Option<OldText>,
}

/// Collect every chain and every for under `children`, outer before
/// nested, document order — the same order the S2 projection walks.
pub fn collect(
    children: &[TemplateChildNode<'_>],
    chains: &mut Vec<OldChain>,
    fors: &mut Vec<OldFor>,
) {
    for child in children {
        match child {
            TemplateChildNode::Element(element) => collect(&element.children, chains, fors),
            TemplateChildNode::If(node) => {
                chains.push(OldChain {
                    branches: node.branches.iter().map(project_branch).collect(),
                });
                for branch in node.branches.iter() {
                    collect(&branch.children, chains, fors);
                }
            }
            TemplateChildNode::IfBranch(branch) => collect(&branch.children, chains, fors),
            TemplateChildNode::For(node) => {
                fors.push(project_for(node));
                collect(&node.children, chains, fors);
            }
            TemplateChildNode::Text(_)
            | TemplateChildNode::Comment(_)
            | TemplateChildNode::Interpolation(_)
            | TemplateChildNode::TextCall(_)
            | TemplateChildNode::CompoundExpression(_)
            | TemplateChildNode::Hoisted(_) => {}
        }
    }
}

fn text(expression: &ExpressionNode<'_>) -> OldText {
    match expression {
        ExpressionNode::Simple(simple) => Some(String::from(simple.content.trim())),
        ExpressionNode::Compound(_) => None,
    }
}

fn project_for(node: &ForNode<'_>) -> OldFor {
    OldFor {
        source: text(&node.source),
        value: node.value_alias.as_ref().map(text),
        key: node.key_alias.as_ref().map(text),
        index: node.object_index_alias.as_ref().map(text),
    }
}

fn project_branch(branch: &IfBranchNode<'_>) -> OldBranch {
    let condition = branch
        .condition
        .as_ref()
        .map(|expression| match expression {
            ExpressionNode::Simple(simple) => Some(String::from(simple.content.trim())),
            ExpressionNode::Compound(_) => None,
        });
    let key = match &branch.user_key {
        None => OldKey::None,
        Some(PropNode::Attribute(attribute)) => OldKey::Static(
            attribute
                .value
                .as_ref()
                .map(|value| String::from(value.content)),
        ),
        Some(PropNode::Directive(dir)) => OldKey::Dynamic {
            text: dir.exp.as_ref().and_then(|exp| match exp {
                ExpressionNode::Simple(simple) => Some(String::from(simple.content.trim())),
                ExpressionNode::Compound(_) => None,
            }),
            dynamic_arg: match dir.arg.as_ref() {
                Some(ExpressionNode::Simple(arg)) => !arg.is_static,
                Some(ExpressionNode::Compound(_)) => true,
                None => false,
            },
        },
    };
    OldBranch {
        condition,
        key,
        template_if: branch.is_template_if,
    }
}
