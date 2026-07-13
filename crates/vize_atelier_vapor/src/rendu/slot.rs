//! Owned Vapor projection of Rendu's component slot plan.

use vize_rendu::RenduProvenance;

use super::{VaporBinding, VaporBlockId, VaporExpressionId, VaporName};

/// One slot function and its runtime name.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct VaporSlot {
    pub name: VaporName,
    pub bindings: Vec<VaporBinding>,
    pub body: VaporBlockId,
    pub provenance: RenduProvenance,
}

/// One branch of a conditionally available slot.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct VaporConditionalSlotBranch {
    pub condition: Option<VaporExpressionId>,
    pub slot: Option<VaporSlot>,
    pub provenance: RenduProvenance,
}

/// A reactive slot source consumed by Vue's Vapor slot normalizer.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum VaporDynamicSlot {
    Direct(VaporSlot),
    Conditional {
        branches: Vec<VaporConditionalSlotBranch>,
        provenance: RenduProvenance,
    },
    Iterated {
        source: VaporExpressionId,
        value: VaporBinding,
        key: Option<VaporBinding>,
        index: Option<VaporBinding>,
        slot: Box<VaporSlot>,
        provenance: RenduProvenance,
    },
}

/// Component children classified once through [`vize_rendu::RenduSlotPlan`].
#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct VaporComponentSlots {
    pub default: Option<VaporBlockId>,
    pub static_slots: Vec<VaporSlot>,
    pub dynamic_slots: Vec<VaporDynamicSlot>,
}
