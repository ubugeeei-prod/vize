//! Reading a component function's signature.
//!
//! The stateful module renderer rebuilds a block-body component as
//! `_defineComponent({ …, setup(…) { … } })`, so everything the authored
//! signature carried — parameters, type parameters, and the prop names a
//! destructuring pattern names — has to survive the rewrite. These helpers
//! recover exactly that, as byte ranges into the original source (re-emitted
//! verbatim) or as plain names.

use oxc_ast::ast::{BindingPattern, FormalParameters, PropertyKey, TSTypeParameterDeclaration};
use oxc_span::GetSpan;
use vize_s0::String;

/// Byte range covering the authored formal parameter list, parentheses excluded.
///
/// The range spans from the first parameter to the last one (or to the rest
/// element when present) so the original text — type annotations, defaults, and
/// any interleaved comments — is preserved verbatim. An empty parameter list
/// collapses to an empty range.
pub(super) fn formal_parameters_range(params: &FormalParameters<'_>) -> (u32, u32) {
    let start = params
        .items
        .first()
        .map(GetSpan::span)
        .or_else(|| params.rest.as_deref().map(GetSpan::span));
    let end = params
        .rest
        .as_deref()
        .map(GetSpan::span)
        .or_else(|| params.items.last().map(GetSpan::span));
    match (start, end) {
        (Some(start), Some(end)) => (start.start, end.end),
        _ => (0, 0),
    }
}

/// Byte range covering the authored type parameter list, angle brackets excluded.
///
/// A generic component (`const List = <T,>(props: Props<T>) => { … }`) annotates
/// its parameters with type names that are only bound by this declaration, so the
/// range is re-emitted on the generated `setup<T>()` method. A non-generic
/// component collapses to an empty range.
pub(super) fn type_parameters_range(
    type_parameters: Option<&TSTypeParameterDeclaration<'_>>,
) -> (u32, u32) {
    let Some(declaration) = type_parameters else {
        return (0, 0);
    };
    let start = declaration.params.first().map(GetSpan::span);
    let end = declaration.params.last().map(GetSpan::span);
    match (start, end) {
        (Some(start), Some(end)) => (start.start, end.end),
        _ => (0, 0),
    }
}

/// Prop names declared by the first parameter's object destructuring pattern, in
/// source order.
///
/// Only a pattern whose every name is statically known is usable: the result
/// becomes the wrapper's `props` option, and declaring a partial list would route
/// the remaining props to `attrs` while looking authoritative. A rest element or
/// a computed key therefore yields no names at all rather than a subset, and a
/// plain `props` parameter carries no names to begin with.
pub(super) fn destructured_prop_names(params: &FormalParameters<'_>) -> std::vec::Vec<String> {
    let mut names = std::vec::Vec::new();
    let Some(BindingPattern::ObjectPattern(object)) =
        params.items.first().map(|param| &param.pattern)
    else {
        return names;
    };
    if object.rest.is_some() {
        return names;
    }
    for property in object.properties.iter() {
        let PropertyKey::StaticIdentifier(key) = &property.key else {
            return std::vec::Vec::new();
        };
        names.push(String::from(key.name.as_str()));
    }
    names
}
