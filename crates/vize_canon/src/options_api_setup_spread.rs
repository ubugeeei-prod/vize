use oxc_allocator::Allocator;
use oxc_ast::ast::{
    Argument, CallExpression, ExportDefaultDeclarationKind, Expression, ObjectExpression,
    ObjectPropertyKind, Program, PropertyKey, Statement,
};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_carton::{FxHashSet, String};
use vize_croquis::{BindingType, Croquis, OptionGroup};

pub(crate) fn suppresses_template_undefined_refs(
    options_api_enabled: bool,
    script_content: Option<&str>,
) -> bool {
    options_api_enabled && script_content.is_some_and(setup_return_has_spread)
}

pub(crate) fn collect_template_setup_bindings(
    summary: &Croquis,
    options_api: bool,
    template_referenced_names: Option<&FxHashSet<String>>,
    script_content: Option<&str>,
) -> Vec<String> {
    let mut names = collect_descriptor_setup_bindings(summary, options_api);
    if suppresses_template_undefined_refs(options_api, script_content) {
        extend_spread_bindings(&mut names, summary, template_referenced_names);
    }
    if let Some(template_referenced_names) = template_referenced_names {
        names.retain(|name| template_referenced_names.contains(name.as_str()));
    }
    names.sort_unstable();
    names.dedup();
    names
}

fn setup_return_has_spread(script: &str) -> bool {
    if !script.contains("export default") {
        return false;
    }
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, script, SourceType::ts()).parse();
    if parsed.panicked {
        return false;
    }
    let Some(options) = component_options_from_program(&parsed.program) else {
        return false;
    };
    let Some(setup) = option_expression_property(options, "setup") else {
        return false;
    };
    setup_return_object_from_expression(setup).is_some_and(object_has_spread)
}

fn collect_descriptor_setup_bindings(summary: &Croquis, options_api: bool) -> Vec<String> {
    if !options_api || summary.bindings.is_script_setup {
        return Vec::new();
    }
    let Some(descriptor) = summary.options_descriptor.as_ref() else {
        return Vec::new();
    };
    descriptor
        .members_in(OptionGroup::Setup)
        .map(|member| member.name.as_str())
        .filter(|name| {
            is_safe_value_identifier(name)
                && matches!(
                    summary.bindings.get(name),
                    Some(BindingType::SetupMaybeRef | BindingType::SetupRef)
                )
        })
        .map(String::from)
        .collect()
}

fn extend_spread_bindings(
    names: &mut Vec<String>,
    summary: &Croquis,
    template_referenced_names: Option<&FxHashSet<String>>,
) {
    if let Some(template_referenced_names) = template_referenced_names {
        names.extend(
            template_referenced_names
                .iter()
                .filter(|name| is_safe_spread_binding(summary, name.as_str()))
                .map(|name| String::from(name.as_str())),
        );
        return;
    }
    names.extend(
        summary
            .undefined_refs
            .iter()
            .filter(|reference| reference.context == "template expression")
            .map(|reference| reference.name.as_str())
            .filter(|name| is_safe_spread_binding(summary, name))
            .map(String::from),
    );
}

fn is_safe_spread_binding(summary: &Croquis, name: &str) -> bool {
    is_safe_value_identifier(name) && summary.bindings.get(name).is_none()
}

fn setup_return_object_from_expression<'a>(
    expression: &'a Expression<'a>,
) -> Option<&'a ObjectExpression<'a>> {
    match expression {
        Expression::FunctionExpression(function) => {
            setup_return_object_in_body(&function.body.as_ref()?.statements)
        }
        Expression::ArrowFunctionExpression(arrow) => {
            if arrow.expression {
                let Statement::ExpressionStatement(expr) = arrow.body.statements.first()? else {
                    return None;
                };
                object_expression_from_expression(&expr.expression)
            } else {
                setup_return_object_in_body(&arrow.body.statements)
            }
        }
        Expression::ParenthesizedExpression(parenthesized) => {
            setup_return_object_from_expression(&parenthesized.expression)
        }
        Expression::TSAsExpression(ts_as) => setup_return_object_from_expression(&ts_as.expression),
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            setup_return_object_from_expression(&ts_satisfies.expression)
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            setup_return_object_from_expression(&ts_non_null.expression)
        }
        _ => None,
    }
}

fn setup_return_object_in_body<'a>(
    statements: &'a oxc_allocator::Vec<'a, Statement<'a>>,
) -> Option<&'a ObjectExpression<'a>> {
    for statement in statements.iter() {
        if let Statement::ReturnStatement(ret) = statement
            && let Some(argument) = ret.argument.as_ref()
        {
            return object_expression_from_expression(argument);
        }
    }
    None
}

fn object_has_spread(object: &ObjectExpression<'_>) -> bool {
    object
        .properties
        .iter()
        .any(|property| matches!(property, ObjectPropertyKind::SpreadProperty(_)))
}

fn option_expression_property<'a>(
    object: &'a ObjectExpression<'a>,
    key_name: &str,
) -> Option<&'a Expression<'a>> {
    object.properties.iter().find_map(|property| {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            return None;
        };
        if property.computed || property_key_name(&property.key) != Some(key_name) {
            return None;
        }
        Some(&property.value)
    })
}

fn component_options_from_program<'a>(
    program: &'a Program<'a>,
) -> Option<&'a ObjectExpression<'a>> {
    program.body.iter().find_map(|statement| {
        let Statement::ExportDefaultDeclaration(export) = statement else {
            return None;
        };
        component_options_from_declaration(&export.declaration)
    })
}

fn component_options_from_declaration<'a>(
    declaration: &'a ExportDefaultDeclarationKind<'a>,
) -> Option<&'a ObjectExpression<'a>> {
    match declaration {
        ExportDefaultDeclarationKind::ObjectExpression(object) => Some(object.as_ref()),
        ExportDefaultDeclarationKind::CallExpression(call) => component_options_from_call(call),
        ExportDefaultDeclarationKind::TSAsExpression(ts_as) => {
            component_options_from_expression(&ts_as.expression)
        }
        ExportDefaultDeclarationKind::TSSatisfiesExpression(ts_satisfies) => {
            component_options_from_expression(&ts_satisfies.expression)
        }
        ExportDefaultDeclarationKind::TSNonNullExpression(ts_non_null) => {
            component_options_from_expression(&ts_non_null.expression)
        }
        _ => None,
    }
}

fn component_options_from_expression<'a>(
    expression: &'a Expression<'a>,
) -> Option<&'a ObjectExpression<'a>> {
    match expression {
        Expression::ObjectExpression(object) => Some(object.as_ref()),
        Expression::CallExpression(call) => component_options_from_call(call),
        Expression::ParenthesizedExpression(parenthesized) => {
            component_options_from_expression(&parenthesized.expression)
        }
        Expression::TSAsExpression(ts_as) => component_options_from_expression(&ts_as.expression),
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            component_options_from_expression(&ts_satisfies.expression)
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            component_options_from_expression(&ts_non_null.expression)
        }
        _ => None,
    }
}

fn component_options_from_call<'a>(
    call: &'a CallExpression<'a>,
) -> Option<&'a ObjectExpression<'a>> {
    if !is_define_component_callee(&call.callee) {
        return None;
    }
    let first = call.arguments.first()?;
    match first {
        Argument::ObjectExpression(object) => Some(object.as_ref()),
        Argument::CallExpression(call) => component_options_from_call(call),
        Argument::ParenthesizedExpression(parenthesized) => {
            component_options_from_expression(&parenthesized.expression)
        }
        Argument::TSAsExpression(ts_as) => component_options_from_expression(&ts_as.expression),
        Argument::TSSatisfiesExpression(ts_satisfies) => {
            component_options_from_expression(&ts_satisfies.expression)
        }
        Argument::TSNonNullExpression(ts_non_null) => {
            component_options_from_expression(&ts_non_null.expression)
        }
        _ => None,
    }
}

fn is_define_component_callee(callee: &Expression<'_>) -> bool {
    match callee {
        Expression::Identifier(callee) => {
            matches!(callee.name.as_str(), "defineComponent" | "_defineComponent")
        }
        Expression::StaticMemberExpression(member) => {
            matches!(
                member.property.name.as_str(),
                "defineComponent" | "_defineComponent"
            )
        }
        _ => false,
    }
}

fn object_expression_from_expression<'a>(
    expression: &'a Expression<'a>,
) -> Option<&'a ObjectExpression<'a>> {
    match expression {
        Expression::ObjectExpression(object) => Some(object.as_ref()),
        Expression::ParenthesizedExpression(parenthesized) => {
            object_expression_from_expression(&parenthesized.expression)
        }
        Expression::TSAsExpression(ts_as) => object_expression_from_expression(&ts_as.expression),
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            object_expression_from_expression(&ts_satisfies.expression)
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            object_expression_from_expression(&ts_non_null.expression)
        }
        _ => None,
    }
}

fn property_key_name<'a>(key: &'a PropertyKey<'a>) -> Option<&'a str> {
    match key {
        PropertyKey::StaticIdentifier(identifier) => Some(identifier.name.as_str()),
        PropertyKey::StringLiteral(string) => Some(string.value.as_str()),
        _ => None,
    }
}

fn is_safe_value_identifier(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first.is_ascii_alphabetic() || first == '_' || first == '$') {
        return false;
    }
    chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '$')
}
