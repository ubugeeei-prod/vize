//! Owned workspace contracts from the original setup parse demand.

use vize_l0::Allocator;

use super::parse;
use super::recovery::parse_program_for_analysis;
use crate::macros::MacroTracker;
use oxc_span::SourceType;

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

/// Read only the emits and slots used by closed-file workspace symbols.
///
/// This keeps the original parser/recovery and registered statement shapes,
/// while leaving the full-demand parser above available to its callers. Only
/// event/slot facts are promised; unrelated macro metadata is not collected.
/// Every returned fact is owned and can survive resetting the caller's arena.
#[doc(hidden)]
pub fn parse_workspace_symbol_contracts_in(allocator: &Allocator, source: &str) -> MacroTracker {
    vize_carton::micro_count!("croquis.setup.parse_requests.bytes", source.len());
    let source_type = SourceType::from_path("script.ts").unwrap_or_default();
    let ret = parse_program_for_analysis(allocator.as_oxc(), source, source_type);
    if ret.panicked {
        return MacroTracker::default();
    }
    super::extract::workspace_symbol_contracts(&ret.program, source)
}
