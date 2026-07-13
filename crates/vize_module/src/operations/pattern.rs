use oxc_ast::ast::{AssignmentTarget, BindingPattern, FormalParameters, VariableDeclarationKind};
use oxc_span::GetSpan;

use crate::{ModuleBindingKind, ModuleObjectBinding, ModulePattern};

use super::{absolute, expression::expression_snapshot, expression::static_path, slice};

pub(super) fn binding_pattern(
    pattern: &BindingPattern<'_>,
    source: &str,
    base: u32,
) -> ModulePattern {
    match pattern {
        BindingPattern::BindingIdentifier(identifier) => {
            ModulePattern::Identifier(identifier.name.as_str().into())
        }
        BindingPattern::ObjectPattern(object) => {
            let mut properties = object
                .properties
                .iter()
                .map(|property| ModuleObjectBinding {
                    key: slice(source, property.key.span()).into(),
                    value: binding_pattern(&property.value, source, base),
                })
                .collect::<Vec<_>>();
            if let Some(rest) = &object.rest {
                properties.push(ModuleObjectBinding {
                    key: "...".into(),
                    value: ModulePattern::Rest(Box::new(binding_pattern(
                        &rest.argument,
                        source,
                        base,
                    ))),
                });
            }
            ModulePattern::Object(properties)
        }
        BindingPattern::ArrayPattern(array) => ModulePattern::Array(
            array
                .elements
                .iter()
                .map(|item| {
                    item.as_ref()
                        .map(|item| binding_pattern(item, source, base))
                })
                .chain(array.rest.iter().map(|rest| {
                    Some(ModulePattern::Rest(Box::new(binding_pattern(
                        &rest.argument,
                        source,
                        base,
                    ))))
                }))
                .collect(),
        ),
        BindingPattern::AssignmentPattern(assignment) => ModulePattern::Assignment {
            binding: Box::new(binding_pattern(&assignment.left, source, base)),
            default: Box::new(expression_snapshot(&assignment.right, source, base)),
        },
    }
}

pub(super) fn assignment_pattern(
    target: &AssignmentTarget<'_>,
    source: &str,
    base: u32,
) -> ModulePattern {
    match target {
        AssignmentTarget::AssignmentTargetIdentifier(identifier) => {
            ModulePattern::Identifier(identifier.name.as_str().into())
        }
        AssignmentTarget::StaticMemberExpression(member) => static_path(&member.object)
            .map(|mut path| {
                path.push(member.property.name.as_str().into());
                ModulePattern::Path(path)
            })
            .unwrap_or_else(|| unknown_pattern(source, target.span(), base)),
        _ => unknown_pattern(source, target.span(), base),
    }
}

pub(super) fn formal_parameters(
    parameters: &FormalParameters<'_>,
    source: &str,
    base: u32,
) -> Vec<ModulePattern> {
    parameters
        .items
        .iter()
        .map(|parameter| binding_pattern(&parameter.pattern, source, base))
        .chain(parameters.rest.iter().map(|rest| {
            ModulePattern::Rest(Box::new(binding_pattern(&rest.rest.argument, source, base)))
        }))
        .collect()
}

pub(super) fn binding_kind(kind: VariableDeclarationKind) -> ModuleBindingKind {
    match kind {
        VariableDeclarationKind::Const => ModuleBindingKind::Const,
        VariableDeclarationKind::Let => ModuleBindingKind::Let,
        VariableDeclarationKind::Var => ModuleBindingKind::Var,
        _ => ModuleBindingKind::Other,
    }
}

fn unknown_pattern(source: &str, span: oxc_span::Span, base: u32) -> ModulePattern {
    ModulePattern::Unknown {
        text: slice(source, span).into(),
        span: absolute(span, base),
    }
}
