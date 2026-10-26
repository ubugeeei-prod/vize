//! Compile-arena v-for binding goal for consumers that need original AST spans.
//! The ordinary Expression of a whole `in` value is never a binding authority.

use oxc_ast::ast::FormalParameters;
use oxc_span::{GetSpan, SourceType};
use vize_l0::{
    Allocator,
    expression_guard::{expression_is_safe_to_parse, is_expression_trailing_trivia},
    profiler::global_profiler,
};

/// A complete original alias goal, borrowed from the exact v-for value.
/// AST spans address `raw`; `offset` is its byte offset in that value.
pub struct VForBindingParameters<'a> {
    pub raw: &'a str,
    pub offset: u32,
    pub parameters: FormalParameters<'a>,
}

/// Select the existing first viable Vue `in`/`of` boundary, then retain the
/// original alias parameter goal. No synthesized array/function wrapper or
/// decoded-name source search is involved. Tuple elisions are grammar, not
/// fabricated bindings, and malformed groups never yield partial declarations.
pub fn retain_v_for_binding_parameters<'a>(
    allocator: &'a Allocator,
    value: &'a str,
) -> Option<VForBindingParameters<'a>> {
    retain_v_for_binding_parameters_with_source(allocator, value).map(|(bindings, _)| bindings)
}

/// Retain the same original alias goal together with its exact iterable slice.
/// Both views borrow `value`; entity decoding, if needed, belongs to its owner.
/// No wrapper, second boundary search, or second alias parse is introduced.
pub fn retain_v_for_binding_parameters_with_source<'a>(
    allocator: &'a Allocator,
    value: &'a str,
) -> Option<(VForBindingParameters<'a>, &'a str)> {
    super::parse_first_v_for_candidate(value, |alias, source| {
        let alias = alias.strip_prefix("const ").unwrap_or(alias).trim();
        let raw = if alias.starts_with('(') {
            alias.strip_prefix('(')?.strip_suffix(')')?.trim()
        } else {
            alias
        };
        if !expression_is_safe_to_parse(raw) {
            return None;
        }
        global_profiler().record_counter("davinci.expr.parses", 1);
        let parameters = oxc_parser::Parser::new(allocator.as_oxc(), raw, SourceType::ts())
            .parse_v_for_parameters()
            .ok()?;
        if !raw
            .get(parameters.span().end as usize..)
            .is_some_and(is_expression_trailing_trivia)
        {
            return None;
        }
        Some((
            VForBindingParameters {
                raw,
                offset: (raw.as_ptr() as usize - value.as_ptr() as usize) as u32,
                parameters,
            },
            source,
        ))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iterable_and_aliases_are_borrowed_from_the_same_exact_value() {
        let allocator = Allocator::default();
        for value in [
            " (item, index) of rows.filter(row => row.visible) ",
            "({ a, b, c, d, e, f, g, h, i, j, value: _imports_0 }, key) in rows",
            r"\u005fimports_0 of rows",
            "(, _imports_0, index) of rows",
            "item of '日本語 &amp; encoded'",
        ] {
            let (bindings, source) =
                retain_v_for_binding_parameters_with_source(&allocator, value).unwrap();
            let old = retain_v_for_binding_parameters(&allocator, value).unwrap();
            assert_eq!(bindings.raw, old.raw);
            assert_eq!(bindings.offset, old.offset);
            assert_eq!(bindings.parameters.span(), old.parameters.span());
            let start = source.as_ptr() as usize - value.as_ptr() as usize;
            assert_eq!(value.get(start..start + source.len()), Some(source));
            assert_eq!(source, value.get(start..).unwrap().trim_end());
            let alias_start = bindings.raw.as_ptr() as usize - value.as_ptr() as usize;
            assert_eq!(alias_start, bindings.offset as usize);
        }
    }

    #[test]
    fn invalid_and_guard_refused_aliases_never_return_partial_bindings() {
        let allocator = Allocator::default();
        for value in [
            "item of",
            "{broken of rows",
            "item; injected of rows",
            "&#95;imports_0 of rows",
        ] {
            assert!(
                retain_v_for_binding_parameters_with_source(&allocator, value).is_none(),
                "{value}"
            );
            assert!(
                retain_v_for_binding_parameters(&allocator, value).is_none(),
                "{value}"
            );
        }
        let deep = [
            "[".repeat(33),
            "item".to_owned(),
            "]".repeat(33),
            " of rows".to_owned(),
        ]
        .concat();
        assert!(retain_v_for_binding_parameters_with_source(&allocator, &deep).is_none());
    }
}
