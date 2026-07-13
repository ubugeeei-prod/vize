//! Arena materialization for consumers that still use Relief's borrowed AST.
//!
//! Atlas retains [`ReliefSnapshot`](super::ReliefSnapshot) as the canonical
//! owned syntax product. This adapter recreates the arena view without parsing
//! source text again, allowing existing compiler and lint rule implementations
//! to consume one shared syntax artifact while they migrate to owned views.

#![allow(deprecated)]

use vize_carton::{Box as AllocBox, Bump, Vec as AllocVec};

use crate::{
    AttributeNode, CommentNode, CompoundExpressionChild, CompoundExpressionNode, DirectiveNode,
    ExpressionNode, ForNode, ForParseResult, IfBranchNode, IfNode, ImportItem, InterpolationNode,
    JsExpression, PropNode, RootNode, SimpleExpressionNode, TemplateChildNode, TextCallContent,
    TextCallNode, TextNode,
};

use super::{
    ReliefSnapshot, ReliefSnapshotNode, ReliefSnapshotNodeId, SnapshotAttribute, SnapshotComment,
    SnapshotCompoundChild, SnapshotCompoundExpression, SnapshotDirective, SnapshotExpression,
    SnapshotFor, SnapshotForParseResult, SnapshotIfBranch, SnapshotInterpolation, SnapshotProp,
    SnapshotSimpleExpression, SnapshotText, SnapshotTextCallContent,
};

impl ReliefSnapshot {
    /// Recreate Relief's arena AST from this owned syntax product.
    ///
    /// This copies already-parsed nodes into `allocator`; it never reads or
    /// parses source text. The returned tree may be transformed independently
    /// without mutating the cached snapshot used by other consumers.
    pub fn materialize<'a>(&self, allocator: &'a Bump) -> RootNode<'a> {
        let mut root = RootNode::new(allocator, self.source.clone());
        root.children = nodes(self, allocator, self.children());
        root.comments = self.comments.iter().map(comment).collect_in(allocator);
        root.helpers = self.helpers.iter().copied().collect_in(allocator);
        root.components = self.components.iter().cloned().collect_in(allocator);
        root.directives = self.directives.iter().cloned().collect_in(allocator);
        #[cfg(feature = "_legacy")]
        {
            root.filters = self.filters.iter().cloned().collect_in(allocator);
        }
        root.imports = self
            .imports
            .iter()
            .map(|item| ImportItem {
                exp: AllocBox::new_in(simple_expression(&item.expression, allocator), allocator),
                path: item.path.clone(),
            })
            .collect_in(allocator);
        root.temps = self.temps;
        root.loc = self.location.clone();
        root.transformed = self.transformed;
        root
    }
}

trait CollectIn<'a, T>: Iterator<Item = T> + Sized {
    fn collect_in(self, allocator: &'a Bump) -> AllocVec<'a, T> {
        let mut values = AllocVec::new_in(allocator);
        values.extend(self);
        values
    }
}

impl<'a, T, I> CollectIn<'a, T> for I where I: Iterator<Item = T> {}

fn nodes<'a>(
    snapshot: &ReliefSnapshot,
    allocator: &'a Bump,
    ids: &[ReliefSnapshotNodeId],
) -> AllocVec<'a, TemplateChildNode<'a>> {
    ids.iter()
        .map(|id| node(snapshot, allocator, *id))
        .collect_in(allocator)
}

fn node<'a>(
    snapshot: &ReliefSnapshot,
    allocator: &'a Bump,
    id: ReliefSnapshotNodeId,
) -> TemplateChildNode<'a> {
    match snapshot.node(id).expect("snapshot node IDs are internal") {
        ReliefSnapshotNode::Element(value) => {
            let mut element =
                crate::ElementNode::new(allocator, value.tag.clone(), value.location.clone());
            element.ns = value.namespace;
            element.tag_type = value.tag_type;
            element.props = value
                .props
                .iter()
                .map(|value| property(value, allocator))
                .collect_in(allocator);
            element.children = nodes(snapshot, allocator, value.children());
            element.is_self_closing = value.is_self_closing;
            element.inner_loc = value.inner_location.clone();
            element.hoisted_props_index = value.hoisted_props_index;
            TemplateChildNode::Element(AllocBox::new_in(element, allocator))
        }
        ReliefSnapshotNode::Text(value) => {
            TemplateChildNode::Text(AllocBox::new_in(text(value), allocator))
        }
        ReliefSnapshotNode::Comment(value) => {
            TemplateChildNode::Comment(AllocBox::new_in(comment(value), allocator))
        }
        ReliefSnapshotNode::Interpolation(value) => TemplateChildNode::Interpolation(
            AllocBox::new_in(interpolation(value, allocator), allocator),
        ),
        ReliefSnapshotNode::If(value) => {
            let mut branches = AllocVec::new_in(allocator);
            for branch_id in value.branches() {
                let ReliefSnapshotNode::IfBranch(branch) = snapshot
                    .node(*branch_id)
                    .expect("snapshot branch IDs are internal")
                else {
                    unreachable!("an If snapshot can only reference IfBranch nodes")
                };
                branches.push(if_branch(snapshot, allocator, branch));
            }
            TemplateChildNode::If(AllocBox::new_in(
                IfNode {
                    branches,
                    loc: value.location.clone(),
                },
                allocator,
            ))
        }
        ReliefSnapshotNode::IfBranch(value) => TemplateChildNode::IfBranch(AllocBox::new_in(
            if_branch(snapshot, allocator, value),
            allocator,
        )),
        ReliefSnapshotNode::For(value) => TemplateChildNode::For(AllocBox::new_in(
            for_node(snapshot, allocator, value),
            allocator,
        )),
        ReliefSnapshotNode::TextCall(value) => TemplateChildNode::TextCall(AllocBox::new_in(
            TextCallNode {
                content: text_call_content(&value.content, allocator),
                loc: value.location.clone(),
            },
            allocator,
        )),
        ReliefSnapshotNode::CompoundExpression(value) => TemplateChildNode::CompoundExpression(
            AllocBox::new_in(compound_expression(value, allocator), allocator),
        ),
        ReliefSnapshotNode::Hoisted(value) => TemplateChildNode::Hoisted(value.index),
    }
}

fn if_branch<'a>(
    snapshot: &ReliefSnapshot,
    allocator: &'a Bump,
    value: &SnapshotIfBranch,
) -> IfBranchNode<'a> {
    IfBranchNode {
        condition: value
            .condition
            .as_ref()
            .map(|value| expression(value, allocator)),
        children: nodes(snapshot, allocator, value.children()),
        user_key: value
            .user_key
            .as_ref()
            .map(|value| property(value, allocator)),
        is_template_if: value.is_template_if,
        loc: value.location.clone(),
    }
}

fn for_node<'a>(
    snapshot: &ReliefSnapshot,
    allocator: &'a Bump,
    value: &SnapshotFor,
) -> ForNode<'a> {
    ForNode {
        source: expression(&value.source, allocator),
        value_alias: value
            .value_alias
            .as_ref()
            .map(|value| expression(value, allocator)),
        key_alias: value
            .key_alias
            .as_ref()
            .map(|value| expression(value, allocator)),
        object_index_alias: value
            .object_index_alias
            .as_ref()
            .map(|value| expression(value, allocator)),
        parse_result: for_parse_result(&value.parse_result, allocator),
        children: nodes(snapshot, allocator, value.children()),
        loc: value.location.clone(),
    }
}

fn property<'a>(value: &SnapshotProp, allocator: &'a Bump) -> PropNode<'a> {
    match value {
        SnapshotProp::Attribute(value) => {
            PropNode::Attribute(AllocBox::new_in(attribute(value), allocator))
        }
        SnapshotProp::Directive(value) => {
            PropNode::Directive(AllocBox::new_in(directive(value, allocator), allocator))
        }
    }
}

fn attribute(value: &SnapshotAttribute) -> AttributeNode {
    AttributeNode {
        name: value.name.clone(),
        name_loc: value.name_location.clone(),
        value: value.value.as_ref().map(text),
        loc: value.location.clone(),
    }
}

fn directive<'a>(value: &SnapshotDirective, allocator: &'a Bump) -> DirectiveNode<'a> {
    DirectiveNode {
        name: value.name.clone(),
        raw_name: value.raw_name.clone(),
        exp: value
            .expression
            .as_ref()
            .map(|value| expression(value, allocator)),
        arg: value
            .argument
            .as_ref()
            .map(|value| expression(value, allocator)),
        modifiers: value
            .modifiers
            .iter()
            .map(|modifier| simple_expression(modifier, allocator))
            .collect_in(allocator),
        for_parse_result: value
            .for_parse_result
            .as_ref()
            .map(|result| for_parse_result(result, allocator)),
        shorthand: value.shorthand,
        loc: value.location.clone(),
    }
}

fn expression<'a>(value: &SnapshotExpression, allocator: &'a Bump) -> ExpressionNode<'a> {
    match value {
        SnapshotExpression::Simple(value) => ExpressionNode::Simple(AllocBox::new_in(
            simple_expression(value, allocator),
            allocator,
        )),
        SnapshotExpression::Compound(value) => ExpressionNode::Compound(AllocBox::new_in(
            compound_expression(value, allocator),
            allocator,
        )),
    }
}

fn simple_expression<'a>(
    value: &SnapshotSimpleExpression,
    allocator: &'a Bump,
) -> SimpleExpressionNode<'a> {
    SimpleExpressionNode {
        content: value.content.clone(),
        is_static: value.is_static,
        const_type: value.const_type,
        loc: value.location.clone(),
        js_ast: value
            .js_raw
            .as_ref()
            .map(|raw| JsExpression::from_raw(raw.clone())),
        hoisted: None,
        identifiers: value
            .identifiers
            .as_ref()
            .map(|identifiers| identifiers.iter().cloned().collect_in(allocator)),
        is_handler_key: value.is_handler_key,
        is_ref_transformed: value.is_ref_transformed,
    }
}

fn compound_expression<'a>(
    value: &SnapshotCompoundExpression,
    allocator: &'a Bump,
) -> CompoundExpressionNode<'a> {
    CompoundExpressionNode {
        children: value
            .children
            .iter()
            .map(|child| compound_child(child, allocator))
            .collect_in(allocator),
        loc: value.location.clone(),
        identifiers: value
            .identifiers
            .as_ref()
            .map(|identifiers| identifiers.iter().cloned().collect_in(allocator)),
        is_handler_key: value.is_handler_key,
    }
}

fn compound_child<'a>(
    value: &SnapshotCompoundChild,
    allocator: &'a Bump,
) -> CompoundExpressionChild<'a> {
    match value {
        SnapshotCompoundChild::Simple(value) => CompoundExpressionChild::Simple(AllocBox::new_in(
            simple_expression(value, allocator),
            allocator,
        )),
        SnapshotCompoundChild::Compound(value) => CompoundExpressionChild::Compound(
            AllocBox::new_in(compound_expression(value, allocator), allocator),
        ),
        SnapshotCompoundChild::Interpolation(value) => CompoundExpressionChild::Interpolation(
            AllocBox::new_in(interpolation(value, allocator), allocator),
        ),
        SnapshotCompoundChild::Text(value) => {
            CompoundExpressionChild::Text(AllocBox::new_in(text(value), allocator))
        }
        SnapshotCompoundChild::String(value) => CompoundExpressionChild::String(value.clone()),
        SnapshotCompoundChild::Symbol(value) => CompoundExpressionChild::Symbol(*value),
    }
}

fn for_parse_result<'a>(value: &SnapshotForParseResult, allocator: &'a Bump) -> ForParseResult<'a> {
    ForParseResult {
        source: expression(&value.source, allocator),
        value: value
            .value
            .as_ref()
            .map(|value| expression(value, allocator)),
        key: value.key.as_ref().map(|value| expression(value, allocator)),
        index: value
            .index
            .as_ref()
            .map(|value| expression(value, allocator)),
        finalized: value.finalized,
    }
}

fn interpolation<'a>(value: &SnapshotInterpolation, allocator: &'a Bump) -> InterpolationNode<'a> {
    InterpolationNode {
        content: expression(&value.content, allocator),
        loc: value.location.clone(),
        #[cfg(feature = "_legacy")]
        raw: value.raw,
    }
}

fn text_call_content<'a>(
    value: &SnapshotTextCallContent,
    allocator: &'a Bump,
) -> TextCallContent<'a> {
    match value {
        SnapshotTextCallContent::Text(value) => {
            TextCallContent::Text(AllocBox::new_in(text(value), allocator))
        }
        SnapshotTextCallContent::Interpolation(value) => TextCallContent::Interpolation(
            AllocBox::new_in(interpolation(value, allocator), allocator),
        ),
        SnapshotTextCallContent::Compound(value) => TextCallContent::Compound(AllocBox::new_in(
            compound_expression(value, allocator),
            allocator,
        )),
    }
}

fn text(value: &SnapshotText) -> TextNode {
    TextNode {
        content: value.content.clone(),
        loc: value.location.clone(),
    }
}

fn comment(value: &SnapshotComment) -> CommentNode {
    CommentNode {
        content: value.content.clone(),
        loc: value.location.clone(),
        kind: value.kind,
        directive: value.directive,
    }
}
