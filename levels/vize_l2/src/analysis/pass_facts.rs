//! The facts L2 passes publish beside the canonical artifact.
//!
//! The artifact's own side tables (compound text, `ui.for` / `ui.if` facts,
//! fragment carriers) are read from the artifact itself; this product holds
//! only what a pass adds. One field per fact family, so a later pass's facts
//! land as new fields rather than a second bag type.

use alloc::vec::Vec as StdVec;
use vize_l0::String;
use vize_l0::side_table::SideTable;

use super::model_facts::ModelFacts;
use super::slot_facts::SlotFacts;
use super::static_facts::StaticFacts;
use super::template_complexity::ComplexityFacts;

/// Runtime-resolved filter assets a legalization introduced.
///
/// Filter legalization rewrites each pipe into a call on the filter's
/// asset local; emission declares one resolver per name.
#[derive(Debug, Default)]
pub struct FilterAssets {
    /// Filter names, first-seen order.
    pub names: StdVec<String>,
    /// A component's own binding surface referenced a filter, which ranks
    /// the filter resolver helper before the component resolver while
    /// keeping component constants first.
    pub resolve_before_components: bool,
}

/// The facts the L2 pass pipeline produces beside the artifact.
#[derive(Debug, Default)]
pub struct L2Facts {
    /// Filter assets from legalization; empty when no input spelled one.
    pub filters: FilterAssets,
    /// Per-`ui.component` canonical slot grouping, keyed by the op's
    /// page-order id.
    pub slot_facts: SideTable<SlotFacts>,
    /// Per-`ui.model` validation faults, keyed by the binding op's
    /// page-order id; sparse — entries only for faulted models.
    pub model_faults: SideTable<ModelFacts>,
    /// Per-owner static-analysis facts, keyed by the `ui.element` /
    /// `ui.component` op's page-order id; dense over the owner family. The
    /// pipeline's first `Optional` product: skipping the pass loses these
    /// and nothing else.
    pub static_facts: SideTable<StaticFacts>,
    /// The component's own template complexity; `None` when the plan
    /// declined the `Optional` analysis.
    pub complexity: Option<ComplexityFacts>,
}
