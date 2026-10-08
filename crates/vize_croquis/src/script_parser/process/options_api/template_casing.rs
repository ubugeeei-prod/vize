//! Authored registration keys for template tag casing.

use oxc_ast::ast::{
    Argument, ExportDefaultDeclarationKind, Expression, ObjectExpression, ObjectPropertyKind,
    PropertyKey,
};
use vize_carton::CompactString;

use super::{ScriptParseResult, is_component_options_callee, property_key_name};

pub(super) fn authored_casing_options<'a>(
    declaration: &'a ExportDefaultDeclarationKind<'a>,
) -> Option<&'a ObjectExpression<'a>> {
    match declaration {
        ExportDefaultDeclarationKind::ObjectExpression(object) => Some(object),
        ExportDefaultDeclarationKind::CallExpression(call)
            if is_component_options_callee(&call.callee) =>
        {
            match call.arguments.first()? {
                Argument::ObjectExpression(object) => Some(object),
                _ => None,
            }
        }
        _ => None,
    }
}

/// Casing checks authored keys, regardless of the component value. Unlike
/// identity resolution, this does not follow local objects or spreads.
pub(super) fn collect_direct_component_names(
    result: &mut ScriptParseResult,
    options: &ObjectExpression<'_>,
) {
    if result.skip_diagnostics {
        return;
    }
    let components = options.properties.iter().find_map(|property| {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            return None;
        };
        if static_key_name(&property.key) != Some("components") {
            return None;
        }
        let Expression::ObjectExpression(object) = &property.value else {
            return None;
        };
        Some(object)
    });
    let Some(components) = components else {
        return;
    };
    for property in &components.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            continue;
        };
        if let Some(name) = static_key_name(&property.key) {
            result
                .template_component_registrations
                .option_names
                .insert(CompactString::new(name));
        }
    }
}

fn static_key_name<'a>(key: &'a PropertyKey<'a>) -> Option<&'a str> {
    if let PropertyKey::TemplateLiteral(template) = key {
        return template
            .expressions
            .is_empty()
            .then(|| template.quasis.first())
            .flatten()
            .and_then(|quasi| quasi.value.cooked.as_ref().map(|value| value.as_str()));
    }
    property_key_name(key)
}
