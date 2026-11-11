//! Shared walks over component option objects.

use oxc_ast::ast::{
    CallExpression, ExportDefaultDeclaration, ExportDefaultDeclarationKind, Expression,
    ObjectExpression, PropertyKey,
};
use oxc_ast_visit::{Visit, walk};
use oxc_span::{GetSpan, Span};
use vize_l0::String;

pub(super) fn component_names(program: &oxc_ast::ast::Program<'_>) -> Vec<(String, Span)> {
    let mut names = Vec::new();
    each_component_object(program, |object| {
        for property in &object.properties {
            let oxc_ast::ast::ObjectPropertyKind::ObjectProperty(property) = property else {
                continue;
            };
            if property_name(&property.key) != Some("name") {
                continue;
            }
            let Expression::StringLiteral(literal) = &property.value else {
                continue;
            };
            names.push((String::from(literal.value.as_str()), literal.span));
        }
    });
    names
}

pub(super) fn component_options(program: &oxc_ast::ast::Program<'_>) -> Vec<(String, Span)> {
    let mut options = Vec::new();
    each_component_object(program, |object| {
        for property in &object.properties {
            let oxc_ast::ast::ObjectPropertyKind::ObjectProperty(property) = property else {
                continue;
            };
            if let Some(name) = property_name(&property.key) {
                options.push((String::from(name), property.key.span()));
            }
        }
    });
    options
}

fn each_component_object(
    program: &oxc_ast::ast::Program<'_>,
    mut visit: impl FnMut(&ObjectExpression<'_>),
) {
    struct Walk<F>(F);
    impl<'a, F: FnMut(&ObjectExpression<'_>)> Visit<'a> for Walk<F> {
        fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
            if matches!(
                callee_name(&call.callee),
                Some("defineComponent" | "defineNuxtComponent" | "defineOptions")
            ) && let Some(oxc_ast::ast::Argument::ObjectExpression(object)) =
                call.arguments.first()
            {
                (self.0)(object);
            }
            walk::walk_call_expression(self, call);
        }

        fn visit_export_default_declaration(&mut self, declaration: &ExportDefaultDeclaration<'a>) {
            if let Some(object) = object_of_export(&declaration.declaration) {
                (self.0)(object);
            }
            walk::walk_export_default_declaration(self, declaration);
        }
    }
    Walk(&mut visit).visit_program(program);
}

pub(super) fn callee_name<'a>(callee: &'a Expression<'a>) -> Option<&'a str> {
    match callee.get_inner_expression() {
        Expression::Identifier(identifier) => Some(identifier.name.as_str()),
        _ => None,
    }
}

pub(super) fn callee_span(callee: &Expression<'_>) -> Span {
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
