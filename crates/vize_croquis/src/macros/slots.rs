use vize_carton::CompactString;

/// Slots definition from defineSlots
#[derive(Debug, Clone)]
pub struct SlotsDefinition {
    /// Slot name
    pub name: CompactString,
    /// Slot props type (if known)
    pub props_type: Option<CompactString>,
}

