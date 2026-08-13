//! Props passed from a child component to its own `<slot>` outlets.
//!
//! `<slot>` is a Vue built-in rather than a normal component usage, but
//! `defineSlots` still provides a contextual prop type for outlet bindings.
//! Emitting those bindings as bare template expressions loses callback
//! parameter types and creates Vize-only `TS7006`.

mod collect;
mod emit;

use vize_carton::CompactString;
use vize_croquis::croquis::{PassedProp, SpreadProp};

pub(super) use collect::{collect_slot_outlet_expression_ranges, collect_slot_outlets_by_scope};
pub(super) use emit::{emit_slot_outlet_helpers, generate_scope_slot_outlet_checks};

pub(super) struct SlotOutlet {
    index: usize,
    scope_id: u32,
    name: CompactString,
    name_is_dynamic: bool,
    start: u32,
    vif_guard: Option<CompactString>,
    props: Vec<PassedProp>,
    spread_props: Vec<SpreadProp>,
}
