use oxc_allocator::Allocator;
use oxc_ast::ast::{
    Argument, CallExpression, Expression, ObjectExpression, ObjectPropertyKind, PropertyKey,
    Statement,
};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_carton::{String, ToCompactString};

use super::MacroCall;

pub(crate) struct DefineModelMetadata {
    pub(crate) name: String,
    pub(crate) options: Option<String>,
    pub(crate) runtime_options: Option<String>,
}

pub(crate) fn define_model_name(source: &str, call: &MacroCall) -> String {
    define_model_metadata(source, call).name
}

pub(crate) fn define_model_metadata(source: &str, call: &MacroCall) -> DefineModelMetadata {
    let Some(source) = source.get(call.start..call.end) else {
        return default_metadata();
    };
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::ts()).parse();
    if parsed.panicked || !parsed.errors.is_empty() {
        return default_metadata();
    }
    let Some(Statement::ExpressionStatement(statement)) = parsed.program.body.first() else {
        return default_metadata();
    };
    let Expression::CallExpression(call) = &statement.expression else {
        return default_metadata();
    };

    extract_metadata_from_call(call, source)
}

fn extract_metadata_from_call(call: &CallExpression<'_>, source: &str) -> DefineModelMetadata {
    let name_arg = call.arguments.first().and_then(argument_string_literal);
    let name = name_arg
        .map(|name| name.to_compact_string())
        .unwrap_or_else(|| "modelValue".to_compact_string());
    let options_index = if name_arg.is_some() { 1 } else { 0 };
    let (options, runtime_options) = call
        .arguments
        .get(options_index)
        .and_then(argument_object)
        .map_or((None, None), |object| split_model_options(object, source));

    DefineModelMetadata {
        name,
        options,
        runtime_options,
    }
}

fn default_metadata() -> DefineModelMetadata {
    DefineModelMetadata {
        name: "modelValue".into(),
        options: None,
        runtime_options: None,
    }
}

pub(crate) fn define_model_prop_option_spans(
    source: &str,
    macro_call: &MacroCall,
) -> Vec<(usize, usize)> {
    let Some(call_source) = source.get(macro_call.start..macro_call.end) else {
        return Vec::new();
    };
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, call_source, SourceType::ts()).parse();
    let Some(Statement::ExpressionStatement(statement)) = parsed.program.body.first() else {
        return Vec::new();
    };
    let Expression::CallExpression(call) = &statement.expression else {
        return Vec::new();
    };
    let name_arg = call.arguments.first().and_then(argument_string_literal);
    let options_index = usize::from(name_arg.is_some());
    let Some(object) = call.arguments.get(options_index).and_then(argument_object) else {
        return Vec::new();
    };
    if has_dynamic_model_options(object) {
        return Vec::new();
    }

    object
        .properties
        .iter()
        .filter_map(|property| {
            let ObjectPropertyKind::ObjectProperty(property) = property else {
                return None;
            };
            if matches!(static_property_name(&property.key), Some("get" | "set")) {
                return None;
            }
            Some((
                macro_call.start + property.span.start as usize,
                macro_call.start + property.span.end as usize,
            ))
        })
        .collect()
}

fn argument_string_literal<'a>(argument: &'a Argument<'a>) -> Option<&'a str> {
    match argument {
        Argument::StringLiteral(literal) => Some(literal.value.as_str()),
        Argument::ParenthesizedExpression(expr) => expression_string_literal(&expr.expression),
        Argument::TSAsExpression(expr) => expression_string_literal(&expr.expression),
        Argument::TSSatisfiesExpression(expr) => expression_string_literal(&expr.expression),
        Argument::TSNonNullExpression(expr) => expression_string_literal(&expr.expression),
        _ => None,
    }
}

fn argument_object<'a>(argument: &'a Argument<'a>) -> Option<&'a ObjectExpression<'a>> {
    match argument {
        Argument::ObjectExpression(object) => Some(object),
        Argument::ParenthesizedExpression(expr) => expression_object(&expr.expression),
        Argument::TSAsExpression(expr) => expression_object(&expr.expression),
        Argument::TSSatisfiesExpression(expr) => expression_object(&expr.expression),
        Argument::TSNonNullExpression(expr) => expression_object(&expr.expression),
        _ => None,
    }
}

fn expression_string_literal<'a>(expression: &'a Expression<'a>) -> Option<&'a str> {
    match expression {
        Expression::StringLiteral(literal) => Some(literal.value.as_str()),
        Expression::ParenthesizedExpression(expr) => expression_string_literal(&expr.expression),
        Expression::TSAsExpression(expr) => expression_string_literal(&expr.expression),
        Expression::TSSatisfiesExpression(expr) => expression_string_literal(&expr.expression),
        Expression::TSNonNullExpression(expr) => expression_string_literal(&expr.expression),
        _ => None,
    }
}

fn expression_object<'a>(expression: &'a Expression<'a>) -> Option<&'a ObjectExpression<'a>> {
    match expression {
        Expression::ObjectExpression(object) => Some(object),
        Expression::ParenthesizedExpression(expr) => expression_object(&expr.expression),
        Expression::TSAsExpression(expr) => expression_object(&expr.expression),
        Expression::TSSatisfiesExpression(expr) => expression_object(&expr.expression),
        Expression::TSNonNullExpression(expr) => expression_object(&expr.expression),
        _ => None,
    }
}

fn has_dynamic_model_options(object: &ObjectExpression<'_>) -> bool {
    object.properties.iter().any(|property| match property {
        ObjectPropertyKind::ObjectProperty(property) => property.computed,
        ObjectPropertyKind::SpreadProperty(_) => true,
    })
}

fn split_model_options(
    object: &ObjectExpression<'_>,
    source: &str,
) -> (Option<String>, Option<String>) {
    let object_source = source
        .get(object.span.start as usize..object.span.end as usize)
        .map(String::from);
    if has_dynamic_model_options(object) {
        return (object_source.clone(), object_source);
    }

    let object_start = object.span.start as usize;
    let object_end = object.span.end as usize;
    let Some(object_close) = object_end.checked_sub(1) else {
        return (object_source, None);
    };
    let Some(object_source) = source.get(object_start..object_end) else {
        return (None, None);
    };
    let mut prop_options = object_source.to_compact_string();
    let mut runtime_options = object_source.to_compact_string();
    let properties: Vec<_> = object
        .properties
        .iter()
        .filter_map(|property| {
            let ObjectPropertyKind::ObjectProperty(property) = property else {
                return None;
            };
            Some((
                property.span.start as usize,
                static_property_name(&property.key),
            ))
        })
        .collect();
    let has_runtime_accessors = properties
        .iter()
        .any(|(_, key)| matches!(key, Some("get" | "set")));

    for (index, (start, key)) in properties.iter().enumerate().rev() {
        let end = properties
            .get(index + 1)
            .map(|(next_start, _)| *next_start)
            .unwrap_or(object_close);
        let range = start - object_start..end - object_start;
        if matches!(key, Some("get" | "set")) {
            prop_options.replace_range(range, "");
        } else {
            runtime_options.replace_range(range, "");
        }
    }
    (
        Some(prop_options),
        has_runtime_accessors.then_some(runtime_options),
    )
}

fn static_property_name<'a>(key: &'a PropertyKey<'a>) -> Option<&'a str> {
    match key {
        PropertyKey::StaticIdentifier(identifier) => Some(identifier.name.as_str()),
        PropertyKey::StringLiteral(literal) => Some(literal.value.as_str()),
        _ => None,
    }
}
