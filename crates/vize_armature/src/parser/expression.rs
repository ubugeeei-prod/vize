//! Parse-once template expression and slot-parameter roles.
//!
//! The single parse site for template expressions: when the parser builds a
//! non-static [`SimpleExpressionNode`], its content is parsed once with
//! `oxc_parser` into the compile's shared oxc arena pool and the resulting
//! AST is retained on the node (`js_ast`). Consumers migrate onto the
//! retained role. A slot value uses the direct original-source parameter
//! grammar; its argument still uses the ordinary expression grammar.
//!
//! Contract:
//!
//! - `davinci.expr.parses` increments once per guard-admitted non-static
//!   node, whether parsing succeeds or fails; guard refusal never attempts
//!   OXC. Consumers of retained slots never parse the same value again.
//! - Ordinary `js_ast` is `Some` iff content parses as one complete
//!   TypeScript-dialect expression covering the whole text, allowing only
//!   trailing whitespace and closed block comments after the parsed expression.
//!   Text an Expression cannot represent (v-for values such as `item of items`,
//!   v-on multi-statement bodies such as `a++; b++`, invalid expressions)
//!   stays `None`; their existing diagnostic contracts remain unchanged.
//! - Slot `js_ast` retains either complete FormalParameters or all original
//!   diagnostics parked in the compile's owned storage. Guard refusal also
//!   keeps the slot error role, preventing a consumer raw-parser fallback.
//!   Depth refusal preserves its existing quiet diagnostic contract.
//! - `raw` is the exact text the AST was parsed from: the source slice when
//!   the accumulated content equals it (the common, copy-free case),
//!   otherwise an arena copy of the decoded content (attribute values with
//!   entities, camelized same-name shorthand arguments).

use oxc_span::{GetSpan, SourceType};
use vize_l0::expression_guard::{expression_is_safe_to_parse, is_expression_trailing_trivia};
use vize_l0::profiler::global_profiler;
use vize_relief::{RetainedJsAst, RetainedJsAstKind, SimpleExpressionNode};

use super::Parser;

/// TypeScript expression dialect (superset of the template's JS): matches
/// what the first retained-AST consumers parse today (croquis identifier and
/// v-for helpers use `expr.ts` / `with_typescript(true)`).
const EXPR_SOURCE_TYPE: SourceType = SourceType::ts();

impl<'a> Parser<'a> {
    pub(super) fn retain_value_ast(
        &self,
        node: &mut SimpleExpressionNode<'a>,
        start: usize,
        end: usize,
        directive: &str,
    ) {
        if directive == "slot" {
            self.retain_slot_parameters_ast(node, start, end);
        } else {
            self.retain_expression_ast(node, start, end);
        }
    }

    /// Attach the parse-once retained AST to a freshly built non-static
    /// expression node whose content was accumulated from `start..end` of the
    /// template source.
    pub(super) fn retain_expression_ast(
        &self,
        node: &mut SimpleExpressionNode<'a>,
        start: usize,
        end: usize,
    ) {
        debug_assert!(!node.is_static);
        let slice = self.get_source_retained(start, end);
        let raw: &'a str = if slice == node.content {
            slice
        } else {
            self.oxc_allocator.alloc_str(node.content)
        };
        node.js_ast = parse_retained(self.oxc_allocator, raw);
    }

    /// A v-slot value owns a parameter grammar, never an expression cover tree.
    pub(super) fn retain_slot_parameters_ast(
        &self,
        node: &mut SimpleExpressionNode<'a>,
        start: usize,
        end: usize,
    ) {
        let slice = self.get_source_retained(start, end);
        let raw = if slice == node.content {
            slice
        } else {
            self.oxc_allocator.alloc_str(node.content)
        };
        node.js_ast = Some(retain_slot_parameters_in(self.allocator, raw));
    }
}

/// Admit a slot value at its original directive or legacy conversion producer.
/// Plain attributes stay unparsed until the dialect-gated conversion selects
/// their parameter role. Consumers only copy this compile-owned carrier.
#[doc(hidden)]
pub fn retain_slot_parameters_in<'a>(
    allocator: &'a vize_l0::Allocator,
    raw: &'a str,
) -> RetainedJsAst<'a> {
    if !expression_is_safe_to_parse(raw) {
        let overflow = vize_l0::expression_guard::expression_exceeds_max_depth(raw);
        return RetainedJsAst::slot_refusal_in(
            allocator,
            raw,
            (!overflow).then_some("mismatched expression delimiters"),
        );
    }
    global_profiler().record_counter("davinci.expr.parses", 1);
    let parameters =
        oxc_parser::Parser::new(allocator.as_oxc(), raw, EXPR_SOURCE_TYPE).parse_slot_parameters();
    if let Ok(parameters) = &parameters
        && !raw
            .get(parameters.span().end as usize..)
            .is_some_and(is_expression_trailing_trivia)
    {
        return RetainedJsAst::slot_refusal_in(allocator, raw, Some("Unexpected token"));
    }
    let parameters = parameters.map_err(|errors| allocator.alloc_owned(errors));
    RetainedJsAst {
        ast: allocator
            .as_oxc()
            .alloc(RetainedJsAstKind::SlotBindings(parameters)),
        raw,
    }
}

/// The single retained-expression parse site.
///
/// Every oxc parse here is one `davinci.expr.parses` increment, whether or
/// not the text turns out to be a lone complete expression — an attempt is a
/// parse. Text the nesting guard refuses is never handed to oxc at all, so
/// it is not an attempt and not counted — the same refusal every legacy
/// parse site applies before creating its parse arena.
fn parse_retained<'a>(
    oxc_allocator: &'a oxc_allocator::Allocator,
    raw: &'a str,
) -> Option<RetainedJsAst<'a>> {
    // The guard shared by every oxc entry point (see
    // `vize_l0::expression_guard`): oxc's recursive parser cannot be
    // depth-limited, so pathologically nested or unbalanced text (#956,
    // #2944, #3712) must be refused before parsing, here exactly as at the
    // transform/codegen sites.
    if !expression_is_safe_to_parse(raw) {
        return None;
    }
    global_profiler().record_counter("davinci.expr.parses", 1);
    let parsed = oxc_parser::Parser::new(oxc_allocator, raw, EXPR_SOURCE_TYPE)
        .parse_expression()
        .ok()?;
    // `parse_expression` stops after the first complete expression without
    // demanding end-of-input, so `a++; b++` would come back as `a++`. A
    // retained AST that does not cover its `raw` would lie to consumers;
    // only trailing whitespace and closed block comments may remain.
    let rest = raw.get(parsed.span().end as usize..)?;
    if !is_expression_trailing_trivia(rest) {
        return None;
    }
    Some(RetainedJsAst {
        ast: oxc_allocator.alloc(RetainedJsAstKind::Expression(parsed)),
        raw,
    })
}
