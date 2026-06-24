//! Extract Storybook CSF (Component Story Format) metadata from a parsed module.
//!
//! Walks the oxc AST of a `*.stories.*` file to recover the component identifier
//! and its import path, the meta `title`, and an ordered list of named-export
//! stories. The story payloads (`render` arrow body and `args` object) are kept
//! as borrowed AST references so the JSX/template emitter can slice the original
//! source by span.

use oxc_ast::ast::{
    Declaration, ExportDefaultDeclarationKind, Expression, ImportDeclarationSpecifier,
    ObjectExpression, ObjectPropertyKind, Program, PropertyKey, Statement, VariableDeclarator,
};
use vize_carton::String;

/// A single CSF story (named export) ready for template emission.
pub(super) struct CsfStory<'a> {
    /// Variant name (the `name:` override if present, else the export name).
    pub name: String,
    /// `render` arrow/function body expression (a JSX element/fragment), if any.
    pub render: Option<&'a Expression<'a>>,
    /// `args` object literal, if present.
    pub args: Option<&'a ObjectExpression<'a>>,
}

/// Everything migration needs from one CSF module.
pub(super) struct CsfModule<'a> {
    /// Module specifier the component was imported from (e.g. `./AfButton.vue`).
    pub component_path: Option<String>,
    /// Meta `title` value (raw, may contain `Category/Name`).
    pub title: Option<String>,
    /// Ordered stories.
    pub stories: Vec<CsfStory<'a>>,
}

/// Extract CSF metadata from a parsed program.
pub(super) fn extract_csf<'a>(program: &'a Program<'a>) -> CsfModule<'a> {
    let meta = find_meta_object(program);
    let component_local = meta.and_then(meta_component_local);
    let title = meta.and_then(meta_title);
    let component_path = component_local
        .as_deref()
        .and_then(|local| find_import_source(program, local));

    let stories = collect_stories(program);

    CsfModule {
        component_path,
        title,
        stories,
    }
}

/// Find the meta object literal: either the `export default {...}` expression or
/// the `const meta = {...}` that is later `export default meta`.
fn find_meta_object<'a>(program: &'a Program<'a>) -> Option<&'a ObjectExpression<'a>> {
    let mut default_export_name: Option<&str> = None;

    for stmt in &program.body {
        if let Statement::ExportDefaultDeclaration(decl) = stmt {
            match unwrap_default_kind(&decl.declaration) {
                DefaultTarget::Object(object) => return Some(object),
                DefaultTarget::Identifier(name) => default_export_name = Some(name),
                DefaultTarget::Other => {}
            }
        }
    }

    let name = default_export_name?;
    for stmt in &program.body {
        if let Statement::VariableDeclaration(decl) = stmt {
            for declarator in &decl.declarations {
                if binding_name(declarator) == Some(name)
                    && let Some(init) = declarator.init.as_ref()
                    && let Some(object) = unwrap_object(init)
                {
                    return Some(object);
                }
            }
        }
    }
    None
}

/// What an `export default ...` resolves to after peeling TS/paren wrappers.
enum DefaultTarget<'a> {
    Object(&'a ObjectExpression<'a>),
    Identifier(&'a str),
    Other,
}

/// Peel `satisfies`/`as`/parentheses on an `export default` declaration to reach
/// either an object literal or a bare identifier reference.
fn unwrap_default_kind<'a>(decl: &'a ExportDefaultDeclarationKind<'a>) -> DefaultTarget<'a> {
    match decl {
        ExportDefaultDeclarationKind::ObjectExpression(object) => DefaultTarget::Object(object),
        ExportDefaultDeclarationKind::Identifier(ident) => {
            DefaultTarget::Identifier(ident.name.as_str())
        }
        ExportDefaultDeclarationKind::TSSatisfiesExpression(inner) => {
            unwrap_expression_target(&inner.expression)
        }
        ExportDefaultDeclarationKind::TSAsExpression(inner) => {
            unwrap_expression_target(&inner.expression)
        }
        ExportDefaultDeclarationKind::ParenthesizedExpression(inner) => {
            unwrap_expression_target(&inner.expression)
        }
        _ => DefaultTarget::Other,
    }
}

fn unwrap_expression_target<'a>(expr: &'a Expression<'a>) -> DefaultTarget<'a> {
    match unwrap_expression(expr) {
        Expression::ObjectExpression(object) => DefaultTarget::Object(object),
        Expression::Identifier(ident) => DefaultTarget::Identifier(ident.name.as_str()),
        _ => DefaultTarget::Other,
    }
}

/// Read the meta `component` property's local identifier name.
fn meta_component_local(meta: &ObjectExpression<'_>) -> Option<String> {
    let value = object_property_value(meta, "component")?;
    if let Expression::Identifier(ident) = unwrap_expression(value) {
        Some(ident.name.as_str().into())
    } else {
        None
    }
}

/// Read the meta `title` string literal.
fn meta_title(meta: &ObjectExpression<'_>) -> Option<String> {
    let value = object_property_value(meta, "title")?;
    string_literal_value(value)
}

/// Find the import specifier source path for a given local identifier.
fn find_import_source(program: &Program<'_>, local: &str) -> Option<String> {
    for stmt in &program.body {
        if let Statement::ImportDeclaration(decl) = stmt
            && let Some(specifiers) = decl.specifiers.as_ref()
        {
            for specifier in specifiers {
                let matches = match specifier {
                    ImportDeclarationSpecifier::ImportDefaultSpecifier(spec) => {
                        spec.local.name.as_str() == local
                    }
                    ImportDeclarationSpecifier::ImportSpecifier(spec) => {
                        spec.local.name.as_str() == local
                    }
                    ImportDeclarationSpecifier::ImportNamespaceSpecifier(spec) => {
                        spec.local.name.as_str() == local
                    }
                };
                if matches {
                    return Some(decl.source.value.as_str().into());
                }
            }
        }
    }
    None
}

/// Collect `export const NAME = {...}` stories in source order.
fn collect_stories<'a>(program: &'a Program<'a>) -> Vec<CsfStory<'a>> {
    let mut stories = Vec::new();

    for stmt in &program.body {
        let Statement::ExportNamedDeclaration(decl) = stmt else {
            continue;
        };
        let Some(Declaration::VariableDeclaration(var)) = decl.declaration.as_ref() else {
            continue;
        };
        for declarator in &var.declarations {
            let Some(export_name) = binding_name(declarator) else {
                continue;
            };
            let Some(init) = declarator.init.as_ref() else {
                continue;
            };
            let Some(object) = unwrap_object(init) else {
                continue;
            };
            stories.push(story_from_object(export_name, object));
        }
    }

    stories
}

/// Build a [`CsfStory`] from a story object literal.
fn story_from_object<'a>(export_name: &str, object: &'a ObjectExpression<'a>) -> CsfStory<'a> {
    let name = object_property_value(object, "name")
        .and_then(string_literal_value)
        .unwrap_or_else(|| export_name.into());

    let render = object_property_value(object, "render").and_then(render_body);
    let args = object_property_value(object, "args").and_then(|value| {
        if let Expression::ObjectExpression(object) = unwrap_expression(value) {
            Some(&**object)
        } else {
            None
        }
    });

    CsfStory { name, render, args }
}

/// Reach the single JSX-returning expression of a `render` arrow/function.
fn render_body<'a>(value: &'a Expression<'a>) -> Option<&'a Expression<'a>> {
    match unwrap_expression(value) {
        Expression::ArrowFunctionExpression(arrow) => {
            if arrow.expression {
                // `() => <expr>`: body holds a single expression statement.
                first_expression_statement(&arrow.body.statements)
            } else {
                first_return_argument(&arrow.body.statements)
            }
        }
        Expression::FunctionExpression(func) => func
            .body
            .as_ref()
            .and_then(|body| first_return_argument(&body.statements)),
        _ => None,
    }
}

fn first_expression_statement<'a>(statements: &'a [Statement<'a>]) -> Option<&'a Expression<'a>> {
    match statements.first()? {
        Statement::ExpressionStatement(stmt) => Some(&stmt.expression),
        _ => None,
    }
}

fn first_return_argument<'a>(statements: &'a [Statement<'a>]) -> Option<&'a Expression<'a>> {
    match statements.first()? {
        Statement::ReturnStatement(stmt) => stmt.argument.as_ref(),
        _ => None,
    }
}

/// Look up an object property value by its static identifier or string key.
pub(super) fn object_property_value<'a>(
    object: &'a ObjectExpression<'a>,
    key: &str,
) -> Option<&'a Expression<'a>> {
    for property in &object.properties {
        if let ObjectPropertyKind::ObjectProperty(prop) = property
            && !prop.computed
            && property_key_name(&prop.key) == Some(key)
        {
            return Some(&prop.value);
        }
    }
    None
}

fn property_key_name<'a>(key: &'a PropertyKey<'a>) -> Option<&'a str> {
    match key {
        PropertyKey::StaticIdentifier(ident) => Some(ident.name.as_str()),
        PropertyKey::StringLiteral(literal) => Some(literal.value.as_str()),
        _ => None,
    }
}

/// Strip `satisfies`/`as`/parenthesized wrappers to reach the underlying object.
pub(super) fn unwrap_object<'a>(expr: &'a Expression<'a>) -> Option<&'a ObjectExpression<'a>> {
    if let Expression::ObjectExpression(object) = unwrap_expression(expr) {
        Some(&**object)
    } else {
        None
    }
}

/// Peel `TSSatisfiesExpression`, `TSAsExpression`, and parentheses.
pub(super) fn unwrap_expression<'a>(expr: &'a Expression<'a>) -> &'a Expression<'a> {
    let mut current = expr;
    loop {
        current = match current {
            Expression::TSSatisfiesExpression(inner) => &inner.expression,
            Expression::TSAsExpression(inner) => &inner.expression,
            Expression::ParenthesizedExpression(inner) => &inner.expression,
            other => return other,
        };
    }
}

fn string_literal_value(expr: &Expression<'_>) -> Option<String> {
    if let Expression::StringLiteral(literal) = unwrap_expression(expr) {
        Some(literal.value.as_str().into())
    } else {
        None
    }
}

fn binding_name<'a>(declarator: &'a VariableDeclarator<'a>) -> Option<&'a str> {
    match &declarator.id {
        oxc_ast::ast::BindingPattern::BindingIdentifier(ident) => Some(ident.name.as_str()),
        _ => None,
    }
}

#[cfg(test)]
mod tests;
