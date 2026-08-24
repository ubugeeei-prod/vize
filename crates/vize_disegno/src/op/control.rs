//! Structured control flow: `ui.if` and `ui.for`.
//!
//! Control flow is regions, not directive attributes: `v-if`, JSX
//! `<Show>`-style patterns and pug conditionals normalize to the same two
//! ops, and each op owns the content it controls (see [`crate::op`] for
//! why ownership is the fusion-tractability point).

use vize_s0::{Span, Vec};

use super::Region;
use crate::expr::ExprRef;

/// `ui.if` - structured conditional.
///
/// The folio models the structure and the S2 verifier ([`crate::verify`])
/// owns the invariants (at least one branch, at most one trailing
/// unconditional branch); the type deliberately encodes neither, matching
/// the "models the dump, not the analysis" folio rule.
#[derive(Debug)]
pub struct IfOp<'a> {
    /// The branches, in authored order.
    pub branches: Vec<'a, IfBranch<'a>>,
    /// The whole conditional's source range, every branch included.
    pub span: Span,
}

/// One branch of a [`IfOp`], owning its region.
#[derive(Debug)]
pub struct IfBranch<'a> {
    /// The branch condition; `None` for the unconditional (`else`) branch.
    pub condition: Option<ExprRef<'a>>,
    /// The content this branch renders.
    pub region: Region<'a>,
    /// The branch's source range.
    pub span: Span,
}

/// `ui.for` - structured iteration over one owned region.
#[derive(Debug)]
pub struct ForOp<'a> {
    /// What is iterated and what each iteration binds.
    pub binding: ForBinding<'a>,
    /// The repeated content.
    pub region: Region<'a>,
    /// The whole iteration's source range.
    pub span: Span,
}

/// The iteration binding of a [`ForOp`]: the source and the per-iteration
/// binding positions.
///
/// The value/key/index split mirrors the authored grammar
/// (`(value, key, index) in source`). These positions are where the
/// P2-5b escape classes bite first: the authored v-for value is
/// [`OpaqueReason::ForValue`](crate::expr::OpaqueReason::ForValue) text
/// (Vue's `in`/`of` split grammar disagrees with JS `in` on
/// `a in b in c`), and the positions here hold what the splitter
/// produced - each an [`ExprRef`] that is `Js` when its text is admitted
/// and `Opaque` otherwise. P2-8's lowering assigns the classes; the type
/// only makes them representable.
#[derive(Debug, Clone, Copy)]
pub struct ForBinding<'a> {
    /// The iterated collection, object, or range.
    pub source: ExprRef<'a>,
    /// The per-iteration value binding.
    pub value: ExprRef<'a>,
    /// The second binding position (object key), when authored.
    pub key: Option<ExprRef<'a>>,
    /// The third binding position (index), when authored.
    pub index: Option<ExprRef<'a>>,
}

/// See [`crate::op`] for the guard rationale.
#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<IfOp<'_>>() == 32);
    assert!(core::mem::size_of::<IfBranch<'_>>() == 48);
    assert!(core::mem::size_of::<ForOp<'_>>() == 96);
    assert!(core::mem::size_of::<ForBinding<'_>>() == 64);
};
