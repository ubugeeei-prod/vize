//! P2-11 named / scoped slot witness: `<template #name>` groups,
//! component-root `v-slot` (bare defaults, named keys preserved), dynamic
//! names, simple scoped params, and `createSlots` (`v-if` / `v-for`
//! slot templates), compared **byte-for-byte** including helper usage.

#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    reason = "insta and fixtures use format!; fixtures use std strings"
)]

#[expect(
    dead_code,
    clippy::disallowed_types,
    clippy::string_slice,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "shared test support; each binary uses a subset"
)]
mod support;

use vize_s0::Allocator;
use vize_s1_to_s2::emit_dom_source;

mod davinci_s2_slots {
    use super::*;

    mod battery;
    mod patch_sites;
    mod unsupported;
}
