//! For-loop and slot scope entries the Vapor generate context stacks.

use vize_carton::{String, cstr};

use super::GenerateContext;
use crate::generate::destructure::parse_destructure_names;

impl GenerateContext<'_> {
    /// Push a slot scope for scoped slots. Returns the slot props variable name.
    pub(crate) fn push_slot_scope(&mut self, destructure_pattern: &str) -> String {
        // A plain identifier names the whole props object, as upstream emits it:
        // `v-slot="p"` becomes `(p) => ... p.x`, not a read through `_slotProps`.
        let pattern = destructure_pattern.trim();
        if pattern.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_' || c == '$')
            && pattern
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$')
        {
            self.slot_scopes.push(SlotScope {
                names: std::vec![String::from(pattern)],
                slot_props_var: String::from(pattern),
                for_depth: self.for_scopes.len(),
                whole: true,
            });
            return String::from(pattern);
        }
        let slot_props_var = cstr!("_slotProps{}", self.slot_scope_count);
        self.slot_scope_count += 1;
        self.slot_scopes.push(SlotScope {
            names: parse_destructure_names(destructure_pattern),
            slot_props_var: slot_props_var.clone(),
            for_depth: self.for_scopes.len(),
            whole: false,
        });
        slot_props_var
    }

    /// Pop the current slot scope
    pub(crate) fn pop_slot_scope(&mut self) {
        self.slot_scopes.pop();
    }
}

/// For-loop scope entry
#[derive(Debug, Clone)]
pub(crate) struct ForScope {
    /// Value alias (e.g., "item") -> "_for_item{depth}"
    pub(crate) value_alias: Option<String>,
    /// Key alias (e.g., "index" or "key") -> "_for_key{depth}"
    pub(crate) key_alias: Option<String>,
    /// Index alias -> "_for_index{depth}"
    pub(crate) index_alias: Option<String>,
    /// Depth of for nesting (0-based)
    pub(crate) depth: usize,
}

/// Slot scope entry for scoped slots
#[derive(Debug, Clone)]
pub(crate) struct SlotScope {
    /// Destructured variable names (e.g., ["item", "index"] from "{ item, index }")
    pub(crate) names: std::vec::Vec<String>,
    /// Slot props variable (e.g., "_slotProps0")
    pub(crate) slot_props_var: String,
    /// Loop scopes already active when this slot scope opened: it is nested
    /// inside exactly those, so its names shadow theirs and no later ones.
    pub(crate) for_depth: usize,
    /// The pattern is a plain identifier naming the whole props object.
    pub(crate) whole: bool,
}
