//! Reactive dependency extraction for effect graph builders.

use oxc_ast::ast::{Argument, BindingPattern, Expression, FunctionBody, Statement};
use vize_carton::{CompactString, FxHashSet};

pub(super) fn collect_argument_deps(
    argument: &Argument<'_>,
    reactive_sources: &FxHashSet<CompactString>,
    excluded: &FxHashSet<CompactString>,
    deps: &mut std::collections::BTreeSet<CompactString>,
) {
    match argument {
        Argument::ArrowFunctionExpression(arrow) => {
            let mut inner_excluded = excluded.clone();
            collect_binding_pattern_names_from_params(&arrow.params, &mut inner_excluded);
            collect_body_deps(&arrow.body, reactive_sources, &inner_excluded, deps);
        }
        Argument::FunctionExpression(function) => {
            let mut inner_excluded = excluded.clone();
            collect_binding_pattern_names_from_params(&function.params, &mut inner_excluded);
            if let Some(body) = &function.body {
                collect_body_deps(body, reactive_sources, &inner_excluded, deps);
            }
        }
        _ => {
            if let Some(expression) = argument.as_expression() {
                collect_expression_deps(expression, reactive_sources, excluded, deps);
            }
        }
    }
}

fn collect_body_deps(
    body: &FunctionBody<'_>,
    reactive_sources: &FxHashSet<CompactString>,
    excluded: &FxHashSet<CompactString>,
    deps: &mut std::collections::BTreeSet<CompactString>,
) {
    for statement in body.statements.iter() {
        collect_statement_deps(statement, reactive_sources, &mut excluded.clone(), deps);
    }
}

fn collect_statement_deps(
    statement: &Statement<'_>,
    reactive_sources: &FxHashSet<CompactString>,
    excluded: &mut FxHashSet<CompactString>,
    deps: &mut std::collections::BTreeSet<CompactString>,
) {
    match statement {
        Statement::ExpressionStatement(statement) => {
            collect_expression_deps(&statement.expression, reactive_sources, excluded, deps);
        }
        Statement::ReturnStatement(statement) => {
            if let Some(argument) = &statement.argument {
                collect_expression_deps(argument, reactive_sources, excluded, deps);
            }
        }
        Statement::VariableDeclaration(declaration) => {
            for declarator in declaration.declarations.iter() {
                if let Some(init) = &declarator.init {
                    collect_expression_deps(init, reactive_sources, excluded, deps);
                }
                collect_binding_pattern_names(&declarator.id, excluded);
            }
        }
        Statement::BlockStatement(block) => {
            let mut local_excluded = excluded.clone();
            for statement in block.body.iter() {
                collect_statement_deps(statement, reactive_sources, &mut local_excluded, deps);
            }
        }
        Statement::IfStatement(statement) => {
            collect_expression_deps(&statement.test, reactive_sources, excluded, deps);
            collect_statement_deps(&statement.consequent, reactive_sources, excluded, deps);
            if let Some(alternate) = &statement.alternate {
                collect_statement_deps(alternate, reactive_sources, excluded, deps);
            }
        }
        _ => {}
    }
}

fn collect_expression_deps(
    expression: &Expression<'_>,
    reactive_sources: &FxHashSet<CompactString>,
    excluded: &FxHashSet<CompactString>,
    deps: &mut std::collections::BTreeSet<CompactString>,
) {
    match expression {
        Expression::Identifier(identifier) => {
            let name = CompactString::new(identifier.name.as_str());
            if reactive_sources.contains(&name) && !excluded.contains(&name) {
                deps.insert(name);
            }
        }
        Expression::StaticMemberExpression(member) => {
            collect_expression_deps(&member.object, reactive_sources, excluded, deps);
        }
        Expression::ComputedMemberExpression(member) => {
            collect_expression_deps(&member.object, reactive_sources, excluded, deps);
            collect_expression_deps(&member.expression, reactive_sources, excluded, deps);
        }
        Expression::ChainExpression(chain) => match &chain.expression {
            oxc_ast::ast::ChainElement::CallExpression(call) => {
                for argument in call.arguments.iter() {
                    collect_argument_deps(argument, reactive_sources, excluded, deps);
                }
            }
            oxc_ast::ast::ChainElement::StaticMemberExpression(member) => {
                collect_expression_deps(&member.object, reactive_sources, excluded, deps);
            }
            oxc_ast::ast::ChainElement::ComputedMemberExpression(member) => {
                collect_expression_deps(&member.object, reactive_sources, excluded, deps);
                collect_expression_deps(&member.expression, reactive_sources, excluded, deps);
            }
            oxc_ast::ast::ChainElement::TSNonNullExpression(expression) => {
                collect_expression_deps(&expression.expression, reactive_sources, excluded, deps);
            }
            oxc_ast::ast::ChainElement::PrivateFieldExpression(expression) => {
                collect_expression_deps(&expression.object, reactive_sources, excluded, deps);
            }
        },
        Expression::CallExpression(call) => {
            for argument in call.arguments.iter() {
                collect_argument_deps(argument, reactive_sources, excluded, deps);
            }
        }
        Expression::ArrowFunctionExpression(arrow) => {
            let mut inner_excluded = excluded.clone();
            collect_binding_pattern_names_from_params(&arrow.params, &mut inner_excluded);
            collect_body_deps(&arrow.body, reactive_sources, &inner_excluded, deps);
        }
        Expression::FunctionExpression(function) => {
            let mut inner_excluded = excluded.clone();
            collect_binding_pattern_names_from_params(&function.params, &mut inner_excluded);
            if let Some(body) = &function.body {
                collect_body_deps(body, reactive_sources, &inner_excluded, deps);
            }
        }
        Expression::LogicalExpression(expression) => {
            collect_expression_deps(&expression.left, reactive_sources, excluded, deps);
            collect_expression_deps(&expression.right, reactive_sources, excluded, deps);
        }
        Expression::BinaryExpression(expression) => {
            collect_expression_deps(&expression.left, reactive_sources, excluded, deps);
            collect_expression_deps(&expression.right, reactive_sources, excluded, deps);
        }
        Expression::ConditionalExpression(expression) => {
            collect_expression_deps(&expression.test, reactive_sources, excluded, deps);
            collect_expression_deps(&expression.consequent, reactive_sources, excluded, deps);
            collect_expression_deps(&expression.alternate, reactive_sources, excluded, deps);
        }
        Expression::ArrayExpression(array) => {
            for element in array.elements.iter() {
                if let Some(expression) = element.as_expression() {
                    collect_expression_deps(expression, reactive_sources, excluded, deps);
                }
            }
        }
        Expression::ObjectExpression(object) => {
            for property in object.properties.iter() {
                match property {
                    oxc_ast::ast::ObjectPropertyKind::ObjectProperty(property) => {
                        if let Some(key) = property.key.as_expression() {
                            collect_expression_deps(key, reactive_sources, excluded, deps);
                        }
                        collect_expression_deps(&property.value, reactive_sources, excluded, deps);
                    }
                    oxc_ast::ast::ObjectPropertyKind::SpreadProperty(spread) => {
                        collect_expression_deps(&spread.argument, reactive_sources, excluded, deps);
                    }
                }
            }
        }
        Expression::AssignmentExpression(expression) => {
            collect_expression_deps(&expression.right, reactive_sources, excluded, deps);
        }
        Expression::UpdateExpression(expression) => {
            collect_simple_assignment_target_deps(
                &expression.argument,
                reactive_sources,
                excluded,
                deps,
            );
        }
        Expression::AwaitExpression(expression) => {
            collect_expression_deps(&expression.argument, reactive_sources, excluded, deps);
        }
        Expression::UnaryExpression(expression) => {
            collect_expression_deps(&expression.argument, reactive_sources, excluded, deps);
        }
        Expression::SequenceExpression(expression) => {
            for expression in expression.expressions.iter() {
                collect_expression_deps(expression, reactive_sources, excluded, deps);
            }
        }
        Expression::ParenthesizedExpression(expression) => {
            collect_expression_deps(&expression.expression, reactive_sources, excluded, deps);
        }
        Expression::TSAsExpression(expression) => {
            collect_expression_deps(&expression.expression, reactive_sources, excluded, deps);
        }
        Expression::TSSatisfiesExpression(expression) => {
            collect_expression_deps(&expression.expression, reactive_sources, excluded, deps);
        }
        Expression::TSNonNullExpression(expression) => {
            collect_expression_deps(&expression.expression, reactive_sources, excluded, deps);
        }
        _ => {}
    }
}

fn collect_simple_assignment_target_deps(
    target: &oxc_ast::ast::SimpleAssignmentTarget<'_>,
    reactive_sources: &FxHashSet<CompactString>,
    excluded: &FxHashSet<CompactString>,
    deps: &mut std::collections::BTreeSet<CompactString>,
) {
    match target {
        oxc_ast::ast::SimpleAssignmentTarget::AssignmentTargetIdentifier(identifier) => {
            let name = CompactString::new(identifier.name.as_str());
            if reactive_sources.contains(&name) && !excluded.contains(&name) {
                deps.insert(name);
            }
        }
        oxc_ast::ast::SimpleAssignmentTarget::StaticMemberExpression(member) => {
            collect_expression_deps(&member.object, reactive_sources, excluded, deps);
        }
        oxc_ast::ast::SimpleAssignmentTarget::ComputedMemberExpression(member) => {
            collect_expression_deps(&member.object, reactive_sources, excluded, deps);
            collect_expression_deps(&member.expression, reactive_sources, excluded, deps);
        }
        oxc_ast::ast::SimpleAssignmentTarget::TSAsExpression(expression) => {
            collect_expression_deps(&expression.expression, reactive_sources, excluded, deps);
        }
        oxc_ast::ast::SimpleAssignmentTarget::TSSatisfiesExpression(expression) => {
            collect_expression_deps(&expression.expression, reactive_sources, excluded, deps);
        }
        oxc_ast::ast::SimpleAssignmentTarget::TSNonNullExpression(expression) => {
            collect_expression_deps(&expression.expression, reactive_sources, excluded, deps);
        }
        _ => {}
    }
}

fn collect_binding_pattern_names_from_params(
    params: &oxc_ast::ast::FormalParameters<'_>,
    names: &mut FxHashSet<CompactString>,
) {
    for param in params.items.iter() {
        collect_binding_pattern_names(&param.pattern, names);
    }
    if let Some(rest) = &params.rest {
        collect_binding_pattern_names(&rest.rest.argument, names);
    }
}

fn collect_binding_pattern_names(
    pattern: &BindingPattern<'_>,
    names: &mut FxHashSet<CompactString>,
) {
    match pattern {
        BindingPattern::BindingIdentifier(identifier) => {
            names.insert(CompactString::new(identifier.name.as_str()));
        }
        BindingPattern::ObjectPattern(object) => {
            for property in object.properties.iter() {
                collect_binding_pattern_names(&property.value, names);
            }
            if let Some(rest) = &object.rest {
                collect_binding_pattern_names(&rest.argument, names);
            }
        }
        BindingPattern::ArrayPattern(array) => {
            for element in array.elements.iter().flatten() {
                collect_binding_pattern_names(element, names);
            }
            if let Some(rest) = &array.rest {
                collect_binding_pattern_names(&rest.argument, names);
            }
        }
        BindingPattern::AssignmentPattern(assign) => {
            collect_binding_pattern_names(&assign.left, names);
        }
    }
}
