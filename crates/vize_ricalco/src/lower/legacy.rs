//! The Vue 2 legacy dialect at lowering (P2-9 series 7, `_legacy` only).
//!
//! # The measured dialect-op test (the record's decision)
//!
//! The shipped legacy transform lane (`transforms/legacy.rs` +
//! `legacy_filters.rs`, behind `vize_atelier_core/legacy`) carries four
//! live behaviours, and the port splits them by one measured question —
//! *does the surface have an exact modern equivalent the shipped lane
//! itself rewrites to?*
//!
//! - **`.sync` expansion**, **`slot-scope`/`scope` desugar**, and the
//!   **v-on event sugar** (`.native` strip, numeric keycodes): yes — the
//!   shipped lane desugars each into its Vue 3 form before its transform
//!   machinery runs (`desugar_legacy_template`, pre-traversal;
//!   `desugar_v2_v_on_modifiers`, per directive). The port follows the
//!   living code: the same desugars, mirrored byte-for-byte at lowering
//!   into the ops the family already speaks (`ui.bind`/`ui.on`/
//!   `ui.slot-content`), each rewrite under a `normalize.legacy.*`
//!   provenance record so nothing launders silently.
//! - **Pipe filters**: no — a filter chain has no modern form (the
//!   shipped rewrite *invents* runtime asset calls), and under the
//!   dialect the text is not a JS expression at all (`|` is the filter
//!   separator). Filters are therefore the flagship dialect op: a lone
//!   filter interpolation lowers to `vue.filter`, a filter-bearing
//!   `v-bind` value to [`OpaqueReason::LegacyFilter`], both with the
//!   split recorded beside the tree ([`Lowered::filters`]) — the
//!   Compound producer's pattern exactly. The asset-registration half
//!   (`ctx.filters` → `RootNode::filters` → `_resolveFilter` codegen)
//!   is realization and lands with the stage that emits it.
//!
//! # Scope narrowings, recorded loud
//!
//! The shipped filter rewrite runs inside `process_expression`, which
//! the live lane reaches **only when identifier prefixing (or TS) is
//! on** — and therefore for *every* expression position it prefixes
//! (conditions, v-for sources, v-on handlers, v-model values). The S2
//! split keys on the dialect alone and applies at Vue 2's documented
//! filter positions — mustache interpolations and `v-bind` values (the
//! shipped `legacy_filters.rs` test suite's own coverage). Other
//! positions are deliberately not split; the legacy differential lane
//! counts them (`filters_other_positions`) instead of comparing inside
//! them. A filter interpolation inside a **merged** text run stays the
//! Compound producer's part (counted, `filters_in_compounds`); the
//! `.sync`/`slot-scope` desugars apply to element and component owners
//! (a `<slot>` outlet's legacy sugar has no S2 story, like the outlet's
//! `v-slot` before series 5 closed it). Each class is a recorded gap
//! with the realization/exit-gate owner named in the series record.
//!
//! [`OpaqueReason::LegacyFilter`]: vize_disegno::expr::OpaqueReason::LegacyFilter
//! [`Lowered::filters`]: crate::lower::Lowered::filters

use vize_carton::{Box, Span, String, Vec, cstr};
use vize_sinopia::{Element, Interpolation};

use vize_disegno::expr::{ExprRef, OpaqueReason};
use vize_disegno::op::{Op, VueFilterOp};

use vize_davinci::id::NodeId;

use super::cx::{Cx, attr_span};
use super::expr::{desc, opaque_at, trimmed};

mod filters;
mod sugar;

pub use filters::{FilterParts, FilterSegment, filter_split};
pub(crate) use sugar::{
    consumed_by_scoped_slot, desugar_scoped_slot, desugar_sync, scoped_slot_plan,
};

/// The legacy Vue line the S2 lowering accepts — a documented
/// **mirror** of `vize_armature::legacy::LegacyVueVersion`, not an
/// import: `vize_armature/legacy` forwards to `vize_relief/_legacy`,
/// whose cfg-gated AST fields break workspace-unified builds of crates
/// without their own mirror feature (the `vize_atelier_jsx/legacy`
/// hazard). The mirror is pinned line-for-line against the armature
/// model by the legacy differential witness, and the exit gate deletes
/// the legacy copy — the installment-2 splitter precedent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LegacyVueLine {
    /// Vue 0.10.x.
    V0_10,
    /// The 0.11-era post-rewrite 0.x line.
    V0_11,
    /// Vue 1.x.
    V1,
    /// Vue 2.x, including 2.7.
    V2,
}

/// The resolved per-file legacy mode — the live lane's once-per-file
/// capability resolution, mirrored down to the three fields the S2
/// lowering consumes; hot paths only read fields (the armature
/// zero-cost contract's shape).
#[derive(Debug, Clone, Copy)]
pub(crate) struct LegacyMode {
    /// Pipe filters (`supports_filters`: every legacy line).
    pub filters: bool,
    /// The `.sync` / `slot-scope` sugar (`scoped_slot_attrs`: V2 only —
    /// the shipped desugar's single gate).
    pub scoped_slot_attrs: bool,
    /// The Vue 2 v-on event sugar (`.native`, numeric keycodes) —
    /// version-keyed exactly as the live
    /// `TransformContext::supports_v2_event_sugar` is.
    pub v2_event_sugar: bool,
}

impl LegacyMode {
    /// The default (Vue 3) mode: every capability off — the same
    /// branch-identical short-circuit the live lane guarantees.
    pub(crate) const fn off() -> Self {
        Self {
            filters: false,
            scoped_slot_attrs: false,
            v2_event_sugar: false,
        }
    }

    /// Resolve one legacy line, once per file — the mirror of
    /// `LegacyVueVersion::capabilities` over the consumed fields.
    pub(crate) const fn for_line(line: LegacyVueLine) -> Self {
        Self {
            filters: true,
            scoped_slot_attrs: matches!(line, LegacyVueLine::V2),
            v2_event_sugar: matches!(line, LegacyVueLine::V2),
        }
    }
}

/// The consumed capability view of one line, for the cross-crate
/// mirror pin (`filters`, `scoped_slot_attrs`, `v2_event_sugar`): the
/// legacy differential witness asserts these against
/// `vize_armature::legacy::LegacyVueVersion::capabilities` where both
/// homes are visible.
#[must_use]
pub fn mode_probe(line: LegacyVueLine) -> (bool, bool, bool) {
    let mode = LegacyMode::for_line(line);
    (mode.filters, mode.scoped_slot_attrs, mode.v2_event_sugar)
}

/// Lower a lone filter interpolation to `vue.filter`. Returns `false`
/// (emitting nothing) when the dialect has no filters or the content
/// carries no valid top-level chain — the caller then lowers the
/// ordinary `ui.interpolation`, byte-identical to the default dialect.
pub(crate) fn filter_interpolation<'a>(
    cx: &mut Cx<'a>,
    node: &Interpolation<'a>,
    out: &mut Vec<'a, Op<'a>>,
) -> bool {
    if !cx.legacy.filters {
        return false;
    }
    let (slice, expr_span) = trimmed(cx, node.content.text);
    let Some(parts) = filter_split(slice) else {
        return false;
    };
    let id = cx.mint_op();
    let span = Span::new(cx.offset(node.open.text), cx.token_span(&node.close).end);
    let expression = opaque_at(cx, OpaqueReason::LegacyFilter, slice, expr_span);
    cx.record(
        "lower.vue-filter",
        id,
        node.content.text,
        cstr!(
            "vue.filter {} segments={}",
            desc(&expression),
            parts.segments.len()
        ),
        span,
    );
    cx.attach_filters(id, parts);
    out.push(Op::VueFilter(Box::new_in(
        VueFilterOp { expression, span },
        &cx.allocator,
    )));
    true
}

/// Admit one `v-bind` value under the dialect: a valid top-level filter
/// chain becomes the pessimal [`OpaqueReason::LegacyFilter`] escape with
/// its split recorded against the binding op; anything else returns
/// `None` and the caller admits normally.
pub(crate) fn filter_value<'a>(
    cx: &mut Cx<'a>,
    node: Option<NodeId>,
    text: &'a str,
) -> Option<ExprRef<'a>> {
    if !cx.legacy.filters {
        return None;
    }
    let (slice, span) = trimmed(cx, text);
    let parts = filter_split(slice)?;
    let expression = opaque_at(cx, OpaqueReason::LegacyFilter, slice, span);
    cx.record(
        "lower.filter-value",
        node,
        text,
        cstr!("{} segments={}", desc(&expression), parts.segments.len()),
        span,
    );
    cx.attach_filters(node, parts);
    Some(expression)
}

/// Map a Vue 2 numeric `keyCode` modifier to its Vue 3 key name —
/// mirrored from the shipped `keycode_to_key_name` (Vue 2's `keyNames`
/// defaults; 8 and 46 both read `delete`, as Vue 2 grouped them).
const fn keycode_to_key_name(code: &str) -> Option<&'static str> {
    Some(match code.as_bytes() {
        b"8" => "delete",
        b"9" => "tab",
        b"13" => "enter",
        b"27" => "esc",
        b"32" => "space",
        b"37" => "left",
        b"38" => "up",
        b"39" => "right",
        b"40" => "down",
        b"46" => "delete",
        _ => return None,
    })
}

/// The Vue 2 v-on event sugar, mirrored from the shipped
/// `desugar_v2_v_on_modifiers`: `.native` is stripped wholesale, numeric
/// keycodes rewrite to their key names, everything else is kept in
/// authored order. Each change leaves a `normalize.legacy.*` record.
pub(crate) fn rewrite_on_modifiers<'a>(
    cx: &mut Cx<'a>,
    element: &Element<'a>,
    index: usize,
    modifiers: &mut Vec<'a, &'a str>,
) {
    if !cx.legacy.v2_event_sugar || modifiers.is_empty() {
        return;
    }
    let attr = &element.open.attrs[index];
    let span = attr_span(cx, attr);
    let mut rewritten: Vec<'a, &'a str> = Vec::new_in(&cx.allocator);
    for modifier in modifiers.iter() {
        if *modifier == "native" {
            cx.record(
                "normalize.legacy.native",
                None,
                modifier,
                String::default(),
                span,
            );
            continue;
        }
        if let Some(name) = keycode_to_key_name(modifier) {
            cx.record(
                "normalize.legacy.keycode",
                None,
                modifier,
                String::from(name),
                span,
            );
            rewritten.push(name);
            continue;
        }
        rewritten.push(modifier);
    }
    *modifiers = rewritten;
}
