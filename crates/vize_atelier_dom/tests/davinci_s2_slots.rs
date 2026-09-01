//! P2-11 named / scoped slot witness: `<template #name>` groups,
//! component-root `v-slot` (bare defaults, named keys preserved), dynamic
//! names, simple scoped params, and `createSlots` (`v-if` / `v-for`
//! slot templates), compared **byte-for-byte** including helper usage.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
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
