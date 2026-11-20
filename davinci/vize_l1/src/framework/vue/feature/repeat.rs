use super::LegacyDialectCapabilities;

pub const fn enable(mut features: LegacyDialectCapabilities) -> LegacyDialectCapabilities {
    features.v_repeat_syntax = true;
    features
}
