use oxc_ast::ast::{TSType, TSTypeAnnotation, TSTypeName};
use oxc_span::GetSpan;
use vize_carton::CompactString;

pub(super) fn prop_type_from_annotation(
    annotation: Option<&TSTypeAnnotation<'_>>,
    source: &str,
) -> Option<CompactString> {
    let span = annotation?.type_annotation.span();
    source
        .get(span.start as usize..span.end as usize)
        .map(str::trim)
        .filter(|ty| !ty.is_empty())
        .map(CompactString::new)
}

pub(super) fn runtime_prop_type_from_ts_type(
    ts_type: &TSType<'_>,
    source: &str,
) -> Option<CompactString> {
    match ts_type {
        TSType::TSTypeReference(type_ref) => {
            let name = simple_type_name(&type_ref.type_name)?;
            matches!(name, "PropType" | "ReadonlyArray")
                .then(|| type_ref.type_arguments.as_ref())
                .flatten()
                .and_then(|args| args.params.first())
                .and_then(|ty| type_source(ty, source))
        }
        TSType::TSParenthesizedType(parenthesized) => {
            runtime_prop_type_from_ts_type(&parenthesized.type_annotation, source)
        }
        _ => None,
    }
}

fn type_source(ts_type: &TSType<'_>, source: &str) -> Option<CompactString> {
    let span = ts_type.span();
    source
        .get(span.start as usize..span.end as usize)
        .map(str::trim)
        .filter(|ty| !ty.is_empty())
        .map(CompactString::new)
}

fn simple_type_name<'a>(type_name: &'a TSTypeName<'_>) -> Option<&'a str> {
    match type_name {
        TSTypeName::IdentifierReference(id) => Some(id.name.as_str()),
        TSTypeName::QualifiedName(qualified) => Some(qualified.right.name.as_str()),
        _ => None,
    }
}
