use oxc_allocator::Allocator;
use oxc_ast::ast::{ArrayExpressionElement, Expression};
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

pub(super) fn extend_options_api_descriptor_names<'a>(
    names: &mut Vec<&'a str>,
    summary: &'a Croquis,
) {
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
    use super::follows_default_keyword;

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
}
