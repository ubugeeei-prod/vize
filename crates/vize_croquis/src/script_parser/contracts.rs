//! Owned workspace contracts from the original setup parse demand.

use vize_l0::Allocator;

use super::parse;
use crate::macros::MacroTracker;

/// Parse the workspace collector's exact demand using a caller-owned arena.
///
/// The returned tracker owns its strings and declaration ranges. Parser ASTs
/// and recovery views are gone before this returns, so the caller may reset
/// the allocator while retaining this tracker.
#[doc(hidden)]
pub fn parse_workspace_contracts_in(allocator: &Allocator, source: &str) -> MacroTracker {
    vize_carton::micro_count!("croquis.setup.parse_requests.bytes", source.len());
    parse::parse_script_setup_with_exports_capture_in::<false>(
        allocator.as_oxc(),
        source,
        None,
        false,
        false,
        true,
        false,
        None,
        false,
    )
    .macros
}
