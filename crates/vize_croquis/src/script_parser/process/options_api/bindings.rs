//! Local object binding inventory for Options API metadata.

use oxc_ast::ast::{BindingPattern, ObjectExpression, Program, Statement};
use vize_carton::FxHashMap;

use super::object_expression_from_expression;

pub(super) fn collect_object_bindings<'a>(
    program: &'a Program<'a>,
    object_bindings: &mut FxHashMap<&'a str, &'a ObjectExpression<'a>>,
) {
    for statement in program.body.iter() {
        let Statement::VariableDeclaration(declaration) = statement else {
            continue;
        };

        for declarator in declaration.declarations.iter() {
            let BindingPattern::BindingIdentifier(id) = &declarator.id else {
                continue;
            };
            let Some(init) = declarator.init.as_ref() else {
                continue;
            };
            let Some(object) = object_expression_from_expression(init) else {
                continue;
            };
            object_bindings.insert(id.name.as_str(), object);
        }
    }
}
