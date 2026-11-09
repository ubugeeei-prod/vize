//! Retained-expression consumption gates (Levels P1-7).
//!
//! P1-5 parses every template expression once, at template parse, into the
//! compile's shared oxc arena (`SimpleExpressionNode::js_ast`, TS dialect,
//! `SourceType::ts()`). The atelier transform/codegen sites migrated in P1-7
//! read that retained AST instead of re-parsing — but only where equality
//! with the legacy re-parse is a parser-determinism fact:
//!
//! - [`retained_whole_expression`] gates on `raw == content`: the retained
//!   AST must describe the node's *current* bytes. Any transform that
//!   rewrote `content` after parse (prefixing, filters, TS stripping) fails
//!   the gate and the site falls back to its legacy parse.
//! - [`js_module_compatible`] delegates the dialect decision to the single
//!   level-owned gate in `vize_l1_to_l2`: the legacy sites parse with
//!   `SourceType::default().with_module(true)` (plain JS, module goal) while
//!   the retained parse is TS with the unambiguous module goal. The scan
//!   accepts an AST only when no construct is present whose parse *could*
//!   differ between those dialects — every TS-flavored node or field, every
//!   strict/module-reserved word used as an identifier, sloppy-only literal
//!   forms (legacy octal, `\digit` string escapes), HTML-like comment
//!   markers, `with`, `delete <ident>`, and whole `function`/`class`
//!   expressions (conservatively: their bodies re-open the strictness
//!   surface). For an accepted AST, the JS-module parse of the same bytes is
//!   the same parse, so consuming the retained AST is byte-equivalent to the
//!   legacy re-parse it replaces. Everything rejected keeps the legacy
//!   parse — fallback classes are recorded in `plan/phase-1.md` P1-7.
//!
//! Sites whose legacy dialect already *is* TS (the vapor generate resolver)
//! need only the module-goal half of the scan; they reuse the same predicate,
//! deliberately over-conservative there (TS-syntax expressions stay on the
//! vapor fallback parse — recorded).
//!
//! Under `cfg(any(test, feature = "legacy-differential"))` every gated
//! consumption is dual-run against the legacy parse it replaced and
//! divergence panics — the P1-6 differential-lane pattern, counters in
//! [`differential`]. The dual-run legacy parses use their own uncounted
//! arenas (`oxc_allocator::Allocator::default()`, not
//! [`crate::expr_parse_probe::parse_arena`]): they exist only while the lane
//! is armed and must not disturb the production re-parse floor the P1-7
//! acceptance pins.
//!
//! Temporary like the probe: P1-9 already drives the transform prefixer
//! from the retained AST on admitted inputs (span splicing, guard re-scans
//! skipped); the gates and the remaining legacy chains collapse together
//! once the recorded residual classes shrink, and this module shrinks with
//! them.

use vize_relief::{JsExpression, SimpleExpressionNode};

/// The node's retained AST, iff it still describes the node's exact bytes.
///
/// `raw` is the text the AST was parsed from (P1-5: equal to the content at
/// parse time), so `raw == content` means no transform rewrote the content
/// since — the self-validating staleness gate.
/// Slot-binding parameter parses do not provide an ordinary expression view.
#[inline]
pub fn retained_whole_expression<'a>(node: &SimpleExpressionNode<'a>) -> Option<JsExpression<'a>> {
    let js = node.js_ast?.as_expression()?;
    (js.raw == node.content).then_some(js)
}

/// True when the retained TS/unambiguous parse of `js.raw` is provably the
/// same parse a `SourceType::default().with_module(true)` (JS, module goal)
/// site would produce — see the module docs for the exact reject classes.
pub fn js_module_compatible(js: &JsExpression<'_>) -> bool {
    vize_l1_to_l2::js_module_compatible(js.ast, js.raw)
}

/// Dual-run instrumentation for the differential lane (see module docs);
/// P1-9 adds the transform-lane legacy classification counters there.
pub mod differential;

#[cfg(test)]
mod tests;
