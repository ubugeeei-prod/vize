//! Davinci - the dump/round-trip substrate for the Vize compiler
//! rearchitecture.
//!
//! Named after Leonardo's manuscripts, whose folios carried both the drawing
//! and the notes needed to read it back. This crate currently hosts exactly
//! one module: [`folio`], the textual stage-dump contract (`trait Folio`).
//! Stage IRs land here in later phases; see `davinci-road/architecture.md`.
//!
//! The crate is `no_std + alloc` from birth so every future stage artifact
//! can print and parse on any target (wasm32-wasip2 included). Host-only
//! helpers belong in binaries (`davinci-opt`), never in the library.

#![no_std]

extern crate alloc;

pub mod folio;
