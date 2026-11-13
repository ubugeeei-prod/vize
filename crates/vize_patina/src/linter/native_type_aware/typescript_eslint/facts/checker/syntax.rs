//! Syntax helpers for the listener queries: super calls, call arity and arrow bodies.
use crate::linter::corsa_session::ExpressionFact;
use corsa::lint::LintNode;
use serde_json::{Value, json};

pub(super) fn range_of(value: &Value) -> Option<(u32, u32)> {
    let start = u32::try_from(value.get("start")?.as_u64()?).ok()?;
    let end = u32::try_from(value.get("end")?.as_u64()?).ok()?;
    Some((start, end))
}

/// The extended class expression of the class whose constructor makes the
/// `super(...)` call. The checker answers `any` for the `super` keyword itself,
/// while typescript-eslint sees the base constructor type.
pub(super) fn super_call_base(node: &LintNode) -> Option<(u32, u32)> {
    if node.kind != "CallExpression" || node.child("callee")?.kind != "Super" {
        return None;
    }
    let ancestors = node.fields.get("__ancestorFacts")?.as_array()?;
    let class = ancestors.iter().rev().find(|ancestor| {
        matches!(
            ancestor.get("kind").and_then(Value::as_str),
            Some("ClassDeclaration" | "ClassExpression")
        )
    })?;
    range_of(class.get("superClass")?)
}

pub(super) fn type_super_callee(node: &mut LintNode, base: &ExpressionFact) {
    let Some(callee) = node.children.get_mut("callee") else {
        return;
    };
    callee
        .fields
        .insert("__typedFromSuperClass".into(), json!(true));
    callee.type_texts = base
        .type_texts
        .iter()
        .map(|text| text.as_str().into())
        .collect();
    callee.property_names = base
        .property_names
        .iter()
        .map(|name| name.as_str().into())
        .collect();
    match &base.ty {
        Some(ty) => {
            callee.fields.insert("__type".into(), ty.clone());
        }
        None => {
            callee.fields.remove("__type");
        }
    }
}

pub(super) fn call_arguments(node: &LintNode) -> u32 {
    let count = if node.kind == "TaggedTemplateExpression" {
        node.child("quasi")
            .and_then(|quasi| quasi.child_list("expressions"))
            .map_or(0, |expressions| expressions.len() + 1)
    } else {
        node.child_list("arguments").map_or(0, <[LintNode]>::len)
    };
    u32::try_from(count).unwrap_or(u32::MAX)
}

pub(super) fn has_expression_body(node: &LintNode) -> bool {
    node.child("body")
        .is_some_and(|body| body.kind != "BlockStatement")
}
