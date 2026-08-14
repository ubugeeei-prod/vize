//! For-loop and slot scope entries the Vapor generate context stacks.

use vize_carton::String;

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
#[allow(dead_code)]
pub(crate) struct SlotScope {
    /// Destructured variable names (e.g., ["item", "index"] from "{ item, index }")
    pub(crate) names: std::vec::Vec<String>,
    /// Slot props variable (e.g., "_slotProps0")
    pub(crate) slot_props_var: String,
}
