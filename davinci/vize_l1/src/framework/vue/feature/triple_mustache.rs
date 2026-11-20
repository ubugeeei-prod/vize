use super::LegacyDialectCapabilities;

pub const fn enable(mut features: LegacyDialectCapabilities) -> LegacyDialectCapabilities {
    features.raw_html_interpolation = true;
    features
}
