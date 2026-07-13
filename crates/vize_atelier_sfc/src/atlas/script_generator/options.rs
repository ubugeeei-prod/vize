//! Owned Options API source facts and rewrite spans.

#[path = "options/component.rs"]
mod component;

use oxc_ast::ast::{
    ArrayExpressionElement, BindingPattern, ExportDefaultDeclarationKind, Expression,
    ObjectExpression, ObjectPropertyKind, Program, Statement, VariableDeclarationKind,
};
use oxc_span::GetSpan;
use vize_carton::{FxHashSet, String};

use super::{ScriptDefaultExportTargets, ScriptOptionsApiPropsSource};
pub(super) use component::{
    component_options_from_program, option_expression_property, option_object_property,
    property_key_name, source_slice,
};
use component::{
    is_safe_value_identifier, object_expression_from_expression,
    object_props_must_stay_in_value_scope,
};

pub(super) fn default_export_targets(program: &Program<'_>) -> ScriptDefaultExportTargets {
    let mut targets = ScriptDefaultExportTargets::default();
    for statement in &program.body {
        let Statement::ExportDefaultDeclaration(export) = statement else {
            continue;
        };
        match &export.declaration {
            ExportDefaultDeclarationKind::ObjectExpression(object) => {
                let span = object.span();
                targets.object = Some((
                    export.span.start as usize,
                    span.start as usize,
                    span.end as usize,
                ));
            }
            ExportDefaultDeclarationKind::ClassDeclaration(class) if class.id.is_some() => {
                let id = class.id.as_ref().expect("class id checked");
                targets.class = Some((
                    export.span.start as usize,
                    class.span.start as usize,
                    class.span.end as usize,
                    id.span.start as usize,
                    id.span.end as usize,
                ));
            }
            other => {
                let span = other.span();
                targets.expr = Some((
                    export.span.start as usize,
                    span.start as usize,
                    span.end as usize,
                ));
            }
        }
        break;
    }
    targets
}

pub(super) fn options_api_props(
    program: &Program<'_>,
    source: &str,
) -> Option<ScriptOptionsApiPropsSource> {
    let options = component_options_from_program(program)?;
    let props = option_expression_property(options, "props")?;
    props_source_from_expression(source, props)
}

fn props_source_from_expression(
    source: &str,
    expression: &Expression<'_>,
) -> Option<ScriptOptionsApiPropsSource> {
    match expression {
        Expression::ObjectExpression(object) => {
            let source = String::from(source_slice(source, object.span())?);
            Some(if object_props_must_stay_in_value_scope(object) {
                ScriptOptionsApiPropsSource::DeferredObject(source)
            } else {
                ScriptOptionsApiPropsSource::Object(source)
            })
        }
        Expression::ArrayExpression(array) => {
            let names = array
                .elements
                .iter()
                .filter_map(|element| {
                    let ArrayExpressionElement::StringLiteral(literal) = element else {
                        return None;
                    };
                    Some(String::from(literal.value.as_str()))
                })
                .collect::<Vec<_>>();
            (!names.is_empty()).then_some(ScriptOptionsApiPropsSource::Names(names))
        }
        Expression::Identifier(identifier)
            if is_safe_value_identifier(identifier.name.as_str()) =>
        {
            Some(ScriptOptionsApiPropsSource::DeferredObject(
                identifier.name.as_str().into(),
            ))
        }
        Expression::ParenthesizedExpression(value) => {
            props_source_from_expression(source, &value.expression)
        }
        Expression::TSAsExpression(value) => {
            props_source_from_expression(source, &value.expression)
        }
        Expression::TSSatisfiesExpression(value) => {
            props_source_from_expression(source, &value.expression)
        }
        Expression::TSNonNullExpression(value) => {
            props_source_from_expression(source, &value.expression)
        }
        _ => None,
    }
}

pub(super) fn has_unresolved_extends(program: &Program<'_>) -> bool {
    let Some(options) = component_options_from_program(program) else {
        return false;
    };
    let Some(extends) = option_expression_property(options, "extends") else {
        return false;
    };
    let object_bindings = object_expression_bindings(program);
    !is_resolved_options_target(extends, &object_bindings)
}

fn object_expression_bindings<'a>(program: &'a Program<'a>) -> FxHashSet<&'a str> {
    let mut bindings = FxHashSet::default();
    for statement in &program.body {
        let Statement::VariableDeclaration(declaration) = statement else {
            continue;
        };
        for declarator in &declaration.declarations {
            let BindingPattern::BindingIdentifier(id) = &declarator.id else {
                continue;
            };
            if declarator
                .init
                .as_ref()
                .is_some_and(|init| object_expression_from_expression(init).is_some())
            {
                bindings.insert(id.name.as_str());
            }
        }
    }
    bindings
}

fn is_resolved_options_target<'a>(
    expression: &'a Expression<'a>,
    bindings: &FxHashSet<&'a str>,
) -> bool {
    match expression {
        Expression::ObjectExpression(_) => true,
        Expression::Identifier(identifier) => bindings.contains(identifier.name.as_str()),
        Expression::ParenthesizedExpression(value) => {
            is_resolved_options_target(&value.expression, bindings)
        }
        Expression::TSAsExpression(value) => {
            is_resolved_options_target(&value.expression, bindings)
        }
        Expression::TSSatisfiesExpression(value) => {
            is_resolved_options_target(&value.expression, bindings)
        }
        Expression::TSNonNullExpression(value) => {
            is_resolved_options_target(&value.expression, bindings)
        }
        _ => false,
    }
}

pub(super) fn props_const_assertion_offsets(program: &Program<'_>) -> Vec<usize> {
    let Some(options) = component_options_from_program(program) else {
        return Vec::new();
    };
    let Some(props) = option_expression_property(options, "props") else {
        return Vec::new();
    };
    let mut prop_bindings = FxHashSet::default();
    collect_props_identifier_names(props, &mut prop_bindings);
    let mut offsets = Vec::new();
    for statement in &program.body {
        let Statement::VariableDeclaration(declaration) = statement else {
            continue;
        };
        if declaration.kind != VariableDeclarationKind::Const {
            continue;
        }
        for declarator in &declaration.declarations {
            let BindingPattern::BindingIdentifier(id) = &declarator.id else {
                continue;
            };
            if prop_bindings.contains(id.name.as_str())
                && let Some(init) = declarator.init.as_ref()
                && let Some(offset) = const_assertion_offset(init)
            {
                offsets.push(offset);
            }
        }
    }
    offsets.sort_unstable();
    offsets.dedup();
    offsets
}

pub(super) fn setup_return_has_spread(program: &Program<'_>) -> bool {
    let Some(options) = component_options_from_program(program) else {
        return false;
    };
    option_expression_property(options, "setup")
        .and_then(setup_return_object_from_expression)
        .is_some_and(|object| {
            object
                .properties
                .iter()
                .any(|property| matches!(property, ObjectPropertyKind::SpreadProperty(_)))
        })
}

fn setup_return_object_from_expression<'a>(
    expression: &'a Expression<'a>,
) -> Option<&'a ObjectExpression<'a>> {
    match expression {
        Expression::FunctionExpression(function) => {
            return_object_in_body(&function.body.as_ref()?.statements)
        }
        Expression::ArrowFunctionExpression(arrow) if arrow.expression => {
            let Statement::ExpressionStatement(statement) = arrow.body.statements.first()? else {
                return None;
            };
            object_expression_from_expression(&statement.expression)
        }
        Expression::ArrowFunctionExpression(arrow) => return_object_in_body(&arrow.body.statements),
        Expression::ParenthesizedExpression(value) => {
            setup_return_object_from_expression(&value.expression)
        }
        Expression::TSAsExpression(value) => setup_return_object_from_expression(&value.expression),
        Expression::TSSatisfiesExpression(value) => {
            setup_return_object_from_expression(&value.expression)
        }
        Expression::TSNonNullExpression(value) => {
            setup_return_object_from_expression(&value.expression)
        }
        _ => None,
    }
}

fn return_object_in_body<'a>(
    statements: &'a oxc_allocator::Vec<'a, Statement<'a>>,
) -> Option<&'a ObjectExpression<'a>> {
    statements.iter().find_map(|statement| {
        let Statement::ReturnStatement(ret) = statement else {
            return None;
        };
        ret.argument
            .as_ref()
            .and_then(object_expression_from_expression)
    })
}

fn collect_props_identifier_names<'a>(
    expression: &'a Expression<'a>,
    names: &mut FxHashSet<&'a str>,
) {
    match expression {
        Expression::Identifier(id) if is_safe_value_identifier(id.name.as_str()) => {
            names.insert(id.name.as_str());
        }
        Expression::ParenthesizedExpression(value) => {
            collect_props_identifier_names(&value.expression, names)
        }
        Expression::TSAsExpression(value) => {
            collect_props_identifier_names(&value.expression, names)
        }
        Expression::TSSatisfiesExpression(value) => {
            collect_props_identifier_names(&value.expression, names)
        }
        Expression::TSNonNullExpression(value) => {
            collect_props_identifier_names(&value.expression, names)
        }
        _ => {}
    }
}

fn const_assertion_offset(expression: &Expression<'_>) -> Option<usize> {
    match expression {
        Expression::ObjectExpression(object) => Some(object.span().end as usize),
        Expression::ParenthesizedExpression(value) => const_assertion_offset(&value.expression),
        _ => None,
    }
}
