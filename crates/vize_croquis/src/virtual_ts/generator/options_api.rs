//! Options API template-binding support for Croquis virtual TS.

use super::{BindingType, ScriptParseResult, VirtualTsGenerator};
use crate::croquis::OptionGroup;
use oxc_ast::ast::{
    Argument, CallExpression, ExportDefaultDeclarationKind, Expression, ObjectExpression,
    ObjectPropertyKind, Program, PropertyKey, Statement,
};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};
use vize_carton::{String, append};

impl VirtualTsGenerator {
    pub(crate) fn emit_options_api_template_bindings(
        &mut self,
        script_content: &str,
        parse_result: &ScriptParseResult,
    ) {
        if !parse_result.is_non_setup_script {
            return;
        }
        let names = options_prop_names(parse_result);
        if names.is_empty() {
            return;
        }
        let Some(props_source) = find_direct_options_props_object(script_content) else {
            return;
        };

        self.emit_line("// Options API props exposed to template");
        self.emit_generated_line(|output| {
            append!(
                *output,
                "const __vize_options_props = {props_source} as const;"
            );
        });
        self.emit_line(
            "type __VizeOptionsProps = __RuntimePropShape<typeof __vize_options_props>;",
        );
        for name in names {
            if !is_safe_identifier(name) {
                continue;
            }
            self.emit_generated_line(|output| {
                append!(
                    *output,
                    "const {name} = undefined as unknown as __VizeOptionsProps["
                );
                push_quoted_ts_string(output, name);
                output.push_str("];");
            });
        }
        self.emit_line("");
    }
}

fn options_prop_names(parse_result: &ScriptParseResult) -> Vec<&str> {
    let Some(descriptor) = parse_result.options_descriptor.as_ref() else {
        return Vec::new();
    };
    descriptor
        .members
        .iter()
        .filter(|member| member.group == OptionGroup::Props)
        .filter_map(|member| {
            let name = member.name.as_str();
            matches!(
                parse_result.bindings.get(name),
                Some(BindingType::Props | BindingType::PropsAliased)
            )
            .then_some(name)
        })
        .collect()
}

fn find_direct_options_props_object(script: &str) -> Option<&str> {
    let allocator = oxc_allocator::Allocator::default();
    let parsed = Parser::new(&allocator, script, SourceType::ts()).parse();
    if parsed.panicked {
        return None;
    }
    let options = component_options_from_program(&parsed.program)?;
    let props = option_expression_property(options, "props")?;
    object_expression_source(script, props)
}

fn object_expression_source<'a>(script: &'a str, expression: &Expression<'_>) -> Option<&'a str> {
    match expression {
        Expression::ObjectExpression(object) => source_slice(script, object.span()),
        Expression::ParenthesizedExpression(parenthesized) => {
            object_expression_source(script, &parenthesized.expression)
        }
        Expression::TSAsExpression(ts_as) => object_expression_source(script, &ts_as.expression),
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            object_expression_source(script, &ts_satisfies.expression)
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            object_expression_source(script, &ts_non_null.expression)
        }
        _ => None,
    }
}

fn component_options_from_program<'a>(
    program: &'a Program<'a>,
) -> Option<&'a ObjectExpression<'a>> {
    program.body.iter().find_map(|statement| {
        let Statement::ExportDefaultDeclaration(export) = statement else {
            return None;
        };
        component_options_from_export(&export.declaration)
    })
}

fn component_options_from_export<'a>(
    declaration: &'a ExportDefaultDeclarationKind<'a>,
) -> Option<&'a ObjectExpression<'a>> {
    match declaration {
        ExportDefaultDeclarationKind::ObjectExpression(object) => Some(object.as_ref()),
        ExportDefaultDeclarationKind::CallExpression(call) => component_options_from_call(call),
        ExportDefaultDeclarationKind::ParenthesizedExpression(parenthesized) => {
            component_options_from_expression(&parenthesized.expression)
        }
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
    if !is_component_options_callee(&call.callee) {
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

fn is_component_options_callee(callee: &Expression<'_>) -> bool {
    match callee {
        Expression::Identifier(callee) => {
            matches!(
                callee.name.as_str(),
                "defineComponent" | "_defineComponent" | "extend"
            )
        }
        Expression::StaticMemberExpression(member) => match member.property.name.as_str() {
            "defineComponent" | "_defineComponent" => true,
            "extend" => matches!(
                &member.object,
                Expression::Identifier(identifier)
                    if matches!(identifier.name.as_str(), "Vue" | "_Vue")
            ),
            _ => false,
        },
        _ => false,
    }
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

fn property_key_name<'a>(key: &'a PropertyKey<'a>) -> Option<&'a str> {
    match key {
        PropertyKey::StaticIdentifier(identifier) => Some(identifier.name.as_str()),
        PropertyKey::StringLiteral(string) => Some(string.value.as_str()),
        _ => None,
    }
}

fn source_slice(script: &str, span: oxc_span::Span) -> Option<&str> {
    script.get(span.start as usize..span.end as usize)
}

fn is_safe_identifier(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first.is_ascii_alphabetic() || first == '_' || first == '$')
        && chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '$')
}

fn push_quoted_ts_string(output: &mut String, value: &str) {
    output.push('"');
    for ch in value.chars() {
        match ch {
            '\\' => output.push_str("\\\\"),
            '"' => output.push_str("\\\""),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            _ => output.push(ch),
        }
    }
    output.push('"');
}
