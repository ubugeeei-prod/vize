//! Explicit placement alternatives for S3 ops (P3-10).
//!
//! Impeto keeps the choice of *where* an op's work runs as data on the op
//! instead of deciding it while emitting, the aegraph discipline recorded in
//! `docs/davinci/prior-art-toolchains.md`. The canonical program runs every op
//! [`Placement::Inline`]; [`annotate`] records the other semantics-preserving
//! shapes an op may take:
//!
//! - [`Placement::Hoist`]: an element whose whole subtree is literal, owned by
//!   a control region (`if`, `for`, slot outlet, or component content), and
//!   free of instance-bound attributes (`ref`, `key`, `is`).
//! - [`Placement::Cache`]: a dynamic `set-event` whose handler is one plain JS
//!   expression under a static event name, with no `for`, slot, or component
//!   scope between it and the root.
//! - [`Placement::Group`]: a dynamic leaf update (`set-prop`,
//!   `set-dynamic-props`, `set-text`, `set-html`) reading one direct reference
//!   (`name` or `name.member`) that its keyed effect-order predecessor reads
//!   too, inside the same root, `if` branch, or `for` item. The group joins
//!   the effect unit of the first op in that contiguous run.
//!
//! Placements are an overlay: ops, regions, edges, effect scopes, and operands
//! stay canonical, so exported partition facts keep describing canonical S3.
//! The TS-27 verifier (`S3V010`) re-derives every recorded alternative and
//! every committed choice; `S3PlacementFolio` is the companion Folio page.

mod annotate;
pub(crate) mod facts;
pub(crate) mod folio;
mod kind;
mod record;

pub use annotate::annotate;
pub use folio::{FolioPlacement, S3PlacementFolio};
pub use kind::{Placement, PlacementSet};
pub use record::PlacementRecord;

use vize_davinci::pass::{Fusability, PassDesc, PassKind, Preserved};

/// [`annotate`] as a pass: optional, whole-program, and graph-preserving.
pub const ANNOTATE: PassDesc = PassDesc::new(
    "annotate-placements",
    PassKind::Optional,
    Fusability::Barrier,
    Preserved::ALL,
);

const _: () = assert!(!core::mem::needs_drop::<PlacementRecord>());
const _: () = assert!(core::mem::size_of::<Placement>() == 1);
const _: () = assert!(core::mem::size_of::<PlacementSet>() == 1);
const _: () = assert!(core::mem::size_of::<PlacementRecord>() <= 16);
