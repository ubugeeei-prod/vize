//! Event names a call of the emit function can dispatch.
//!
//! A literal names its event. A name Patina can still prove is resolved too:
//! a static template literal, a conditional between such names, or a
//! top-level `const` (or a static key of a `const` object, `as const` or not)
//! that holds one. Any other argument computes the name, so the rule cannot
//! prove a declared event unused and must stay silent.

use oxc_ast::ast::{
    Argument, Declaration, Expression, ObjectPropertyKind, Program, Statement,
    VariableDeclarationKind,
};
use vize_l0::{CompactString, FxHashMap, cstr};

use super::keys::property_key_name;

/// Top-level `const` names (and `Name.key` object members) bound to event
/// names, in declaration order so a const may name an earlier one.
#[derive(Default)]
pub(super) struct ConstEventNames {
    names: FxHashMap<CompactString, Vec<CompactString>>,
}

impl ConstEventNames {
    pub(super) fn of(program: &Program<'_>) -> Self {
        let mut consts = Self::default();
        for statement in &program.body {
            let declaration = match statement {
                Statement::VariableDeclaration(declaration) => declaration,
                Statement::ExportNamedDeclaration(export) => match &export.declaration {
                    Some(Declaration::VariableDeclaration(declaration)) => declaration,
                    _ => continue,
                },
                _ => continue,
            };
            if declaration.kind != VariableDeclarationKind::Const || declaration.declare {
                continue;
            }
            for declarator in &declaration.declarations {
                let (Some(name), Some(init)) = (
                    declarator.id.get_identifier_name(),
                    declarator.init.as_ref(),
                ) else {
                    continue;
                };
                if let Expression::ObjectExpression(object) = init.get_inner_expression() {
                    for property in &object.properties {
                        let ObjectPropertyKind::ObjectProperty(property) = property else {
                            continue;
                        };
                        if property.computed {
                            continue;
                        }
                        let (Some(key), Some(names)) = (
                            property_key_name(&property.key),
                            consts.resolve(&property.value),
                        ) else {
                            continue;
                        };
                        consts.names.insert(cstr!("{name}.{key}"), names);
                    }
                } else if let Some(names) = consts.resolve(init) {
                    consts
                        .names
                        .insert(CompactString::new(name.as_str()), names);
                }
            }
        }
        consts
    }

    /// The events `argument` names, or `None` when the name is computed.
    pub(super) fn argument(&self, argument: &Argument<'_>) -> Option<Vec<CompactString>> {
        match argument {
            Argument::SpreadElement(_) => None,
            _ => self.resolve(argument.as_expression()?),
        }
    }

    fn resolve(&self, expression: &Expression<'_>) -> Option<Vec<CompactString>> {
        match expression.get_inner_expression() {
            Expression::StringLiteral(literal) => {
                Some(vec![CompactString::new(literal.value.as_str())])
            }
            Expression::TemplateLiteral(literal) => literal
                .single_quasi()
                .map(|quasi| vec![CompactString::new(quasi.as_str())]),
            Expression::ConditionalExpression(conditional) => {
                let mut names = self.resolve(&conditional.consequent)?;
                names.extend(self.resolve(&conditional.alternate)?);
                Some(names)
            }
            Expression::Identifier(identifier) => self.names.get(identifier.name.as_str()).cloned(),
            Expression::StaticMemberExpression(member) => {
                let Expression::Identifier(object) = member.object.get_inner_expression() else {
                    return None;
                };
                self.names
                    .get(cstr!("{}.{}", object.name, member.property.name).as_str())
                    .cloned()
            }
            _ => None,
        }
    }
}
