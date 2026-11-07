//! Component bindings visible above every template-local scope.

use crate::context::LintContext;
use oxc_ast::ast::{
    ArrayExpressionElement, BindingPattern, ExportDefaultDeclarationKind, Expression,
    ImportDeclarationSpecifier, ObjectExpression, ObjectPropertyKind, Program, PropertyKey,
    Statement,
};
use vize_l0::{CompactString, FxHashSet};

pub(super) fn collect(ctx: &LintContext<'_>) -> FxHashSet<CompactString> {
    let mut names = FxHashSet::default();
    if let Some(analysis) = ctx.analysis() {
        names.extend(analysis.bindings.bindings.keys().cloned());
        names.extend(analysis.macros.props().iter().map(|prop| prop.name.clone()));
    }
    let Some(scripts) = ctx.sfc_scripts() else {
        return names;
    };
    if let Some(parsed) = scripts.script_setup() {
        collect_setup(&parsed.program, &mut names);
    }
    if let Some(parsed) = scripts.script() {
        collect_options(&parsed.program, &mut names);
    }
    names
}

fn collect_setup(program: &Program<'_>, names: &mut FxHashSet<CompactString>) {
    for statement in &program.body {
        match statement {
            Statement::VariableDeclaration(declaration) if !declaration.declare => {
                for declarator in &declaration.declarations {
                    collect_pattern(&declarator.id, names);
                }
            }
            Statement::FunctionDeclaration(function) if !function.declare => {
                if let Some(identifier) = &function.id {
                    names.insert(CompactString::new(identifier.name.as_str()));
                }
            }
            Statement::ClassDeclaration(class) if !class.declare => {
                if let Some(identifier) = &class.id {
                    names.insert(CompactString::new(identifier.name.as_str()));
                }
            }
            Statement::ImportDeclaration(import) if !import.import_kind.is_type() => {
                for specifier in import.specifiers.iter().flatten() {
                    let local = match specifier {
                        ImportDeclarationSpecifier::ImportSpecifier(specifier)
                            if !specifier.import_kind.is_type() =>
                        {
                            &specifier.local
                        }
                        ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => {
                            &specifier.local
                        }
                        ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => {
                            &specifier.local
                        }
                        _ => continue,
                    };
                    names.insert(CompactString::new(local.name.as_str()));
                }
            }
            _ => {}
        }
    }
}

fn collect_pattern(pattern: &BindingPattern<'_>, names: &mut FxHashSet<CompactString>) {
    match pattern {
        BindingPattern::BindingIdentifier(identifier) => {
            names.insert(CompactString::new(identifier.name.as_str()));
        }
        BindingPattern::ObjectPattern(object) => {
            for property in &object.properties {
                collect_pattern(&property.value, names);
            }
            if let Some(rest) = &object.rest {
                collect_pattern(&rest.argument, names);
            }
        }
        BindingPattern::ArrayPattern(array) => {
            for element in array.elements.iter().flatten() {
                collect_pattern(element, names);
            }
            if let Some(rest) = &array.rest {
                collect_pattern(&rest.argument, names);
            }
        }
        BindingPattern::AssignmentPattern(assignment) => collect_pattern(&assignment.left, names),
    }
}

fn collect_options(program: &Program<'_>, names: &mut FxHashSet<CompactString>) {
    for statement in &program.body {
        let Statement::ExportDefaultDeclaration(export) = statement else {
            continue;
        };
        let options = match &export.declaration {
            ExportDefaultDeclarationKind::ObjectExpression(object) => Some(object.as_ref()),
            ExportDefaultDeclarationKind::CallExpression(call) => call
                .arguments
                .first()
                .and_then(|argument| argument.as_expression())
                .and_then(object_expression),
            _ => None,
        };
        if let Some(options) = options {
            collect_options_object(options, names);
        }
    }
}

fn collect_options_object(object: &ObjectExpression<'_>, names: &mut FxHashSet<CompactString>) {
    for property in &object.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            continue;
        };
        if property.computed {
            continue;
        }
        match property_key_name(&property.key) {
            Some("props" | "inject") => collect_array_or_object(&property.value, names),
            Some("computed" | "methods") => collect_object_keys(&property.value, names),
            Some("data" | "asyncData" | "setup") => {
                if let Some(returned) = returned_object(&property.value) {
                    collect_object_keys_from_object(returned, names);
                }
            }
            _ => {}
        }
    }
}

fn collect_array_or_object(expression: &Expression<'_>, names: &mut FxHashSet<CompactString>) {
    match expression.without_parentheses() {
        Expression::ArrayExpression(array) => {
            for element in &array.elements {
                if let ArrayExpressionElement::StringLiteral(literal) = element {
                    names.insert(CompactString::new(literal.value.as_str()));
                }
            }
        }
        Expression::ObjectExpression(object) => collect_object_keys_from_object(object, names),
        _ => {}
    }
}

fn collect_object_keys(expression: &Expression<'_>, names: &mut FxHashSet<CompactString>) {
    if let Some(object) = object_expression(expression) {
        collect_object_keys_from_object(object, names);
    }
}

fn collect_object_keys_from_object(
    object: &ObjectExpression<'_>,
    names: &mut FxHashSet<CompactString>,
) {
    for property in &object.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            continue;
        };
        if !property.computed
            && let Some(name) = property_key_name(&property.key)
        {
            names.insert(CompactString::new(name));
        }
    }
}

fn returned_object<'a>(expression: &'a Expression<'a>) -> Option<&'a ObjectExpression<'a>> {
    match expression.without_parentheses() {
        Expression::FunctionExpression(function) => function
            .body
            .as_ref()?
            .statements
            .iter()
            .find_map(returned_object_statement),
        Expression::ArrowFunctionExpression(arrow) if arrow.expression => {
            let Statement::ExpressionStatement(statement) = arrow.body.statements.first()? else {
                return None;
            };
            object_expression(&statement.expression)
        }
        Expression::ArrowFunctionExpression(arrow) => arrow
            .body
            .statements
            .iter()
            .find_map(returned_object_statement),
        _ => None,
    }
}

fn returned_object_statement<'a>(statement: &'a Statement<'a>) -> Option<&'a ObjectExpression<'a>> {
    let Statement::ReturnStatement(statement) = statement else {
        return None;
    };
    object_expression(statement.argument.as_ref()?)
}

fn object_expression<'a>(expression: &'a Expression<'a>) -> Option<&'a ObjectExpression<'a>> {
    match expression.without_parentheses() {
        Expression::ObjectExpression(object) => Some(object),
        _ => None,
    }
}

fn property_key_name<'a>(key: &'a PropertyKey<'a>) -> Option<&'a str> {
    match key {
        PropertyKey::StaticIdentifier(identifier) => Some(identifier.name.as_str()),
        PropertyKey::StringLiteral(literal) => Some(literal.value.as_str()),
        _ => None,
    }
}
