//! HTML content-model facts and the exact nesting checker (Davinci P4-11a).
//!
//! **What is checked.** A template is checked as the HTML it will produce:
//! the statically known element skeleton ([`skeleton`]) is evaluated node by
//! node against its ancestor chain, the checker's model of the HTML parser's
//! stack of open elements. Two families of violation classes
//! ([`ViolationClass`]):
//!
//! - **parser** — the tree-construction algorithm (WHATWG §13.2.6) would not
//!   insert the serialized node under its authored parent: `<div>` closes an
//!   open `<p>`, `<tr>` gets an implied `<tbody>`, table content is
//!   foster-parented, `<span>` breaks out of `<svg>`. The DOM then differs
//!   from the virtual DOM: SSR hydration mismatches, and `innerHTML`-built
//!   static content renders differently.
//! - **content model** — the DOM is built as written but a content model
//!   (§4) forbids it: `<div>` in `<span>`, `<button>` in `<a>`, `<div>` in
//!   `<ul>`.
//!
//! **Data.** Every element set and content model the checker reads comes
//! from `whatwg.tsv`, a projection of a pinned WHATWG snapshot whose rows
//! each name the spec clause they project; the loader rejects unknown,
//! duplicate and missing rows.
//!
//! **Precision tier `exact` within the declared domain.** Every fact is
//! three-valued; a violation is reported only when it is proven in every
//! context consistent with what the template shows, so an unresolved
//! component, a slot, a dynamic binding or the unknown mount point of a
//! component produces `unknown` — silence — never a guess. Within the domain
//! (statically known elements in no-quirks body content, outside native
//! `<template>` contents and scripting-dependent `<noscript>`), the parser
//! verdicts are committed for every pair and triple of the differential
//! universe (`tests/html_content_model_differential.rs`) and must agree
//! exactly with an independent HTML parser — Chromium in CI, parse5 where no
//! browser runs. Only the first divergence on each root-to-node path is
//! reported: below a proven parser divergence the document is already
//! restructured.
//!
//! **Input.** The checker reads the template **as authored**
//! ([`authored_skeleton`]): the default template syntax repairs part of the
//! tree construction while parsing, which would hide exactly the nesting this
//! checker exists to report.

mod build;
mod build_helpers;
mod chain;
mod check;
mod class;
mod composed;
mod composed_tests;
mod content_rules;
mod facts;
mod parser_rules;
mod rows;
mod skeleton;
mod table_rules;
mod tests;
mod tri;

pub use build::{authored_skeleton, skeleton};
pub use chain::{Base, Chain, Frame, NsSet};
pub use check::{Context, Report, Verdict, check, check_with};
pub use class::{Family, ViolationClass};
pub use composed::{ComposedFinding, compose};
pub use facts::{Attr, ElemId, Facts, Ns, WHATWG_TSV, facts};
pub use skeleton::{AttrFacts, BoundaryKind, Element, Node, NodeKind, Skeleton};
pub use tri::Tri;
