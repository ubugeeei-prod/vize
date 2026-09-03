//! `#[derive(Folio)]` - the mechanical half of the Davinci stage-dump
//! contract.
//!
//! **Experimental:** generated format and macro API may change in any alpha
//! release; record intentional breaking changes in the release notes.
//!
//! The derive generates `vize_davinci::folio::Folio`'s exact shape -
//! `print(&self, w, mode)` and `parse(input) -> Result<Self, FolioError>` -
//! for an owned document struct, following the normalization rules written
//! in `davinci-road/plan/folio-format.md` ("Derived pages"): stable field
//! order from the type shape, fixed section order, sorted map iteration,
//! empty sections omitted, LF line endings, 1-based error line numbers.
//!
//! **Mechanical trio only.** The derive owns print, parse and field order;
//! anything carrying a semantic decision - what `Display` elides, what a
//! section means - stays hand-written. A derived page therefore prints the
//! same canonical text in both modes: eliding is a decision, and the derive
//! refuses to make decisions. `CroquisFolio` keeps its hand impl for exactly
//! this reason (its grammar is nothing but decisions).
//!
//! # The generated page
//!
//! For a struct `BudgetObserver` the page is headed `[budget-observer]`
//! (kebab-case of the type name). Scalar fields print as `name=value` lines
//! in declaration order inside the header section; `Vec<T>` fields print as
//! `[page.field]` sections of one entry per line, order preserved; and
//! `FxHashMap<K, V>` fields print as `[page.field]` sections of `key=value`
//! lines sorted by printed key (byte order). Field values go through
//! `vize_davinci::folio::value::FolioValue`, so an unsupported field type is
//! a missing-impl compile error in the deriving crate, not a silent format.
//!
//! # Host build dependency
//!
//! This crate is a proc-macro: it runs on the **host** at build time and is
//! never linked into any target, so it is `std` by nature. The code it
//! *generates* runs inside `no_std + alloc` crates and therefore names only
//! `::core` and `vize_davinci` support paths - never `::std` or `::alloc`.
//! This is the approved `std` host-build edge P2-14's boundary audit records.

// The house string discipline (clippy.toml: no std String / format! /
// to_string) exists for the compiler's hot paths. A proc-macro crate runs
// on the host at build time, is never linked into any target, and speaks
// syn/quote's std-String API - pulling vize_carton into a build dependency
// to satisfy the lint would invert the point of the rule.
#![allow(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros
)]

mod codegen;
mod model;

use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

/// Derive `vize_davinci::folio::Folio` for an owned document struct.
///
/// See the crate docs for the generated page format, and
/// `davinci-road/plan/folio-format.md` for the normalization contract the
/// generated pair upholds (TS-16: `print(parse(t)) == t` byte-exact in
/// `Full` mode, `parse(print(v)) == v` structurally).
#[proc_macro_derive(Folio)]
pub fn derive_folio(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match model::PageModel::from_input(&input) {
        Ok(model) => codegen::expand(&model).into(),
        Err(error) => error.to_compile_error().into(),
    }
}
