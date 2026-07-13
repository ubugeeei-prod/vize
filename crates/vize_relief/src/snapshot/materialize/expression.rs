use vize_carton::{Box as AllocBox, Bump};

use crate::{
    CompoundExpressionChild, CompoundExpressionNode, ExpressionNode, ForParseResult,
    InterpolationNode, JsExpression, SimpleExpressionNode,
};

use super::{CollectIn, text};
use crate::snapshot::{
    SnapshotCompoundChild, SnapshotCompoundExpression, SnapshotExpression, SnapshotForParseResult,
    SnapshotInterpolation, SnapshotSimpleExpression,
};

pub(super) fn expression<'a>(
    value: &SnapshotExpression,
    allocator: &'a Bump,
) -> ExpressionNode<'a> {
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

pub(super) fn simple_expression<'a>(
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

pub(super) fn compound_expression<'a>(
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

pub(super) fn for_parse_result<'a>(
    value: &SnapshotForParseResult,
    allocator: &'a Bump,
) -> ForParseResult<'a> {
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

pub(super) fn interpolation<'a>(
    value: &SnapshotInterpolation,
    allocator: &'a Bump,
) -> InterpolationNode<'a> {
    InterpolationNode {
        content: expression(&value.content, allocator),
        loc: value.location.clone(),
        #[cfg(feature = "_legacy")]
        raw: value.raw,
    }
}
