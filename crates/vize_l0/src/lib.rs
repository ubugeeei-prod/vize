//! L0 — foundation (codename: Carton, absorbing the Davinci substrate).
//!
//! **Experimental:** a compiling skeleton. The public API is laid down first
//! and filled by moving code in from `vize_davinci` (#6833) and `vize_carton`
//! (#6834). Unimplemented bodies are `todo!()` inside modules that carry
//! `#![expect(clippy::todo, reason = "skeleton: #NNNN")]`; no product path
//! reaches them, and a CI ratchet only lets their count go down.
//!
//! Levels are type boundaries, not runtime boundaries: every artifact shares
//! one arena and dense `u32` ids, and nothing is serialized between levels.
//! Dumps exist for `vize dump` and observers only.
//!
//! - [`span`], [`source`], [`arena`] — source text, byte spans and storage.
//! - [`id`] — [`NodeId`](id::NodeId) and [`AnalysisId`](id::AnalysisId).
//! - [`side_table`] — analysis results stored beside a tree, keyed by id.
//! - [`key`] — span-relative content keys for cached artifacts.
//! - [`dump`] — the textual dump contract ([`Dump`](dump::Dump),
//!   [`DumpValue`](dump::DumpValue)) and the per-pass dump runtime.
//! - [`diag`] — diagnostics and their witness chains.
//! - [`pass`] — the pass manager and its observers (remarks, fusion, timing).
//! - [`fact`] — fact groups and the fact manager.
//! - [`level`] — the level registry: L0–L4 and the conversions between them.
//! - [`extension`] — placeholder for the neutral extension wire.

#![no_std]

extern crate alloc;

pub mod arena;
pub mod diag;
pub mod dump;
pub mod extension;
pub mod fact;
pub mod id;
pub mod key;
pub mod level;
pub mod pass;
pub mod side_table;
pub mod source;
pub mod span;
