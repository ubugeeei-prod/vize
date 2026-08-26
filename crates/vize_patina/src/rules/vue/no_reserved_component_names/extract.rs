//! AST extraction helpers for `vue/no-reserved-component-names`.
//!
//! These walk a parsed `<script>` / `<script setup>` program to find the
//! explicit component name declared via the Options API `name` field or
//! `defineOptions({ name })`.

use oxc_ast::ast::{
    Argument, BindingPattern, CallExpression, ExportDefaultDeclarationKind, Expression,
    ObjectExpression, ObjectPropertyKind, Program, PropertyKey, Statement,
};
use oxc_ast_visit::{Visit, walk::walk_call_expression};
use oxc_span::Span;
use vize_s0::FxHashMap;
use vize_s0::String;

pub(super) struct ComponentRegistrationName {
    pub(super) name: String,
    pub(super) span: Span,
}

pub(super) fn options_name(options: &ObjectExpression<'_>) -> Option<ComponentRegistrationName> {
    for property in &options.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            continue;
        };
        if !matches!(
            object_property_key_name(property.computed, &property.key),
            Some("name")
        ) {
            continue;
        }
        return static_expression_name(&property.value);
    }
    None
}

pub(super) fn component_registration_names<'a>(
    options: &'a ObjectExpression<'a>,
) -> impl Iterator<Item = ComponentRegistrationName> + 'a {
    components_object(options)
        .into_iter()
        .flat_map(|components| components.properties.iter())
        .filter_map(|property| {
            let ObjectPropertyKind::ObjectProperty(property) = property else {
                return None;
            };
            let name = object_property_key_name(property.computed, &property.key)?;
            Some(ComponentRegistrationName {
                name: String::from(name),
                span: property.span,
            })
        })
}

pub(super) fn global_component_registration_names<'a>(
    program: &'a Program<'a>,
) -> Vec<ComponentRegistrationName> {
    let mut visitor = GlobalComponentRegistrationVisitor { names: Vec::new() };
    visitor.visit_program(program);
    visitor.names
}

fn components_object<'a>(options: &'a ObjectExpression<'a>) -> Option<&'a ObjectExpression<'a>> {
    for property in &options.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            continue;
        };
        if !matches!(
            object_property_key_name(property.computed, &property.key),
            Some("components")
        ) {
            continue;
        }
        if let Expression::ObjectExpression(object) = &property.value {
            return Some(object);
        }
    }
    None
}

struct GlobalComponentRegistrationVisitor {
    names: Vec<ComponentRegistrationName>,
}

impl<'a> Visit<'a> for GlobalComponentRegistrationVisitor {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if is_vue_component_call(call)
            && let Some(name) = static_argument_name(call.arguments.first())
        {
            self.names.push(name);
        }

        walk_call_expression(self, call);
    }
}

fn is_vue_component_call(call: &CallExpression<'_>) -> bool {
    if call.arguments.len() != 2 {
        return false;
    }
    let Expression::StaticMemberExpression(member) = &call.callee else {
        return false;
    };
    if member.property.name.as_str() != "component" {
        return false;
    }
    is_component_registry_object(&member.object)
}

fn is_component_registry_object(expression: &Expression<'_>) -> bool {
    match expression {
        Expression::Identifier(_) => true,
        Expression::ParenthesizedExpression(parenthesized) => {
            is_component_registry_object(&parenthesized.expression)
        }
        Expression::TSAsExpression(ts_as) => is_component_registry_object(&ts_as.expression),
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            is_component_registry_object(&ts_satisfies.expression)
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            is_component_registry_object(&ts_non_null.expression)
        }
        _ => false,
    }
}

fn static_argument_name<'a>(
    argument: Option<&'a Argument<'a>>,
) -> Option<ComponentRegistrationName> {
    match argument? {
        Argument::StringLiteral(literal) => Some(ComponentRegistrationName {
            name: String::from(literal.value.as_str()),
            span: literal.span,
        }),
        Argument::TemplateLiteral(template) => Some(ComponentRegistrationName {
            name: String::from(template.single_quasi()?.as_str()),
            span: template.span,
        }),
        _ => None,
    }
}

pub(super) fn define_options_name(program: &Program<'_>) -> Option<ComponentRegistrationName> {
    for statement in program.body.iter() {
        let Statement::ExpressionStatement(expression) = statement else {
            continue;
        };
        let Expression::CallExpression(call) = &expression.expression else {
            continue;
        };
        let Expression::Identifier(callee) = &call.callee else {
            continue;
        };
        if !matches!(callee.name.as_str(), "defineOptions") {
            continue;
        }
        if let Some(Argument::ObjectExpression(object)) = call.arguments.first() {
            return options_name(object);
        }
    }
    None
}

fn object_property_key_name<'a>(computed: bool, key: &'a PropertyKey<'a>) -> Option<&'a str> {
    if computed {
        return computed_property_key_name(key);
    }
    property_key_name(key)
}

fn property_key_name<'a>(key: &'a PropertyKey<'a>) -> Option<&'a str> {
    match key {
        PropertyKey::StaticIdentifier(id) => Some(id.name.as_str()),
        PropertyKey::StringLiteral(string) => Some(string.value.as_str()),
        PropertyKey::TemplateLiteral(template) => {
            template.single_quasi().map(|value| value.as_str())
        }
        _ => None,
    }
}

fn computed_property_key_name<'a>(key: &'a PropertyKey<'a>) -> Option<&'a str> {
    match key {
        PropertyKey::StringLiteral(string) => Some(string.value.as_str()),
        PropertyKey::TemplateLiteral(template) => {
            template.single_quasi().map(|value| value.as_str())
        }
        _ => None,
    }
}

fn static_expression_name(expression: &Expression<'_>) -> Option<ComponentRegistrationName> {
    match expression {
        Expression::StringLiteral(literal) => Some(ComponentRegistrationName {
            name: String::from(literal.value.as_str()),
            span: literal.span,
        }),
        Expression::TemplateLiteral(template) => Some(ComponentRegistrationName {
            name: String::from(template.single_quasi()?.as_str()),
            span: template.span,
        }),
        _ => None,
    }
}

pub(super) fn find_component_options<'a>(
    program: &'a Program<'a>,
) -> Option<&'a ObjectExpression<'a>> {
    let mut bindings: FxHashMap<&'a str, &'a ObjectExpression<'a>> = FxHashMap::default();

    for statement in program.body.iter() {
        let Statement::VariableDeclaration(declaration) = statement else {
            continue;
        };
        for declarator in &declaration.declarations {
            let Some(init) = declarator.init.as_ref() else {
                continue;
            };
            if let BindingPattern::BindingIdentifier(id) = &declarator.id
                && let Some(object) = options_from_expression(init, &bindings)
            {
                bindings.insert(id.name.as_str(), object);
            }
        }
    }

    for statement in program.body.iter() {
        let Statement::ExportDefaultDeclaration(export) = statement else {
            continue;
        };
        if let Some(object) = options_from_export(&export.declaration, &bindings) {
            return Some(object);
        }
    }

    None
}

fn options_from_export<'a>(
    declaration: &'a ExportDefaultDeclarationKind<'a>,
    bindings: &FxHashMap<&'a str, &'a ObjectExpression<'a>>,
) -> Option<&'a ObjectExpression<'a>> {
    match declaration {
        ExportDefaultDeclarationKind::ObjectExpression(object) => Some(object),
        ExportDefaultDeclarationKind::CallExpression(call) => options_from_call(call, bindings),
        ExportDefaultDeclarationKind::Identifier(identifier) => {
            bindings.get(identifier.name.as_str()).copied()
        }
        ExportDefaultDeclarationKind::ParenthesizedExpression(paren) => {
            options_from_expression(&paren.expression, bindings)
        }
        ExportDefaultDeclarationKind::TSAsExpression(ts_as) => {
            options_from_expression(&ts_as.expression, bindings)
        }
        ExportDefaultDeclarationKind::TSSatisfiesExpression(ts_satisfies) => {
            options_from_expression(&ts_satisfies.expression, bindings)
        }
        ExportDefaultDeclarationKind::TSNonNullExpression(ts_non_null) => {
            options_from_expression(&ts_non_null.expression, bindings)
        }
        _ => None,
    }
}

fn options_from_expression<'a>(
    expression: &'a Expression<'a>,
    bindings: &FxHashMap<&'a str, &'a ObjectExpression<'a>>,
) -> Option<&'a ObjectExpression<'a>> {
    match expression {
        Expression::ObjectExpression(object) => Some(object),
        Expression::CallExpression(call) => options_from_call(call, bindings),
        Expression::Identifier(identifier) => bindings.get(identifier.name.as_str()).copied(),
        Expression::ParenthesizedExpression(paren) => {
            options_from_expression(&paren.expression, bindings)
        }
        Expression::TSAsExpression(ts_as) => options_from_expression(&ts_as.expression, bindings),
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            options_from_expression(&ts_satisfies.expression, bindings)
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            options_from_expression(&ts_non_null.expression, bindings)
        }
        _ => None,
    }
}

fn options_from_call<'a>(
    call: &'a CallExpression<'a>,
    bindings: &FxHashMap<&'a str, &'a ObjectExpression<'a>>,
) -> Option<&'a ObjectExpression<'a>> {
    let Expression::Identifier(callee) = &call.callee else {
        return None;
    };
    if !matches!(callee.name.as_str(), "defineComponent" | "_defineComponent") {
        return None;
    }
    match call.arguments.first()? {
        Argument::ObjectExpression(object) => Some(object),
        Argument::Identifier(identifier) => bindings.get(identifier.name.as_str()).copied(),
        argument => argument
            .as_expression()
            .and_then(|expression| options_from_expression(expression, bindings)),
    }
}
