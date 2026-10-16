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
    super::parse_first_v_for_candidate(value, |alias, _| {
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
        Some(VForBindingParameters {
            raw,
            offset: (raw.as_ptr() as usize - value.as_ptr() as usize) as u32,
            parameters,
        })
    })
}
