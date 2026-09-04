//! Cache-slot renumbering.

use alloc::vec::Vec as StdVec;

use vize_s0::{String, ToCompactString};

use super::EmitCx;

/// The shipped codegen allocates a cache slot each time it *prints* one,
/// so in its output the first appearance of `_cache[n]` runs 0, 1, 2, …
/// in text order. This lane renders slot bodies in source order and
/// prints them in the slot object's order, so a body authored before a
/// named `<template #x>` takes its number first and prints second. The
/// recorded sites carry the exact digit positions, so the numbering is
/// re-derived here without moving a byte of anything else.
///
/// The map must cover every allocated slot: a site the emit forgot to
/// record would keep its old number while its neighbours moved, so a
/// short map leaves the output untouched rather than corrupting it.
pub(super) fn renumber(cx: &mut EmitCx<'_>) {
    if cx.cache_sites.is_empty() {
        return;
    }
    cx.cache_sites
        .sort_unstable_by_key(|(offset, order_key, _)| (*order_key, *offset));
    let mut order: StdVec<u32> = StdVec::new();
    for (_, _, slot) in cx.cache_sites.iter() {
        if !order.contains(slot) {
            order.push(*slot);
        }
    }
    if order.len() != cx.once_cache_index as usize {
        return;
    }
    if order.iter().copied().eq(0..cx.once_cache_index) {
        return;
    }
    let renumbered = |slot: u32| -> u32 {
        order
            .iter()
            .position(|seen| *seen == slot)
            .unwrap_or(slot as usize) as u32
    };
    cx.cache_sites
        .sort_unstable_by_key(|(offset, _, _)| *offset);
    let mut out = String::with_capacity(cx.buf.code.len());
    let mut cursor = 0usize;
    for (offset, _, slot) in cx.cache_sites.iter().copied() {
        out.push_str(&cx.buf.code.as_str()[cursor..offset]);
        let width = slot.to_compact_string().len();
        out.push_str(renumbered(slot).to_compact_string().as_str());
        cursor = offset + width;
    }
    out.push_str(&cx.buf.code.as_str()[cursor..]);
    cx.buf.code = out;
}
