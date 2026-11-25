//! Creusot proves the fast-path gate of `escape_js_string` on its shipped
//! source (`escape_gate.rs`). This file is the library root of
//! `tests/formal/creusot/js-escape-gate` and is not part of
//! `vize_atelier_core`'s module tree: the module below is the one the product
//! compiles, found by ordinary module discovery.
//!
//! The gate lets a string skip both slow stages when none of its bytes is
//! flagged. That is sound only if every scalar a slow stage rewrites puts a
//! flagged byte into the UTF-8 text, for every one of the 1,112,064 scalars.
//! The shipped contract states it for the first byte of each encoding.
//! See docs/davinci/decisions/2026-11-17-creusot.md.

mod escape_gate;

use creusot_std::prelude::*;
use escape_gate::{byte_may_need_js_escaping, rewritten, scalar, utf8_lead};

/// The first byte of every scalar's encoding is a UTF-8 lead byte: below
/// 0x80, or from 0xC2 to 0xF4.
#[logic]
#[ensures(forall<c: Int> scalar(c) ==> 0 <= utf8_lead(c) && utf8_lead(c) < 0x80
    || 0xC2 <= utf8_lead(c) && utf8_lead(c) <= 0xF4)]
pub fn every_scalar_starts_with_a_lead_byte() {}

/// No scalar that `decode_html_entities` or `escape_js_string` rewrites can
/// start with a byte the gate lets through, so a string the gate passes is
/// returned unchanged by both stages.
#[ensures(forall<c: Int> rewritten(c) && utf8_lead(c) == b@ ==> result)]
pub fn no_rewritten_scalar_passes_the_gate(b: u8) -> bool {
    byte_may_need_js_escaping(b)
}
