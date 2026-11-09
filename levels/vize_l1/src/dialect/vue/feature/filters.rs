use super::LegacyDialectCapabilities;

/// Add call-style Vue 2 filter syntax.
pub const fn call_style(mut features: LegacyDialectCapabilities) -> LegacyDialectCapabilities {
    features.supports_filters = true;
    features
}

/// Add the space-separated Vue 0/1 filter grammar.
pub const fn space_separated(mut features: LegacyDialectCapabilities) -> LegacyDialectCapabilities {
    features.supports_filters = true;
    features.space_separated_filter_args = true;
    features
}
