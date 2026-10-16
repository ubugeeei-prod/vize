//! Only generated component metadata belongs to the declaration stripper.

use std::ops::Range;

use oxc_ast::ast::{
    BindingPattern, Program, PropertyKey, Statement, TSSignature, TSType, TSTypeLiteral,
    VariableDeclarationKind,
};

pub(super) fn internal_field_ranges(program: &Program<'_>) -> Vec<Range<usize>> {
    let mut ranges = Vec::new();
    for statement in &program.body {
        let Statement::VariableDeclaration(declaration) = statement else {
            continue;
        };
        if !declaration.declare
            || declaration.kind != VariableDeclarationKind::Const
            || declaration.declarations.len() != 1
        {
            continue;
        }
        for declarator in &declaration.declarations {
            let BindingPattern::BindingIdentifier(binding) = &declarator.id else {
                continue;
            };
            if binding.name != "__vize_component__" || declarator.init.is_some() {
                continue;
            }
            let Some(annotation) = &declarator.type_annotation else {
                continue;
            };
            let Some(literal) = component_literal(&annotation.type_annotation) else {
                continue;
            };
            // Do not descend into public property types, interfaces, constructor
            // returns, or the authored branch of an Options API component.
            for member in &literal.members {
                let TSSignature::TSPropertySignature(property) = member else {
                    continue;
                };
                let PropertyKey::StaticIdentifier(key) = &property.key else {
                    continue;
                };
                if property.readonly
                    && !property.computed
                    && matches!(
                        key.name.as_str(),
                        "__vizeComponentMarker"
                            | "__vizeHasFallthroughProps"
                            | "__vizeFallthroughProps"
                    )
                {
                    ranges.push(property.span.start as usize..property.span.end as usize);
                }
            }
        }
    }
    ranges.sort_by_key(|range| range.start);
    ranges
}

fn component_literal<'a, 'b>(kind: &'b TSType<'a>) -> Option<&'b TSTypeLiteral<'a>> {
    match kind {
        TSType::TSTypeLiteral(literal) => Some(literal),
        TSType::TSIntersectionType(intersection) => component_literal(intersection.types.first()?),
        TSType::TSConditionalType(conditional) => component_literal(&conditional.true_type),
        TSType::TSParenthesizedType(parenthesized) => {
            component_literal(&parenthesized.type_annotation)
        }
        _ => None,
    }
}

/// Preserve the established pretty-printed output, without deleting adjacent
/// authored tokens or comments when TypeScript puts two members on one line.
pub(super) fn include_line_whitespace(source: &str, range: Range<usize>) -> Range<usize> {
    let start = source[..range.start]
        .rfind('\n')
        .map_or(0, |offset| offset + 1);
    let end = source[range.end..]
        .find('\n')
        .map_or(source.len(), |offset| range.end + offset + 1);
    if source[start..range.start].trim().is_empty() && source[range.end..end].trim().is_empty() {
        start..end
    } else {
        range
    }
}
