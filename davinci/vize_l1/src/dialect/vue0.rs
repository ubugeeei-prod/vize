//! Vue 0 template capability table.

use super::vue::{LegacyDialectCapabilities, feature};

/// Existing V0_10 template syntax, resolved once per file.
pub const V0_10: LegacyDialectCapabilities = feature::computed::dollar_get_set(V0_11);
/// Existing V0_11 template syntax, resolved once per file.
pub const V0_11: LegacyDialectCapabilities = feature::triple_mustache::enable(
    feature::attribute::interpolation(feature::component::directive(
        feature::inheritance::with_directive(feature::directive::clause(feature::repeat::enable(
            feature::filters::space_separated(feature::BASE),
        ))),
    )),
);
