//! Cross-file prop members resolved from an imported macro type.

use oxc_ast::ast::TSType;
use oxc_span::GetSpan;

use super::TypedProp;

/// Append the properties supplied by the cross-file type world, when present.
pub(super) fn append_resolved_properties(
    typed: &mut Vec<TypedProp>,
    source: &str,
    ty: &TSType<'_>,
    offset: usize,
    world: Option<&vize_croquis::types::ResolvedTypeWorld>,
) {
    let Some(world) = world else {
        return;
    };
    let Some(type_source) = source.get(ty.span().start as usize..ty.span().end as usize) else {
        return;
    };

    typed.extend(
        world
            .resolve_properties(type_source)
            .properties
            .into_iter()
            .map(|property| TypedProp {
                name: property.name,
                span: ty.span(),
                key_span: ty.span(),
                required: !property.optional,
                boolean: property.prop_type.as_deref().is_some_and(|ty| {
                    matches!(ty.trim(), "boolean" | "Boolean" | "true" | "false")
                }),
                offset,
            }),
    );
}
