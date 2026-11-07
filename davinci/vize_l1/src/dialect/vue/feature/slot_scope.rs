use super::LegacyDialectCapabilities;

pub const fn enable(mut features: LegacyDialectCapabilities) -> LegacyDialectCapabilities {
    features.scoped_slot_attrs = true;
    features
}
