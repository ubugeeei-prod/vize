//! What a `createRouter(...)` call's options say, read without following
//! bindings: the `routes` slot, the export names of the instance, and
//! static strings.

use oxc_ast::ast::{
    BindingPattern, Declaration, Expression, ObjectExpression, ObjectPropertyKind, Statement,
};
use oxc_span::GetSpan;
use vize_carton::CompactString;

use super::super::resolve::Script;
use super::extract::Slot;

pub(super) fn routes_property<'b, 'a>(
    options: &'b ObjectExpression<'a>,
) -> Slot<&'b Expression<'a>> {
    let mut routes = Slot::Absent;
    for property in &options.properties {
        match property {
            ObjectPropertyKind::SpreadProperty(_) => routes = Slot::Unknown,
            ObjectPropertyKind::ObjectProperty(property) if property.computed => {
                routes = Slot::Unknown;
            }
            ObjectPropertyKind::ObjectProperty(property) => {
                if property.key.is_specific_static_name("routes") {
                    routes = Slot::Known(&property.value);
                }
            }
        }
    }
    routes
}

/// The export names the router instance created by the call at `span` is
/// reachable under: `const router = createRouter(…)` plus its exports, or
/// `export default createRouter(…)`.
pub(super) fn instance_exports(script: &Script<'_>, span: oxc_span::Span) -> Vec<CompactString> {
    let mut names = Vec::new();
    for statement in &script.program.body {
        let declaration = match statement {
            Statement::VariableDeclaration(declaration) => Some(&**declaration),
            Statement::ExportNamedDeclaration(export) => match &export.declaration {
                Some(Declaration::VariableDeclaration(declaration)) => Some(&**declaration),
                _ => None,
            },
            Statement::ExportDefaultDeclaration(export) => {
                if export
                    .declaration
                    .as_expression()
                    .is_some_and(|expression| expression.get_inner_expression().span() == span)
                {
                    names.push(CompactString::new("default"));
                }
                None
            }
            _ => None,
        };
        for declarator in declaration.into_iter().flat_map(|d| &d.declarations) {
            if let BindingPattern::BindingIdentifier(binding) = &declarator.id
                && declarator
                    .init
                    .as_ref()
                    .is_some_and(|init| init.get_inner_expression().span() == span)
                && let Some(symbol) = binding.symbol_id.get()
            {
                names.extend(script.export_names(symbol).map(CompactString::new));
            }
        }
    }
    names
}

/// A string literal or a substitution-free template literal.
pub(in crate::providers::vue_router) fn static_string(
    expr: &Expression<'_>,
) -> Option<CompactString> {
    match expr.get_inner_expression() {
        Expression::StringLiteral(literal) => Some(CompactString::new(literal.value.as_str())),
        Expression::TemplateLiteral(literal) => literal
            .single_quasi()
            .map(|quasi| CompactString::new(quasi.as_str())),
        _ => None,
    }
}
