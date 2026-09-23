//! TS-34 — declarative specifications and naive evaluators for the Croquis
//! fact groups (the Polonius discipline).
//!
//! Every migrated group states what it computes as **rules over input
//! relations**, written as data, and ships a naive evaluator that applies
//! those rules with no caching, no streaming and no early exit. The
//! production population pass (the tracker code) is the optimized
//! implementation; TS-34 runs both over the same artifact and demands exact
//! equality of the group's table.
//!
//! | group | input relations | rules |
//! | ----- | --------------- | ----- |
//! | `Bindings` | top-level declarations of `<script setup>`, read by an independent parse ([`bindings_extract`]); prop names of the macro tracker | [`bindings::DECLARATION_RULES`], [`bindings::MACRO_RULES`], [`bindings::REACTIVITY_RULES`] |
//! | `UndefinedRefs` | every template expression the drawer checks, with its enclosing scope ([`trace`]); the final `Bindings` table; the builtin tables | [`undefined_refs::evaluate`] |
//! | `Reactivity` | lattice origin, effect set, escape and verdict; `ReactiveKind` | [`reactivity::join_matrix`] |
//!
//! A spec states its **scope**: artifacts outside it are skipped with a
//! named, counted reason ([`agreement::Agreement`]), and a run that compares
//! nothing fails instead of passing.

pub mod agreement;
pub mod bindings;
pub mod bindings_extract;
pub mod reactivity;
pub mod trace;
pub mod undefined_refs;
