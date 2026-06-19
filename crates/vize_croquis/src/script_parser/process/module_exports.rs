use oxc_ast::ast::{BindingPattern, Declaration};
use vize_carton::CompactString;

use crate::script_parser::ScriptParseResult;

pub(super) fn record_module_value_exports(result: &mut ScriptParseResult, decl: &Declaration<'_>) {
    match decl {
        Declaration::VariableDeclaration(variable) => {
            for declarator in &variable.declarations {
                record_module_value_binding(result, &declarator.id);
            }
        }
        Declaration::FunctionDeclaration(func) => {
            if let Some(id) = &func.id {
                result
                    .module_value_bindings
                    .insert(CompactString::new(id.name.as_str()));
            }
        }
        Declaration::ClassDeclaration(class) => {
            if let Some(id) = &class.id {
                result
                    .module_value_bindings
                    .insert(CompactString::new(id.name.as_str()));
            }
        }
        _ => {}
    }
}

fn record_module_value_binding(result: &mut ScriptParseResult, pattern: &BindingPattern<'_>) {
    match pattern {
        BindingPattern::BindingIdentifier(id) => {
            result
                .module_value_bindings
                .insert(CompactString::new(id.name.as_str()));
        }
        BindingPattern::ObjectPattern(object) => {
            for property in &object.properties {
                record_module_value_binding(result, &property.value);
            }
            if let Some(rest) = &object.rest {
                record_module_value_binding(result, &rest.argument);
            }
        }
        BindingPattern::ArrayPattern(array) => {
            for element in array.elements.iter().flatten() {
                record_module_value_binding(result, element);
            }
            if let Some(rest) = &array.rest {
                record_module_value_binding(result, &rest.argument);
            }
        }
        BindingPattern::AssignmentPattern(assignment) => {
            record_module_value_binding(result, &assignment.left);
        }
    }
}
