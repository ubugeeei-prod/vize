use vize_relief::{Namespace, SnapshotDirective, SnapshotExpression, SnapshotProp};
use vize_rendu::{RenduBinding, RenduNamespace, RenduProvenance, RenduSourceId};

use super::{expression::expression_code, provenance::rendu_provenance};

pub(super) fn namespace(namespace: Namespace) -> RenduNamespace {
    match namespace {
        Namespace::Html => RenduNamespace::Html,
        Namespace::Svg => RenduNamespace::Svg,
        Namespace::MathMl => RenduNamespace::MathMl,
    }
}

pub(super) fn slot_directive(properties: &[SnapshotProp]) -> Option<(usize, &SnapshotDirective)> {
    properties.iter().enumerate().find_map(|(index, property)| {
        let SnapshotProp::Directive(directive) = property else {
            return None;
        };
        (directive.name == "slot").then_some((index, directive.as_ref()))
    })
}

pub(super) fn is_name_argument(expression: &SnapshotExpression) -> bool {
    matches!(
        expression,
        SnapshotExpression::Simple(simple) if simple.is_static && simple.content == "name"
    )
}

pub(super) fn binding(
    expression: Option<&SnapshotExpression>,
    fallback: &'static str,
    provenance: &RenduProvenance,
) -> RenduBinding {
    let code = expression
        .map(expression_code)
        .filter(|code| !code.is_empty());
    let pattern: Box<str> = code
        .as_ref()
        .map_or_else(|| fallback.into(), |code| code.as_str().into());
    RenduBinding::new(pattern).with_provenance(provenance.clone())
}

pub(super) fn optional_binding(
    expression: Option<&SnapshotExpression>,
    provenance: &RenduProvenance,
) -> Option<RenduBinding> {
    let code = expression.map(expression_code)?;
    (!code.is_empty()).then(|| RenduBinding::new(code.as_str()).with_provenance(provenance.clone()))
}

pub(super) fn slot_bindings(
    directive: &SnapshotDirective,
    source: RenduSourceId,
) -> Vec<RenduBinding> {
    directive
        .expression
        .as_ref()
        .map(|expression| {
            RenduBinding::new(expression_code(expression).as_str())
                .with_provenance(rendu_provenance(expression.location(), source))
        })
        .into_iter()
        .collect()
}
