use vize_carton::CompactString;

/// Slots definition from defineSlots
#[derive(Debug, Clone)]
pub struct SlotsDefinition {
    /// Slot name
    pub name: CompactString,
    /// Slot props type (if known)
    pub props_type: Option<CompactString>,
}

impl super::MacroTracker {
    /// Add a slot and retain its static authored key from the existing AST walk.
    pub fn add_slot_with_declaration(
        &mut self,
        slot: SlotsDefinition,
        declaration: Option<(u32, u32)>,
    ) {
        self.slot_declarations.push(declaration);
        self.slots.push(slot);
    }

    /// The unchanged inner type-argument range from the recognized macro AST.
    pub fn set_slot_type_argument_range(&mut self, start: u32, end: u32) {
        self.slot_type_argument_range = Some((start, end));
    }

    /// Inner type bounds, excluding the existing angle-bracket delimiters.
    pub fn slot_type_argument_range(&self) -> Option<(u32, u32)> {
        self.slot_type_argument_range
    }

    /// Every static written key in source order, including repeated names.
    pub fn static_slot_declarations(&self) -> impl Iterator<Item = (u32, u32)> + '_ {
        self.slot_declarations.iter().flatten().copied()
    }
}
