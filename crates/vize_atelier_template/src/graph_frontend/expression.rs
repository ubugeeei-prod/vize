use vize_carton::String;
use vize_relief::{
    SnapshotCompoundChild, SnapshotCompoundExpression, SnapshotExpression, SnapshotSimpleExpression,
};
use vize_rendu::{
    RenduBuilder, RenduExpression, RenduExpressionId, RenduExpressionKind, RenduName, RenduSourceId,
};

use super::provenance::rendu_provenance;

pub(super) fn add_rendu_expression(
    builder: &mut RenduBuilder,
    expression: &SnapshotExpression,
    source: RenduSourceId,
) -> RenduExpressionId {
    let kind = match expression {
        SnapshotExpression::Simple(simple) => simple_kind(simple),
        SnapshotExpression::Compound(_) => RenduExpressionKind::Compound,
    };
    builder.add_expression(
        RenduExpression::new(expression_code(expression).as_str(), kind)
            .with_provenance(rendu_provenance(expression.location(), source)),
    )
}

pub(super) fn add_rendu_compound(
    builder: &mut RenduBuilder,
    expression: &SnapshotCompoundExpression,
    source: RenduSourceId,
) -> RenduExpressionId {
    builder.add_expression(
        RenduExpression::new(
            compound_code(expression).as_str(),
            RenduExpressionKind::Compound,
        )
        .with_provenance(rendu_provenance(&expression.location, source)),
    )
}

pub(super) fn add_rendu_name(
    builder: &mut RenduBuilder,
    expression: &SnapshotExpression,
    source: RenduSourceId,
) -> RenduName {
    match expression {
        SnapshotExpression::Simple(simple) if simple.is_static => {
            RenduName::static_name(simple.content.as_str())
        }
        _ => RenduName::Dynamic(add_rendu_expression(builder, expression, source)),
    }
}

pub(super) fn expression_code(expression: &SnapshotExpression) -> String {
    match expression {
        SnapshotExpression::Simple(simple) => simple.content.clone(),
        SnapshotExpression::Compound(compound) => compound_code(compound),
    }
}

pub(super) fn compound_code(compound: &SnapshotCompoundExpression) -> String {
    let mut code = String::new("");
    for child in &compound.children {
        match child {
            SnapshotCompoundChild::Simple(simple) => code.push_str(&simple.content),
            SnapshotCompoundChild::Compound(compound) => code.push_str(&compound_code(compound)),
            SnapshotCompoundChild::Interpolation(interpolation) => {
                code.push_str(&expression_code(&interpolation.content));
            }
            SnapshotCompoundChild::Text(text) => code.push_str(&text.content),
            SnapshotCompoundChild::String(value) => code.push_str(value),
            SnapshotCompoundChild::Symbol(symbol) => code.push_str(symbol.name()),
        }
    }
    code
}

fn simple_kind(expression: &SnapshotSimpleExpression) -> RenduExpressionKind {
    if expression.is_static {
        RenduExpressionKind::Literal
    } else if is_identifier(&expression.content) {
        RenduExpressionKind::Reference
    } else {
        RenduExpressionKind::Opaque
    }
}

fn is_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first == '_' || first == '$' || first.is_ascii_alphabetic())
        && chars.all(|character| {
            character == '_' || character == '$' || character.is_ascii_alphanumeric()
        })
}
