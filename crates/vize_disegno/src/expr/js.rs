//! [`JsExpr`] - the retained-AST payload, and the S2 load-path parse site.
//!
//! # The admission rule (the P1-5 completeness contract, restated here)
//!
//! A `JsExpr` exists iff its `source` parses as **one complete
//! TS-dialect expression covering the whole text** (only trailing
//! whitespace may remain). The rule is the same one
//! `crates/vize_armature/src/parser/expression.rs` applies at template
//! parse - a retained AST that does not cover its text would lie to every
//! consumer - and [`JsExpr::parse_in`] is its second and last home,
//! with two callers: the **folio load path**, where the owned page's
//! slice + span re-enter an arena (arenas cannot persist - P1-11 - so a
//! folio can only carry text), and the **S1→S2 lowering** (P2-8). The
//! P2-5b text expected lowerings to receive armature's retained ASTs;
//! the S1 that actually landed (P2-7, crate `vize_s1`, codename Sinopia) is token-level and
//! retains no ASTs, so the lowering is where a template expression first
//! parses on the Davinci lane - through this one admission rule, never a
//! private variant of it. The deviation is recorded in the P2-8 record.
//!
//! Unlike armature's site, a parse here does **not** increment the
//! `davinci.expr.parses` profiler counter: the P1-5 counter law
//! (`counter == distinct expressions`, each parsed at most once) is a
//! statement about the shipped compile path. A folio load is tooling
//! re-entry, and the P2-8 lowering is the additive Davinci lane - it
//! runs beside the shipped path, so counting it would double-count any
//! file both paths see. The lane's counter story lands with the fused
//! build path (P2-12b), where the lane *becomes* the compile path.

use oxc_span::{GetSpan, SourceType};
use vize_s0::expression_guard::expression_is_safe_to_parse;
use vize_s0::{Allocator, Span};

use super::opaque::OpaqueReason;

/// TypeScript expression dialect (superset of the template's JS) - the
/// exact `SourceType` the armature retained-parse site uses, so both
/// sites admit the same language.
const EXPR_SOURCE_TYPE: SourceType = SourceType::ts();

/// The retained JS payload behind [`super::ExprRef::Js`].
///
/// The architecture names the variant `Js(&'a Expression<'a>)`; the
/// landed payload carries that AST reference **beside** its exact text
/// and authored span, because the folio contract in the same task
/// requires slice + span at print time and an AST pointer alone cannot
/// recover either (a standalone-parsed expression's oxc spans are
/// text-relative, not file-relative). The deviation is recorded in
/// `davinci-road/plan/phase-2-records/p2-5b.md`.
#[derive(Debug, Clone, Copy)]
pub struct JsExpr<'a> {
    /// The retained AST. Covers [`JsExpr::source`] entirely; its internal
    /// oxc spans are relative to that text, not to the compiled file.
    pub ast: &'a oxc_ast::ast::Expression<'a>,
    /// The exact text `ast` was parsed from (the P1-5 `raw` contract).
    pub source: &'a str,
    /// The authored range of `source` in the compiled file.
    pub span: Span,
}

impl<'a> JsExpr<'a> {
    /// Parse `source` as one complete TS-dialect expression into the
    /// arena, or classify why that is impossible.
    ///
    /// The refusals are the two **text-classifiable** escape classes
    /// (see [`OpaqueReason`] for the position-classified ones):
    ///
    /// - [`OpaqueReason::NestingRefused`] - the shared nesting guard
    ///   (`vize_s0::expression_guard`) refused the text before oxc
    ///   ever saw it, exactly as every other oxc entry point does.
    /// - [`OpaqueReason::ParseRejected`] - oxc rejected the text, or the
    ///   first complete expression does not cover it (`a++; b++` parses
    ///   as `a++` and stops; retaining that would lie).
    pub fn parse_in(
        allocator: &'a Allocator,
        source: &'a str,
        span: Span,
    ) -> Result<&'a Self, OpaqueReason> {
        if !expression_is_safe_to_parse(source) {
            return Err(OpaqueReason::NestingRefused);
        }
        let oxc = allocator.as_oxc();
        let Ok(parsed) = oxc_parser::Parser::new(oxc, source, EXPR_SOURCE_TYPE).parse_expression()
        else {
            return Err(OpaqueReason::ParseRejected);
        };
        let Some(rest) = source.get(parsed.span().end as usize..) else {
            return Err(OpaqueReason::ParseRejected);
        };
        if !rest.trim().is_empty() {
            return Err(OpaqueReason::ParseRejected);
        }
        Ok(allocator.alloc(Self {
            ast: allocator.alloc(parsed),
            source,
            span,
        }))
    }
}

/// See [`crate::op`] for both guard rationales.
const _: () = assert!(!core::mem::needs_drop::<JsExpr<'static>>());
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<JsExpr<'_>>() == 32);
