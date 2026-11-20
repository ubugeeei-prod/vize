use super::LegacyDialectCapabilities;

pub const fn interpolation(mut features: LegacyDialectCapabilities) -> LegacyDialectCapabilities {
    features.attr_value_interpolation = true;
    features
}

/// Apply the plain-attribute map semantics shared by pre-Vue-3 compilers.
pub const fn keep_last_duplicate(
    mut features: LegacyDialectCapabilities,
) -> LegacyDialectCapabilities {
    features.duplicate_static_attrs_keep_last = true;
    features
}
