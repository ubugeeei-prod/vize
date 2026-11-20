use super::LegacyDialectCapabilities;

/// Add Vue 2 `.native` and numeric key-code event sugar.
pub const fn vue2_sugar(mut features: LegacyDialectCapabilities) -> LegacyDialectCapabilities {
    features.v2_event_sugar = true;
    features
}
