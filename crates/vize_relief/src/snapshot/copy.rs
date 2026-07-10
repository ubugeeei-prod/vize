use crate::{
    AttributeNode, CommentNode, CompoundExpressionChild, CompoundExpressionNode, DirectiveNode,
    ExpressionNode, ForParseResult, InterpolationNode, PropNode, SimpleExpressionNode,
    TextCallContent, TextNode,
};

use super::{
    SnapshotAttribute, SnapshotComment, SnapshotCompoundChild, SnapshotCompoundExpression,
    SnapshotDirective, SnapshotExpression, SnapshotForParseResult, SnapshotInterpolation,
    SnapshotProp, SnapshotSimpleExpression, SnapshotText, SnapshotTextCallContent,
};

pub(super) fn expression(node: &ExpressionNode<'_>) -> SnapshotExpression {
    match node {
        ExpressionNode::Simple(node) => SnapshotExpression::Simple(simple_expression(node)),
        ExpressionNode::Compound(node) => SnapshotExpression::Compound(compound_expression(node)),
    }
}

pub(super) fn simple_expression(node: &SimpleExpressionNode<'_>) -> SnapshotSimpleExpression {
    SnapshotSimpleExpression {
        content: node.content.clone(),
        is_static: node.is_static,
        const_type: node.const_type,
        location: node.loc.clone(),
        js_raw: node.js_ast.as_ref().map(|ast| ast.raw.clone()),
        identifiers: node
            .identifiers
            .as_ref()
            .map(|identifiers| identifiers.iter().cloned().collect()),
        is_handler_key: node.is_handler_key,
        is_ref_transformed: node.is_ref_transformed,
    }
}

pub(super) fn compound_expression(node: &CompoundExpressionNode<'_>) -> SnapshotCompoundExpression {
    SnapshotCompoundExpression {
        children: node.children.iter().map(compound_child).collect(),
        location: node.loc.clone(),
        identifiers: node
            .identifiers
            .as_ref()
            .map(|identifiers| identifiers.iter().cloned().collect()),
        is_handler_key: node.is_handler_key,
    }
}

fn compound_child(node: &CompoundExpressionChild<'_>) -> SnapshotCompoundChild {
    match node {
        CompoundExpressionChild::Simple(node) => {
            SnapshotCompoundChild::Simple(simple_expression(node))
        }
        CompoundExpressionChild::Compound(node) => {
            SnapshotCompoundChild::Compound(compound_expression(node))
        }
        CompoundExpressionChild::Interpolation(node) => {
            SnapshotCompoundChild::Interpolation(interpolation(node))
        }
        CompoundExpressionChild::Text(node) => SnapshotCompoundChild::Text(text(node)),
        CompoundExpressionChild::String(value) => SnapshotCompoundChild::String(value.clone()),
        CompoundExpressionChild::Symbol(symbol) => SnapshotCompoundChild::Symbol(*symbol),
    }
}

pub(super) fn text(node: &TextNode) -> SnapshotText {
    SnapshotText {
        content: node.content.clone(),
        location: node.loc.clone(),
    }
}

pub(super) fn comment(node: &CommentNode) -> SnapshotComment {
    SnapshotComment {
        content: node.content.clone(),
        location: node.loc.clone(),
        kind: node.kind,
        directive: node.directive,
    }
}

pub(super) fn interpolation(node: &InterpolationNode<'_>) -> SnapshotInterpolation {
    SnapshotInterpolation {
        content: expression(&node.content),
        location: node.loc.clone(),
        #[cfg(feature = "_legacy")]
        raw: node.raw,
    }
}

pub(super) fn property(node: &PropNode<'_>) -> SnapshotProp {
    match node {
        PropNode::Attribute(node) => SnapshotProp::Attribute(attribute(node)),
        PropNode::Directive(node) => SnapshotProp::Directive(Box::new(directive(node))),
    }
}

fn attribute(node: &AttributeNode) -> SnapshotAttribute {
    SnapshotAttribute {
        name: node.name.clone(),
        name_location: node.name_loc.clone(),
        value: node.value.as_ref().map(text),
        location: node.loc.clone(),
    }
}

fn directive(node: &DirectiveNode<'_>) -> SnapshotDirective {
    SnapshotDirective {
        name: node.name.clone(),
        raw_name: node.raw_name.clone(),
        expression: node.exp.as_ref().map(expression),
        argument: node.arg.as_ref().map(expression),
        modifiers: node.modifiers.iter().map(simple_expression).collect(),
        for_parse_result: node.for_parse_result.as_ref().map(for_parse_result),
        shorthand: node.shorthand,
        location: node.loc.clone(),
    }
}

pub(super) fn for_parse_result(node: &ForParseResult<'_>) -> SnapshotForParseResult {
    SnapshotForParseResult {
        source: expression(&node.source),
        value: node.value.as_ref().map(expression),
        key: node.key.as_ref().map(expression),
        index: node.index.as_ref().map(expression),
        finalized: node.finalized,
    }
}

pub(super) fn text_call_content(node: &TextCallContent<'_>) -> SnapshotTextCallContent {
    match node {
        TextCallContent::Text(node) => SnapshotTextCallContent::Text(text(node)),
        TextCallContent::Interpolation(node) => {
            SnapshotTextCallContent::Interpolation(interpolation(node))
        }
        TextCallContent::Compound(node) => {
            SnapshotTextCallContent::Compound(compound_expression(node))
        }
    }
}
