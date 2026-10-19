//! P2-11 named / scoped slot witness: `<template #name>` groups,
//! component-root `v-slot` (bare defaults, named keys preserved), dynamic
//! names, simple scoped params, and `createSlots` (`v-if` / `v-for`
//! slot templates), compared **byte-for-byte** including helper usage.

#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    reason = "insta and fixtures use format!; fixtures use std strings"
)]

use crate::support;

use vize_l0::Allocator;
use vize_l1_to_l2::emit_dom_source;

mod battery;
mod patch_sites;
mod unsupported;
