//! Script macro facts with the exact demand used by a script-only Drawer.

use crate::macros::MacroTracker;

/// Extract owned macro facts without constructing an SFC-wide Croquis.
///
/// The existing parser owns recovery, aliases, declaration ranges and ordering.
/// This uses the script-only Drawer's demand: no generic/JSX, unused binding,
/// diagnostic, occurrence or module-export collection. Other script facts are
/// dropped before the owned tracker reaches its consumer.
pub fn parse_script_setup_macros(source: &str) -> MacroTracker {
    super::parse_script_setup_with_exports::<false>(source, None, false, false, true, false, None)
        .macros
}
