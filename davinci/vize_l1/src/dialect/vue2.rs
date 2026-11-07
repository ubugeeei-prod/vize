//! Vue 2 template capability table.

pub mod surface;
pub mod text;

use super::vue::{LegacyDialectCapabilities, feature};

/// Existing V2 template syntax, resolved once per file.
pub const CAPABILITIES: LegacyDialectCapabilities = feature::event::vue2_sugar(
    feature::slot_scope::enable(feature::filters::call_style(feature::BASE)),
);
