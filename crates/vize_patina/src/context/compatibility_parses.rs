//! Parse-once fallback syntax for compatibility template rules.
//!
//! L2 already retains every complete expression. The only missing shapes are
//! handler statement lists and the two independent halves of `v-for`; cache
//! those here so a file with several compatibility rules never reparses them.

use oxc_ast::ast::{Expression, FormalParameters, Program};
use vize_l0::{Allocator, FxHashMap, Span};

type ParseKey = (u32, u32);

#[derive(Clone, Copy)]
pub(crate) struct RetainedVFor<'a> {
    pub(crate) parameters: &'a FormalParameters<'a>,
    pub(crate) parameters_offset: u32,
    pub(crate) source: &'a Expression<'a>,
    pub(crate) source_offset: u32,
}

#[derive(Default)]
pub(crate) struct CompatibilityParses<'a> {
    expressions: FxHashMap<ParseKey, Option<&'a Expression<'a>>>,
    handlers: FxHashMap<ParseKey, Option<&'a Program<'a>>>,
    v_for: FxHashMap<ParseKey, Option<RetainedVFor<'a>>>,
}

impl<'a> CompatibilityParses<'a> {
    pub(crate) fn expression(
        &mut self,
        allocator: &'a Allocator,
        span: Span,
        source: &str,
    ) -> Option<&'a Expression<'a>> {
        *self.expressions.entry(key(span)).or_insert_with(|| {
            let source = allocator.alloc_str(source);
            vize_l2::expr::JsExpr::parse_ast_in(allocator, source)
                .ok()
                .map(|expression| &*allocator.as_oxc().alloc(expression))
        })
    }

    pub(crate) fn handler(
        &mut self,
        allocator: &'a Allocator,
        span: Span,
        source: &str,
    ) -> Option<&'a Program<'a>> {
        *self.handlers.entry(key(span)).or_insert_with(|| {
            let source = allocator.alloc_str(source);
            vize_l2::expr::JsExpr::parse_handler_program_in(allocator, source)
                .ok()
                .map(|program| &*allocator.as_oxc().alloc(program))
        })
    }

    pub(crate) fn v_for(
        &mut self,
        allocator: &'a Allocator,
        span: Span,
        source: &str,
    ) -> Option<RetainedVFor<'a>> {
        *self.v_for.entry(key(span)).or_insert_with(|| {
            let source = allocator.alloc_str(source);
            let (parameters, iterable) =
                vize_croquis::drawer::retain_v_for_binding_parameters_with_source(
                    allocator, source,
                )?;
            let source_expression =
                vize_l2::expr::JsExpr::parse_ast_in(allocator, iterable).ok()?;
            let source_expression = &*allocator.as_oxc().alloc(source_expression);
            Some(RetainedVFor {
                parameters: &*allocator.as_oxc().alloc(parameters.parameters),
                parameters_offset: parameters.offset,
                source: source_expression,
                source_offset: (iterable.as_ptr().addr() - source.as_ptr().addr()) as u32,
            })
        })
    }
}

const fn key(span: Span) -> ParseKey {
    (span.start, span.end)
}
