use super::LegacyDialectCapabilities;

pub const fn interpolation(mut features: LegacyDialectCapabilities) -> LegacyDialectCapabilities {
    features.attr_value_interpolation = true;
    features
}
