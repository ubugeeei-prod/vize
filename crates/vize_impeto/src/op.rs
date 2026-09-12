//! The Impeto op family: flat, id-addressed operations plus explicit state
//! edges.
//!
//! S3 deliberately does not reuse Vapor's nested block IR. Regions name
//! containment, effects name update scopes, and state edges name ordering.
//! Later lowering can therefore ask graph questions without trusting an
//! incidental traversal order.

mod edge;
mod id;
mod kind;
mod phase;
mod program;
mod region;

pub use edge::{EdgeKind, StateEdge};
pub use id::{EffectId, OpId, RegionId};
pub use kind::OpKind;
pub use phase::Phase;
pub use program::{EffectScope, Op, Program};
pub use region::Region;

const _: () = assert!(!core::mem::needs_drop::<Op>());
const _: () = assert!(!core::mem::needs_drop::<Region>());
const _: () = assert!(!core::mem::needs_drop::<StateEdge>());
const _: () = assert!(!core::mem::needs_drop::<EffectScope>());
const _: () = assert!(!core::mem::needs_drop::<Program<'static>>());

#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<Op>() <= 32);
    assert!(core::mem::size_of::<Region>() <= 32);
    assert!(core::mem::size_of::<StateEdge>() <= 24);
    assert!(core::mem::size_of::<EffectScope>() <= 24);
};
