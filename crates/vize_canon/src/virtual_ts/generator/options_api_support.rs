use oxc_allocator::Allocator;
use oxc_ast::ast::{ArrayExpressionElement, Expression, TSType, TSTypeName};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};
use oxc_syntax::identifier::is_identifier_part;
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
pub(super) fn find_options_api_slots_type(script: &str) -> Option<String> {
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
    slots_type_from_expression(script, slots).map(String::from)
}

fn slots_type_from_expression<'a>(script: &'a str, expression: &Expression<'_>) -> Option<&'a str> {
    match expression {
        Expression::TSAsExpression(value) => slots_type_argument(script, &value.type_annotation)
            .or_else(|| slots_type_from_expression(script, &value.expression)),
        Expression::TSSatisfiesExpression(value) => {
            slots_type_argument(script, &value.type_annotation)
                .or_else(|| slots_type_from_expression(script, &value.expression))
        }
        Expression::TSTypeAssertion(value) => slots_type_argument(script, &value.type_annotation)
            .or_else(|| slots_type_from_expression(script, &value.expression)),
        Expression::TSNonNullExpression(value) => {
            slots_type_from_expression(script, &value.expression)
        }
        Expression::ParenthesizedExpression(value) => {
            slots_type_from_expression(script, &value.expression)
        }
        _ => None,
    }
}

fn slots_type_argument<'a>(script: &'a str, annotation: &TSType<'_>) -> Option<&'a str> {
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
    source_slice(script, argument.span())
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
    fn keeps_the_authored_slots_type_and_jsdoc() {
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
            find_options_api_slots_type(source).as_deref(),
            Some("{\n    /** Row slot. */\n    item: { row: number }\n  }")
        );
    }
}
