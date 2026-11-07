use super::LegacyDialectCapabilities;

pub const fn directive(mut features: LegacyDialectCapabilities) -> LegacyDialectCapabilities {
    features.v_component_directive = true;
    features
}
