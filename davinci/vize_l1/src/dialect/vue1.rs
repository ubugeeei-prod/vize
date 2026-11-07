//! Vue 1 template capability table.

pub mod surface;
pub mod text;

use super::vue::{LegacyDialectCapabilities, feature};

/// Existing V1 template syntax, resolved once per file.
pub const CAPABILITIES: LegacyDialectCapabilities = feature::triple_mustache::enable(
    feature::attribute::interpolation(feature::filters::space_separated(feature::BASE)),
);
