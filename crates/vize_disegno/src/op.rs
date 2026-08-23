//! The S2 op family: one concrete typed enum per position, never a uniform
//! `Operation` structure (`davinci-road/architecture.md`, "What we take
//! from MLIR, and what we refuse").
//!
//! # Regions are owned by their op
//!
//! A [`Region`] is a child sequence owned by exactly one op. This is what
//! makes fusion tractable: the shipped pipeline merges `v-else` /
//! `v-else-if` siblings onto the **parent's** child list mid-traversal
//! (`crates/vize_atelier_core/src/transform/structural.rs`), and that
//! enter/exit sibling mutation is precisely the re-visit source a
//! region-owning [`IfOp`] never needs - its branches are regions from
//! birth, so no pass ever revisits a child list to stitch them together.
//!
//! # The core is fair, dialect ops are named
//!
//! The neutral core (`ui.*`) is a fair abstraction, not Vue's AST renamed;
//! whatever is genuinely Vue-specific is a `vue.*` dialect op. A dialect
//! op lands with the transform that needs it (P2-9), never speculatively:
//! custom directives as [`BindingOp::VueDirective`], Vue 2 `.sync` /
//! `slot-scope` as [`BindingOp::VueSync`] / [`BindingOp::VueSlotScope`],
//! and Vue 2 pipe filters as [`crate::expr::ExprRef::Filter`].
//!
//! # `Drop`-free by construction
//!
//! Every type here is arena-resident through `vize_carton::{Box, Vec}`,
//! whose const assertion rejects `Drop` payloads (it caught two real
//! violations during P1-10). The `const` assertions below restate the
//! property directly, and the node-size assertions pin each op's 64-bit
//! footprint (guarded on pointer width for the same reason as
//! `crates/vize_relief/src/relief/elements.rs:31-36` - the wasm32 lane
//! P2-14 makes required is 32-bit).

use vize_carton::{Box, Vec};

pub mod bind;
pub mod control;
pub mod element;
pub mod model;
pub mod slot;
pub mod text;
pub mod vue;

pub use bind::{BindOp, OnOp};
pub use control::{ForBinding, ForOp, IfBranch, IfOp};
pub use element::{Attribute, ComponentOp, ElementOp, Namespace};
pub use model::{BindingContract, ModelOp};
pub use slot::{DynamicName, SlotContentOp, SlotOp};
pub use text::{InterpolationOp, TextOp};
pub use vue::{VueDirectiveOp, VueSlotScopeOp, VueSyncOp};

/// One S2 op standing in a region (a child position).
///
/// Attached ops - bindings that belong to one element or component rather
/// than to a child position - live in [`BindingOp`] instead; the two enums
/// together are the closed op family of the stage.
///
/// Payloads are boxed into the arena so the enum stays two words - the
/// shape `vize_relief::TemplateChildNode` already uses for child enums.
#[derive(Debug)]
pub enum Op<'a> {
    /// `ui.element` - a native element with attributes, attached bindings,
    /// and an owned children region.
    Element(Box<'a, ElementOp<'a>>),
    /// `ui.component` - a component reference; same surface as an element,
    /// resolved at lowering.
    Component(Box<'a, ComponentOp<'a>>),
    /// `ui.text` - static text.
    Text(Box<'a, TextOp<'a>>),
    /// `ui.interpolation` - an expression rendered as text.
    Interpolation(Box<'a, InterpolationOp<'a>>),
    /// `ui.if` - structured conditional; every branch owns its region.
    If(Box<'a, IfOp<'a>>),
    /// `ui.for` - structured iteration; the repeated content is one owned
    /// region.
    For(Box<'a, ForOp<'a>>),
    /// `ui.slot` - a slot outlet owning its fallback region.
    Slot(Box<'a, SlotOp<'a>>),
}

impl Op<'_> {
    /// The op's stage-wide mnemonic - the keyword its folio line starts
    /// with.
    #[must_use]
    pub const fn mnemonic(&self) -> &'static str {
        match self {
            Self::Element(_) => "ui.element",
            Self::Component(_) => "ui.component",
            Self::Text(_) => "ui.text",
            Self::Interpolation(_) => "ui.interpolation",
            Self::If(_) => "ui.if",
            Self::For(_) => "ui.for",
            Self::Slot(_) => "ui.slot",
        }
    }
}

/// One S2 op attached to an element or component (a binding position).
///
/// A binding belongs to exactly one owner and never stands in a region -
/// which is why it is a separate enum rather than more [`Op`] variants: the
/// type system rules out a floating `ui.model` instead of a verifier rule.
///
/// The normalized one-way bindings (`ui.bind`, `ui.on`) landed with the
/// P2-9 element/binding-family installment - the transform that needs
/// them - exactly as `ui.slot-content` landed with slot normalization;
/// the exhaustive-match canary made both arrivals loud.
#[derive(Debug)]
pub enum BindingOp<'a> {
    /// `ui.bind` - one one-way binding (`v-bind` / `:` / `.`).
    Bind(Box<'a, BindOp<'a>>),
    /// `ui.on` - one event handler binding (`v-on` / `@`).
    On(Box<'a, OnOp<'a>>),
    /// `ui.model` - the two-way binding contract (never its realization).
    Model(Box<'a, ModelOp<'a>>),
    /// `ui.slot-content` - one authored `v-slot` spelling on its carrier
    /// (the syntactic surface; grouping is the slot pass's fact).
    SlotContent(Box<'a, SlotContentOp<'a>>),
    /// `vue.directive` - a Vue custom directive carried through as a
    /// dialect op.
    VueDirective(Box<'a, VueDirectiveOp<'a>>),
    /// `vue.sync` - Vue 2 `:foo.sync` two-way bind sugar.
    VueSync(Box<'a, VueSyncOp<'a>>),
    /// `vue.slot-scope` - Vue 2 `slot-scope` / `scope` scoped-slot sugar.
    VueSlotScope(Box<'a, VueSlotScopeOp<'a>>),
}

impl BindingOp<'_> {
    /// The op's stage-wide mnemonic - the keyword its folio line starts
    /// with.
    #[must_use]
    pub const fn mnemonic(&self) -> &'static str {
        match self {
            Self::Bind(_) => "ui.bind",
            Self::On(_) => "ui.on",
            Self::Model(_) => "ui.model",
            Self::SlotContent(_) => "ui.slot-content",
            Self::VueDirective(_) => "vue.directive",
            Self::VueSync(_) => "vue.sync",
            Self::VueSlotScope(_) => "vue.slot-scope",
        }
    }
}

/// A child sequence owned by exactly one op.
///
/// Ownership is the point (see the module docs): a region is reachable
/// through its op alone, so no transform ever mutates a sibling list it is
/// currently visiting.
#[derive(Debug)]
pub struct Region<'a> {
    /// The ops of this region, in document order.
    pub ops: Vec<'a, Op<'a>>,
}

/// The op family is `Drop`-free by construction: `vize_carton::{Box, Vec}`
/// already reject `Drop` payloads at their construction sites, and these
/// assertions restate the property on the enums themselves so a violation
/// names this file.
const _: () = assert!(!core::mem::needs_drop::<Op<'static>>());
const _: () = assert!(!core::mem::needs_drop::<BindingOp<'static>>());
const _: () = assert!(!core::mem::needs_drop::<Region<'static>>());

/// Node footprints are pinned per op type (64-bit only: the figures are
/// pointer-dependent and the wasm32-wasip2 lane is 32-bit, the same guard
/// rationale as `crates/vize_relief/src/relief/elements.rs:31-36`). The
/// enums stay two words because every payload is boxed. The figures
/// include the 16-byte [`crate::expr::ExprRef`] per expression position
/// (P2-5b replaced the zero-sized `ExprSlot`, moving every
/// expression-carrying payload's footprint - each is pinned in its own
/// file).
#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<Op<'_>>() == 16);
    assert!(core::mem::size_of::<BindingOp<'_>>() == 16);
    assert!(core::mem::size_of::<Region<'_>>() == 24);
};
