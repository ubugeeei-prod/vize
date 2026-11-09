//! Default Vue 3 template capability table.

pub mod directive;
pub(crate) mod for_head;
pub(crate) mod operand;
pub mod surface;
pub use directive::{DirectiveKind, VueDirectives};

/// Component classification for the default Vue lint policy.
///
/// Uppercase tags include the core built-ins. The lowercase dynamic component
/// is Vue syntax only while its header is active; exact or inherited `v-pre`
/// leaves it literal. Configured native/custom element resolution is separate.
pub fn is_lint_component(tag: &str, dynamic_component: bool) -> bool {
    (dynamic_component && tag == "component") || tag.chars().next().is_some_and(char::is_uppercase)
}

use super::vue::{LegacyDialectCapabilities, feature};

/// No legacy syntax; colon-style directive arguments.
pub const CAPABILITIES: LegacyDialectCapabilities = feature::BASE;
