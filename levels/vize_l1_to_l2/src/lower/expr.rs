//! Expression admission for the lowering: every expression position goes
//! through P2-5b's total [`ExprRef`] contract.
//!
//! The lowering **never re-derives the admission rule**: text goes
//! through [`ExprRef::parse_js_in`] and comes back `Js` when admitted or
//! `Opaque` with the text-classified reason when not. Vue 2 pipe filters
//! remain the total exception. Physical ownership lives under `lang/` while
//! this ordinary child module preserves the existing internal identity.

mod coordinates;

include!("../lang/expression.rs");
