//! Owned mirrors of the expression payloads (P2-5b).
//!
//! The folio stores expressions as **owned text plus span**, never as AST
//! references, because arena references cannot persist across a compile
//! (P1-11's contract, enforced by the debug arena-generation stamp in
//! `davinci/vize_l0/src/allocator/generation.rs`): a `js(...)` payload
//! re-parses into the arena on load through
//! [`JsExpr::parse_in`](crate::expr::JsExpr::parse_in) (or totally via
//! [`ExprRef::parse_js_in`](crate::expr::ExprRef::parse_js_in)). The
//! owned mirrors *do* implement `PartialEq` - equality here compares what
//! the folio carries, which is explicit text, not expression semantics;
//! the pessimal no-equality law lives on the arena types.

include!("../../lang/dump/owned.rs");
