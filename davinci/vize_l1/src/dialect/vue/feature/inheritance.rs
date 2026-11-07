use super::LegacyDialectCapabilities;

pub const fn with_directive(mut features: LegacyDialectCapabilities) -> LegacyDialectCapabilities {
    features.v_with_directive = true;
    features
}
