//! Retained template-expression parsing (Davinci P1-5).
//!
//! The single parse site for template expressions: when the parser builds a
//! non-static [`SimpleExpressionNode`], its content is parsed once with
//! `oxc_parser` into the compile's shared oxc arena pool and the resulting
//! AST is retained on the node (`js_ast`). Consumers migrate onto the
//! retained AST in P1-6/P1-7; until then the legacy re-parse sites keep
//! running unchanged (`vize_atelier_core::expr_parse_probe` keeps counting
//! them, so P1-6/7 can measure their drop).
//!
//! Contract:
//!
//! - The P0-11 profiler counter `davinci.expr.parses` increments exactly once
//!   per built non-static expression node — parse-at-template-parse, so
//!   counter == distinct expressions (the P1-5 counter law: each expression
//!   parsed at most once, always).
//! - `js_ast` is `Some` iff the node's content parses as one complete
//!   TypeScript-dialect expression covering the whole text, allowing only
//!   trailing whitespace and closed block comments after the parsed expression.
//!   Text a lone
//!   `Expression` cannot represent (v-for values such as `item of items`,
//!   v-on multi-statement bodies such as `a++; b++`, invalid expressions)
//!   stays `None`, and parse failures are swallowed: today every template
//!   expression diagnostic comes from the legacy re-parse sites, and this
//!   site must not add or change any.
//! - `raw` is the exact text the AST was parsed from: the source slice when
//!   the accumulated content equals it (the common, copy-free case),
//!   otherwise an arena copy of the decoded content (attribute values with
//!   entities, camelized same-name shorthand arguments).

use oxc_span::{GetSpan, SourceType};
use vize_relief::{JsExpression, SimpleExpressionNode};
use vize_s0::expression_guard::{expression_is_safe_to_parse, is_expression_trailing_trivia};
use vize_s0::profiler::global_profiler;

use super::Parser;

/// TypeScript expression dialect (superset of the template's JS): matches
/// what the first retained-AST consumers parse today (croquis identifier and
/// v-for helpers use `expr.ts` / `with_typescript(true)`).
const EXPR_SOURCE_TYPE: SourceType = SourceType::ts();

impl<'a> Parser<'a> {
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
) -> Option<JsExpression<'a>> {
    // The guard shared by every oxc entry point (see
    // `vize_s0::expression_guard`): oxc's recursive parser cannot be
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
    Some(JsExpression {
        ast: oxc_allocator.alloc(parsed),
        raw,
    })
}
