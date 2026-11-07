use super::LegacyDialectCapabilities;

pub const fn dollar_get_set(mut features: LegacyDialectCapabilities) -> LegacyDialectCapabilities {
    features.computed_dollar_get_set = true;
    features
}
