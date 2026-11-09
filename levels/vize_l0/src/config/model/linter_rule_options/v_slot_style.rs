use serde::{Deserialize, Serialize};

/// Configuration accepted by `vue/v-slot-style`.
///
/// A string applies one spelling everywhere. The object form keeps the
/// upstream position-specific policy and defaults.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum VSlotStyle {
    Uniform(VSlotUniformStyle),
    ByPosition(VSlotStyleOptions),
}

/// Uniform spellings supported for every slot position.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum VSlotUniformStyle {
    Shorthand,
    Longform,
}

/// Position-specific `vue/v-slot-style` options.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct VSlotStyleOptions {
    pub at_component: VSlotStyleValue,
    #[serde(rename = "default")]
    pub default_slot: VSlotStyleValue,
    pub named: VSlotNamedStyle,
}

impl Default for VSlotStyleOptions {
    fn default() -> Self {
        Self {
            at_component: VSlotStyleValue::VSlot,
            default_slot: VSlotStyleValue::Shorthand,
            named: VSlotNamedStyle::Shorthand,
        }
    }
}

/// Spelling available to default slots.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum VSlotStyleValue {
    Shorthand,
    Longform,
    VSlot,
}

/// Spelling available to named slots.
///
/// Bare `v-slot` cannot preserve a named slot, so it is excluded at the type
/// and schema boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum VSlotNamedStyle {
    Shorthand,
    Longform,
}
