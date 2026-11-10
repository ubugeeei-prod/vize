use oxc_allocator::Allocator;
use oxc_ast::ast::{
    ArrayExpressionElement, Expression, PropertyKey, TSSignature, TSType, TSTypeName,
};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};
use oxc_syntax::identifier::{is_identifier_name, is_identifier_part};
use vize_carton::String;
use vize_croquis::{Croquis, OptionGroup};

use super::options_api::{
    component_options_from_program, option_expression_property, source_slice,
};
pub(super) use crate::options_api_setup_spread::is_safe_value_identifier;
use crate::virtual_ts::props::OptionsApiPropsSource;

pub(super) fn follows_default_keyword(rest: &str) -> bool {
    rest.chars()
        .next()
        .is_none_or(|ch| ch != '\\' && !is_identifier_part(ch))
}

pub(super) fn extend_options_api_descriptor_names(names: &mut Vec<String>, summary: &Croquis) {
    let Some(descriptor) = summary.options_descriptor.as_ref() else {
        return;
    };
    names.extend(descriptor.members.iter().filter_map(|member| {
        matches!(
            member.group,
            OptionGroup::Props
                | OptionGroup::Inject
                | OptionGroup::Computed
                | OptionGroup::Methods
                | OptionGroup::Data
                | OptionGroup::Setup
        )
        .then_some(member.name.as_str())
        .filter(|name| is_safe_value_identifier(name))
        .map(String::from)
    }));
}

pub(super) fn props_source_from_object(source: &str) -> OptionsApiPropsSource {
    OptionsApiPropsSource::deferred_object(String::from(source))
}

/// Extract direct runtime props or the inferred props of a mixin-only export.
pub(super) fn find_options_api_props(script: &str) -> Option<OptionsApiPropsSource> {
    if !script.contains("export default") {
        return None;
    }
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, script, SourceType::ts()).parse();
    if parsed.panicked {
        return None;
    }
    let options = component_options_from_program(&parsed.program)?;
    let direct = option_expression_property(options, "props")
        .and_then(|props| props_from_expression(script, props));
    if option_expression_property(options, "mixins").is_some() {
        return Some(direct.map_or_else(
            OptionsApiPropsSource::default_export,
            OptionsApiPropsSource::with_default_export,
        ));
    }
    direct
}

/// The authored public slot contract from an Options API `slots` option.
///
/// Vue's `SlotsType<T>` is a runtime marker whose useful part is `T`. Keeping
/// that exact type in the component projection preserves member JSDoc and the
/// authored key spelling for hover instead of rebuilding the same shape from
/// the component's `<slot>` outlets.
pub(super) fn find_options_api_slots_type(
    script: &str,
    project_public_slot_functions: bool,
) -> Option<String> {
    if !script.contains("SlotsType") || !script.contains("export default") {
        return None;
    }
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, script, SourceType::ts()).parse();
    if parsed.panicked {
        return None;
    }
    let options = component_options_from_program(&parsed.program)?;
    let slots = option_expression_property(options, "slots")?;
    slots_type_from_expression(script, slots, project_public_slot_functions)
}

fn slots_type_from_expression(
    script: &str,
    expression: &Expression<'_>,
    project_public_slot_functions: bool,
) -> Option<String> {
    match expression {
        Expression::TSAsExpression(value) => slots_type_argument(
            script,
            &value.type_annotation,
            project_public_slot_functions,
        )
        .or_else(|| {
            slots_type_from_expression(script, &value.expression, project_public_slot_functions)
        }),
        Expression::TSSatisfiesExpression(value) => slots_type_argument(
            script,
            &value.type_annotation,
            project_public_slot_functions,
        )
        .or_else(|| {
            slots_type_from_expression(script, &value.expression, project_public_slot_functions)
        }),
        Expression::TSTypeAssertion(value) => slots_type_argument(
            script,
            &value.type_annotation,
            project_public_slot_functions,
        )
        .or_else(|| {
            slots_type_from_expression(script, &value.expression, project_public_slot_functions)
        }),
        Expression::TSNonNullExpression(value) => {
            slots_type_from_expression(script, &value.expression, project_public_slot_functions)
        }
        Expression::ParenthesizedExpression(value) => {
            slots_type_from_expression(script, &value.expression, project_public_slot_functions)
        }
        _ => None,
    }
}

fn slots_type_argument(
    script: &str,
    annotation: &TSType<'_>,
    project_public_slot_functions: bool,
) -> Option<String> {
    let TSType::TSTypeReference(reference) = annotation else {
        return None;
    };
    let TSTypeName::IdentifierReference(name) = &reference.type_name else {
        return None;
    };
    if name.name != "SlotsType" {
        return None;
    }
    let argument = reference.type_arguments.as_ref()?.params.first()?;
    project_public_slot_functions
        .then(|| options_api_slots_contract(script, argument))
        .flatten()
        .or_else(|| source_slice(script, argument.span()).map(String::from))
}

/// Vue's `SlotsType` records the payload object for property-form members,
/// while the public component instance exposes optional slot functions. Emit
/// that public shape directly so TypeScript hover sees both the payload and
/// the authored member documentation.
fn options_api_slots_contract(script: &str, argument: &TSType<'_>) -> Option<String> {
    let TSType::TSTypeLiteral(literal) = argument else {
        return None;
    };
    let mut rendered = String::from("{\n");
    let mut previous_end = literal.span.start.saturating_add(1) as usize;
    for member in &literal.members {
        let (key, value, member_end) = match member {
            TSSignature::TSPropertySignature(property) if !property.computed => {
                let key = slot_member_key(script, &property.key)?;
                let annotation = property.type_annotation.as_ref()?;
                let value = source_slice(script, annotation.type_annotation.span())?;
                let value = if matches!(annotation.type_annotation, TSType::TSFunctionType(_)) {
                    String::from(value)
                } else {
                    vize_carton::cstr!("(props: {value}) => any")
                };
                (key, value, property.span.end as usize)
            }
            TSSignature::TSMethodSignature(method)
                if !method.computed && method.type_parameters.is_none() =>
            {
                let key = slot_member_key(script, &method.key)?;
                let params = source_slice(script, method.params.span())?;
                (
                    key,
                    vize_carton::cstr!("{params} => any"),
                    method.span.end as usize,
                )
            }
            _ => return None,
        };
        let key_start = member.span().start as usize;
        if let Some(doc) = leading_jsdoc(script.get(previous_end..key_start)?) {
            rendered.push_str("  ");
            rendered.push_str(doc);
            rendered.push('\n');
        }
        rendered.push_str("  ");
        rendered.push_str(key.as_str());
        rendered.push_str("?: ");
        rendered.push_str(value.as_str());
        rendered.push_str(";\n");
        previous_end = member_end;
    }
    rendered.push('}');
    Some(rendered)
}

fn slot_member_key(script: &str, key: &PropertyKey<'_>) -> Option<String> {
    match key {
        PropertyKey::StaticIdentifier(identifier) => Some(String::from(identifier.name.as_str())),
        PropertyKey::Identifier(identifier) => Some(String::from(identifier.name.as_str())),
        PropertyKey::StringLiteral(literal) if is_identifier_name(literal.value.as_str()) => {
            Some(String::from(literal.value.as_str()))
        }
        _ => source_slice(script, key.span()).map(String::from),
    }
}

fn leading_jsdoc(preamble: &str) -> Option<&str> {
    let start = preamble.rfind("/**")?;
    let doc = preamble.get(start..)?.trim();
    doc.ends_with("*/").then_some(doc)
}

fn props_from_expression(
    script: &str,
    expression: &Expression<'_>,
) -> Option<OptionsApiPropsSource> {
    match expression {
        Expression::ObjectExpression(object) => Some(props_source_from_object(source_slice(
            script,
            object.span(),
        )?)),
        Expression::ArrayExpression(array) => {
            let names = array
                .elements
                .iter()
                .filter_map(|element| match element {
                    ArrayExpressionElement::StringLiteral(literal) => {
                        Some(String::from(literal.value.as_str()))
                    }
                    _ => None,
                })
                .collect::<Vec<_>>();
            (!names.is_empty()).then(|| OptionsApiPropsSource::names(names))
        }
        Expression::Identifier(identifier) => is_safe_value_identifier(identifier.name.as_str())
            .then(|| {
                OptionsApiPropsSource::deferred_object(String::from(identifier.name.as_str()))
            }),
        Expression::ParenthesizedExpression(value) => {
            props_from_expression(script, &value.expression)
        }
        Expression::TSAsExpression(value) => props_from_expression(script, &value.expression),
        Expression::TSSatisfiesExpression(value) => {
            props_from_expression(script, &value.expression)
        }
        Expression::TSNonNullExpression(value) => props_from_expression(script, &value.expression),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{find_options_api_slots_type, follows_default_keyword};

    #[test]
    fn default_export_boundary_accepts_punctuation_but_not_identifiers() {
        for rest in ["", " ", "{", "(", "/* comment */{"] {
            assert!(follows_default_keyword(rest), "expected boundary: {rest}");
        }
        for rest in ["Thing", "π", "\\u0061"] {
            assert!(
                !follows_default_keyword(rest),
                "identifier continuation is not a boundary: {rest}"
            );
        }
    }

    #[test]
    fn scopes_public_slot_projection_to_editor_generation() {
        let source = r#"
import { defineComponent, type SlotsType } from 'vue'
export default defineComponent({
  slots: Object as SlotsType<{
    /** Row slot. */
    item: { row: number }
  }>,
})
"#;
        assert_eq!(
            find_options_api_slots_type(source, true).as_deref(),
            Some("{\n  /** Row slot. */\n  item?: (props: { row: number }) => any;\n}")
        );
        assert_eq!(
            find_options_api_slots_type(source, false).as_deref(),
            Some("{\n    /** Row slot. */\n    item: { row: number }\n  }")
        );
    }
}
