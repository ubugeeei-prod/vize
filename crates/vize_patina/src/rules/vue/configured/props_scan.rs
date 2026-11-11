//! Prop name collection shared by the configurable prop rules.

use oxc_ast::ast::{
    Argument, ArrayExpressionElement, CallExpression, ExportDefaultDeclaration,
    ExportDefaultDeclarationKind, Expression, ObjectExpression, ObjectPropertyKind, Program,
    PropertyKey,
};
use oxc_ast_visit::{Visit, walk};
use oxc_span::{GetSpan, Span};
use vize_l0::String;

pub(super) struct Group {
    pub(super) anchor: Span,
    pub(super) props: Vec<(String, Span)>,
}

pub(super) fn groups(program: &Program<'_>) -> Vec<Group> {
    let mut collector = Collector {
        program,
        groups: Vec::new(),
    };
    collector.visit_program(program);
    collector.groups
}

pub(super) fn unique_count(props: &[(String, Span)]) -> usize {
    let mut names: Vec<&str> = props.iter().map(|(name, _)| name.as_str()).collect();
    names.sort_unstable();
    names.dedup();
    names.len()
}

struct Collector<'a> {
    program: &'a Program<'a>,
    groups: Vec<Group>,
}

impl<'a> Visit<'a> for Collector<'a> {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        match callee_name(&call.callee) {
            Some("defineProps") => {
                self.groups.push(Group {
                    anchor: callee_span(&call.callee),
                    props: define_props(self.program, call),
                });
            }
            Some("defineComponent" | "defineNuxtComponent") => {
                if let Some(Argument::ObjectExpression(object)) = call.arguments.first() {
                    self.options_props(object);
                }
            }
            _ => {}
        }
        walk::walk_call_expression(self, call);
    }

    fn visit_export_default_declaration(&mut self, declaration: &ExportDefaultDeclaration<'a>) {
        if let Some(object) = object_of_export(&declaration.declaration) {
            self.options_props(object);
        }
        walk::walk_export_default_declaration(self, declaration);
    }
}

impl<'a> Collector<'a> {
    fn options_props(&mut self, object: &ObjectExpression<'_>) {
        for property in &object.properties {
            let ObjectPropertyKind::ObjectProperty(property) = property else {
                continue;
            };
            if property_name(&property.key) != Some("props") {
                continue;
            }
            let props = match &property.value {
                Expression::ObjectExpression(props) => object_keys(props),
                Expression::ArrayExpression(array) => array_names(array),
                _ => Vec::new(),
            };
            self.groups.push(Group {
                anchor: property.key.span(),
                props,
            });
        }
    }
}

fn define_props(program: &Program<'_>, call: &CallExpression<'_>) -> Vec<(String, Span)> {
    if let Some(types) = &call.type_arguments
        && let Some(ty) = types.params.first()
    {
        return super::props_type::props_from_type(program, ty, 4);
    }
    call.arguments
        .first()
        .map(runtime_argument)
        .unwrap_or_default()
}

fn runtime_argument(argument: &Argument<'_>) -> Vec<(String, Span)> {
    match argument {
        Argument::ObjectExpression(object) => object_keys(object),
        Argument::ArrayExpression(array) => array_names(array),
        _ => Vec::new(),
    }
}

fn object_keys(object: &ObjectExpression<'_>) -> Vec<(String, Span)> {
    object
        .properties
        .iter()
        .filter_map(|property| {
            let ObjectPropertyKind::ObjectProperty(property) = property else {
                return None;
            };
            super::props_type::named(&property.key)
        })
        .collect()
}

fn array_names(array: &oxc_ast::ast::ArrayExpression<'_>) -> Vec<(String, Span)> {
    array
        .elements
        .iter()
        .filter_map(|element| {
            let ArrayExpressionElement::StringLiteral(literal) = element else {
                return None;
            };
            Some((String::from(literal.value.as_str()), literal.span))
        })
        .collect()
}

fn callee_name<'a>(callee: &'a Expression<'a>) -> Option<&'a str> {
    match callee.get_inner_expression() {
        Expression::Identifier(identifier) => Some(identifier.name.as_str()),
        _ => None,
    }
}

fn callee_span(callee: &Expression<'_>) -> Span {
    match callee.get_inner_expression() {
        Expression::Identifier(identifier) => identifier.span,
        _ => callee.span(),
    }
}

fn property_name<'a>(key: &'a PropertyKey<'a>) -> Option<&'a str> {
    match key {
        PropertyKey::StaticIdentifier(identifier) => Some(identifier.name.as_str()),
        PropertyKey::StringLiteral(literal) => Some(literal.value.as_str()),
        _ => None,
    }
}

fn object_of_export<'a>(
    declaration: &'a ExportDefaultDeclarationKind<'a>,
) -> Option<&'a ObjectExpression<'a>> {
    match declaration {
        ExportDefaultDeclarationKind::ObjectExpression(object) => Some(object),
        ExportDefaultDeclarationKind::TSAsExpression(expression) => {
            object_of(&expression.expression)
        }
        ExportDefaultDeclarationKind::TSSatisfiesExpression(expression) => {
            object_of(&expression.expression)
        }
        ExportDefaultDeclarationKind::ParenthesizedExpression(expression) => {
            object_of(&expression.expression)
        }
        _ => None,
    }
}

fn object_of<'a>(expression: &'a Expression<'a>) -> Option<&'a ObjectExpression<'a>> {
    match expression {
        Expression::ObjectExpression(object) => Some(object),
        Expression::ParenthesizedExpression(expression) => object_of(&expression.expression),
        Expression::TSAsExpression(expression) => object_of(&expression.expression),
        Expression::TSSatisfiesExpression(expression) => object_of(&expression.expression),
        _ => None,
    }
}
